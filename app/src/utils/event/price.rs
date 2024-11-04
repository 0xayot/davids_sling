use crate::{
  db,
  utils::{
    misc::{PriceAnalyzer, PriceTrend},
    notifications::{notify_user_by_telegram, notify_users},
    swap::solana::{execute_sell_trade, record_transaction},
    wallets::solana::get_token_balance,
  },
};
use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use entity::{raydium_token_launches, token_prices as prices, trade_orders, users, wallets};
use futures::future::join_all;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn handle_price_update(ca: &str, price: f64) -> Result<()> {
  let db = db::connect_db()
    .await
    .context("Failed to connect to the database")?;

  let five_minutes_ago = Utc::now() - Duration::minutes(5);

  let prices = prices::Entity::find()
    .filter(prices::Column::ContractAddress.eq(ca))
    .filter(prices::Column::CreatedAt.gt(five_minutes_ago))
    .all(&db)
    .await
    .context("Failed to retrieve prices")?;

  let token_launch = raydium_token_launches::Entity::find()
    .filter(raydium_token_launches::Column::Evaluation.eq("track"))
    .filter(raydium_token_launches::Column::ContractAddress.eq(ca))
    .one(&db)
    .await
    .context("Failed to retrieve token launch")?;

  if token_launch.is_none() {
    eprintln!("Token launch not found for contract address: {}", ca);
    return Ok(());
  }

  let price_list: Vec<f64> = prices
    .iter()
    .filter_map(|price_record| price_record.price.map(|p| p as f64))
    .collect();

  let mut tasks = vec![];

  if price_list.len() > 5 {
    let analyzer = PriceAnalyzer::new(3, 20.0);
    let reference_price = price_list.first().cloned().unwrap_or(0.0);

    if reference_price == 0.0 {
      return Ok(());
    }

    let simple_trend = analyzer.analyze_trend(&price_list);

    if simple_trend == PriceTrend::Increasing {
      // Do nothing for now
      if price >= reference_price * 2.0 {
        let notification_message = format!(
          "the price of {} is up from {:?} to {}",
          ca,
          token_launch.as_ref().and_then(|t| t.launch_price_usd),
          price
        );
        notify_users(notification_message, &db).await?;
      }
    } else if simple_trend == PriceTrend::Decreasing {
      let trade_orders = trade_orders::Entity::find()
        .filter(trade_orders::Column::ContractAddress.eq(ca))
        .filter(trade_orders::Column::Strategy.eq("launch_stop_loss"))
        .find_with_related(users::Entity)
        .all(&db)
        .await
        .context("Failed to retrieve active trade orders")?;

      for (order, related_users) in trade_orders {
        let user = related_users.into_iter().next();
        let database = db.clone();
        let task = tokio::spawn(async move {
          process_single_stop_loss_order(order, user, &database, reference_price, price).await
        });
        tasks.push(task);
      }
    }
  } else {
    let reference_price = token_launch
      .as_ref()
      .and_then(|launch| launch.launch_price_usd) // Get launch_price_usd if available
      .map(|price| price as f64)
      .unwrap_or_else(|| price_list.first().cloned().unwrap_or(0.0));

    if reference_price == 0.0 {
      return Ok(());
    }

    // Check if the current price has dropped below 40%
    let price_change_threshold = reference_price * 0.6;

    if price_change_threshold > price {
      let trade_orders = trade_orders::Entity::find()
        .filter(trade_orders::Column::ContractAddress.eq(ca))
        .filter(trade_orders::Column::Strategy.eq("launch_stop_loss"))
        .find_with_related(users::Entity)
        .all(&db)
        .await
        .context("Failed to retrieve active trade orders")?;

      // Handle the stop loss logic here
      for (order, related_users) in trade_orders {
        let user = related_users.into_iter().next();
        let database = db.clone();
        let task = tokio::spawn(async move {
          process_single_stop_loss_order(order, user, &database, reference_price, price).await
        });

        tasks.push(task);
      }
    }
  }

  join_all(tasks).await;
  let _ = db.close().await;
  Ok(())
}

async fn process_single_stop_loss_order(
  order: trade_orders::Model,
  user: Option<users::Model>,
  db: &DatabaseConnection,
  entry_price: f64,
  latest_price: f64,
) -> Result<()> {
  let user = match user {
    Some(user) => user,
    None => {
      eprintln!("No user found for order {}", order.id);
      return Ok(()); // Early return if no user found
    }
  };

  let wallet = wallets::Entity::find_by_id(order.wallet_id)
    .one(db)
    .await
    .unwrap()
    .context("Failed to get wallet")?;

  // Fetch the token balance
  let balance = get_token_balance(&wallet.address, &order.contract_address)
    .await
    .map_err(|e| anyhow!("Failed to get balance: {}", e))?;

  // Proceed if there is a positive balance
  if balance.ui_amount > 0.0 {
    let sell_attempt = execute_sell_trade(
      user.id,
      wallet.id,
      &order.contract_address,
      balance.ui_amount,
      &wallet.address,
      db,
      6,
    )
    .await;

    let sell_size_usd = latest_price * balance.ui_amount;

    if let Ok(attempt) = sell_attempt {
      record_transaction(
        db,
        user.id,
        wallet.id,
        &order.contract_address,
        attempt,
        balance.ui_amount,
        sell_size_usd,
      )
      .await?;
    }

    // Notify user about the sale
    if let Ok(tg_id_parsed) = user.tg_id.parse::<i64>() {
      let message = format!(
        "{}: Token {} was bought at launch and sold at {}, for ${:.2}. Entry Price: {}",
        &order.strategy, &order.contract_address, latest_price, sell_size_usd, entry_price
      );

      if let Err(e) = notify_user_by_telegram(tg_id_parsed, &message).await {
        eprintln!("Error notifying user {}: {}", tg_id_parsed, e);
      }
    }
  } else {
    // Notify user if no balance was available for sale
    if let Ok(tg_id_parsed) = user.tg_id.parse::<i64>() {
      let message = format!(
        "{}: Token {} was bought at launch but no balance available to sell for {}; entry price {}",
        &order.strategy, &order.contract_address, latest_price, entry_price
      );

      if let Err(e) = notify_user_by_telegram(tg_id_parsed, &message).await {
        eprintln!("Error notifying user {}: {}", tg_id_parsed, e);
      }
    }
  }

  Ok(())
}
