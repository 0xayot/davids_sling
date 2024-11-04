use crate::{db, integrations::raydium::RaydiumPriceFetcher};
use chrono::{Duration, Utc};
use entity::{onchain_transactions, token_prices as prices, tokens, trade_orders, users, wallets};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::utils::swap::solana::execute_user_swap_txs;
use crate::utils::wallets::solana::get_token_balance;

// Ideally, I should have used events that are emitted on price updates.

// Tracks the price of a token over the past 5 minutes and sells if the price has dropped.

use futures::future::join_all;

async fn process_single_order(
  order: trade_orders::Model,
  user: Option<users::Model>,
  db: &DatabaseConnection,
  token: &tokens::Model,
  latest_price: f64,
) -> Result<(), Box<dyn std::error::Error>> {
  // Skip if price condition not met
  if order.target_price < latest_price as f32 {
    return Ok(());
  }

  // Validate user exists
  let user = match user {
    Some(user) => user,
    None => {
      eprintln!("No user found for order {}", order.id);
      return Ok(());
    }
  };

  let wallet = match wallets::Entity::find_by_id(order.wallet_id).one(db).await? {
    Some(wallet) => wallet,
    None => {
      eprintln!("No wallet found for order {}", order.id);
      return Ok(());
    }
  };

  // Get token balance
  let balance = match get_token_balance(&wallet.address, &order.contract_address).await {
    Ok(balance) => balance,
    Err(e) => {
      eprintln!("Error fetching balance for order {}: {}", order.id, e);
      return Ok(());
    }
  };

  let raydium_client = RaydiumPriceFetcher::new();

  // Get swap quote
  let quote = match raydium_client
    .get_swap_quote(
      &token.contract_address,
      "So11111111111111111111111111111111111111112",
      &balance.amount.to_string(),
      &50.to_string(),
    )
    .await
  {
    Ok(quote) => quote,
    Err(e) => {
      eprintln!("Error getting swap quote for order {}: {}", order.id, e);
      return Ok(());
    }
  };

  // Get token public key
  let token_public_key = match &token.token_public_key {
    Some(key) => key.as_str(),
    None => {
      eprintln!("Token public key missing for token {}", token.id);
      return Ok(());
    }
  };

  // Get swap transaction
  let swap = match raydium_client
    .get_swap_tx(
      &wallet.address,
      quote,
      &token.contract_address,
      "So11111111111111111111111111111111111111112",
      token_public_key,
    )
    .await
  {
    Ok(swap) => swap,
    Err(e) => {
      eprintln!(
        "Error getting swap transaction for order {}: {}",
        order.id, e
      );
      return Ok(());
    }
  };

  // let tx = swap.first().unwrap()

  // Execute swap and record transaction
  match execute_user_swap_txs(user.id, wallet.id, db.clone(), swap).await {
    Ok(attempt) => {
      let transaction = onchain_transactions::ActiveModel {
        user_id: Set(user.id),
        wallet_id: Set(wallet.id),
        transaction_hash: Set(Some(attempt.transaction_hash)),
        chain: Set("solana".to_string()),
        source: Set(Some("raydium".to_string())),
        status: Set(Some(
          if attempt.success {
            "confirmed"
          } else {
            "submitted"
          }
          .to_string(),
        )),
        r#type: Set(Some("swap".to_string())),
        value_native: Set(Some(0.0)),
        value_usd: Set(Some(0.0)),
        from_token: Set(Some(token.contract_address.clone())),
        to_token: Set(Some(
          "So11111111111111111111111111111111111111112".to_string(),
        )),
        ..Default::default()
      };

      if let Err(e) = onchain_transactions::Entity::insert(transaction)
        .exec(db)
        .await
      {
        eprintln!("Failed to record transaction for order {}: {}", order.id, e);
      }
    }
    Err(e) => {
      eprintln!("Error executing swap for order {}: {}", order.id, e);
    }
  }

  Ok(())
}

pub async fn default_stop_loss_strategy_solana() -> Result<(), Box<dyn std::error::Error>> {
  let db = db::connect_db().await?;

  let tokens = tokens::Entity::find()
    .filter(tokens::Column::Chain.eq("solana"))
    .all(&db)
    .await?;

  let five_minutes_ago = Utc::now() - Duration::minutes(5);

  let recent_prices = prices::Entity::find()
    .filter(prices::Column::CreatedAt.gt(five_minutes_ago))
    .all(&db)
    .await?;

  let latest_price = recent_prices
    .last()
    .and_then(|p| Some(p.price.unwrap_or(0.0)))
    .unwrap_or(0.0);

  let mut tasks = vec![];

  for token in tokens {
    let db_clone = db.clone();

    let task = tokio::spawn(async move {
      let stop_loss_orders = trade_orders::Entity::find()
        .filter(trade_orders::Column::Strategy.eq("stop_loss"))
        .filter(trade_orders::Column::TokenId.eq(token.id))
        .filter(trade_orders::Column::CreatedBy.eq("app"))
        .filter(trade_orders::Column::Active.eq(true))
        .filter(trade_orders::Column::ContractAddress.eq(token.contract_address.clone()))
        .find_with_related(users::Entity)
        .all(&db_clone)
        .await;

      match stop_loss_orders {
        Ok(orders) => {
          for (order, related_users) in orders {
            // Get the first (and should be only) related user
            let user = related_users.into_iter().next();

            if let Err(e) =
              process_single_order(order, user, &db_clone, &token, latest_price as f64).await
            {
              eprintln!("Error processing order: {}", e);
            }
          }
        }
        Err(e) => {
          eprintln!("Error fetching stop loss orders: {}", e);
        }
      }
    });

    tasks.push(task);
  }

  join_all(tasks).await;

  Ok(())
}
