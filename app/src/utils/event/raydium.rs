use anyhow::{anyhow, Context, Result};
use entity::{raydium_token_launches, users, wallets};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::{
  task,
  time::{sleep, Duration},
};

use crate::{
  db,
  integrations::dexscreener::{self},
  utils::{
    notifications::{notify_user_by_telegram, notify_users},
    price::solana::fetch_token_price,
    swap::solana::{create_stop_loss_order, execute_buy_trade, record_transaction, TradeParams},
    wallets::solana::{find_or_create_token, get_token_details, get_wallet_sol_balance},
  },
};
#[derive(Debug, Deserialize, Serialize)]
pub struct LPInfo {
  address: String,
  decimals: u8,
  lp_amount: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RaydiumTokenEvent {
  creator: String,
  timestamp: String,
  base_info: LPInfo,
  quote_info: LPInfo,
}

pub async fn handle_token_created_event(data: RaydiumTokenEvent) {
  let db = db::connect_db()
    .await
    .expect("Failed to connect to the database");

  let lower_limit: f64 = env::var("LOWER_LIQUIDITY_LAUNCH_LIMIT")
    .unwrap_or_else(|_| "30.0".to_string())
    .parse()
    .expect("LOWER_LIQUIDITY_LAUNCH_LIMIT must be a valid float");

  let mid_limit: f64 = env::var("MID_LIQUIDITY_LAUNCH_LIMIT")
    .unwrap_or_else(|_| "70.0".to_string())
    .parse()
    .expect("MID_LIQUIDITY_LAUNCH_LIMIT must be a valid float");

  let normal_limit: f64 = env::var("NORMAL_LIQUIDITY_LAUNCH_LIMIT")
    .unwrap_or_else(|_| "100.0".to_string())
    .parse()
    .expect("NORMAL_LIQUIDITY_LAUNCH_LIMIT must be a valid float");

  let pro_limit: f64 = env::var("PRO_LIQUIDITY_LAUNCH_LIMIT")
    .unwrap_or_else(|_| "250.0".to_string())
    .parse()
    .expect("PRO_LIQUIDITY_LAUNCH_LIMIT must be a valid float");

  let pool_sol_liquidity = data.quote_info.lp_amount;
  let sol_price = fetch_token_price(&data.quote_info.address).await.unwrap();
  let pool_sol_liquidity_usd = sol_price * pool_sol_liquidity;
  let contract_address = &data.base_info.address;

  // Wait for 5 seconds before calling dexscreener becuse dexscreener may not have registered the launch
  // sleep(Duration::from_secs(5)).await;

  let token_info_from_dexscreener =
    match dexscreener::fetch_token_data(&data.base_info.address).await {
      Ok(data) => Some(data),
      Err(_e) => None,
    };

  if pool_sol_liquidity <= lower_limit {
    println!("loser liquidity.");

    let mut loser_launch = raydium_token_launches::ActiveModel {
      contract_address: Set(contract_address.clone()),
      creator_address: Set(data.creator),
      evaluation: Set(Some("skip".to_string())),
      launch_class: Set(Some("below_limit".to_string())),
      launch_liquidity: Set(data.base_info.lp_amount as f32),
      launch_liquidity_usd: Set(pool_sol_liquidity_usd as f32),
      ..Default::default()
    };
    if let Some(info) = token_info_from_dexscreener {
      if let Some(first_pair) = info.pairs.get(0) {
        loser_launch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
      }
    } else {
      println!("Retrying dexscreener below launch limit.");

      match dexscreener::fetch_token_data(contract_address).await {
        Ok(data) => {
          if let Some(first_pair) = data.pairs.get(0) {
            loser_launch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
          }
        }
        Err(_e) => {
          eprintln!("Failed to query dexscreener");
        }
      };
    }

    let _ = raydium_token_launches::Entity::insert(loser_launch)
      .exec(&db)
      .await
      .map_err(|e| e.to_string());
  } else if pool_sol_liquidity > lower_limit && pool_sol_liquidity < mid_limit {
    let mut lower_limit_launch = raydium_token_launches::ActiveModel {
      contract_address: Set(contract_address.clone()),
      creator_address: Set(data.creator),
      evaluation: Set(Some("skip".to_string())),
      launch_class: Set(Some("lower_limit".to_string())),
      launch_liquidity: Set(data.base_info.lp_amount as f32),
      launch_liquidity_usd: Set(pool_sol_liquidity_usd as f32),
      ..Default::default()
    };
    if let Some(info) = token_info_from_dexscreener {
      if let Some(first_pair) = info.pairs.get(0) {
        lower_limit_launch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
      }
    } else {
      println!("retrying dex screener for lower limit launch.");
      sleep(Duration::from_secs(30)).await;

      match dexscreener::fetch_token_data(contract_address).await {
        Ok(data) => {
          if let Some(first_pair) = data.pairs.get(0) {
            lower_limit_launch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
          }
        }
        Err(_e) => {
          eprintln!("Failed to query dexscreener");
        }
      };
    }
    let _ = raydium_token_launches::Entity::insert(lower_limit_launch)
      .exec(&db)
      .await
      .map_err(|e| e.to_string());
  } else if pool_sol_liquidity >= mid_limit && pool_sol_liquidity < normal_limit {
    println!("processing mid launch.");

    let mut mid_limit_launch = raydium_token_launches::ActiveModel {
      contract_address: Set(contract_address.clone()),
      creator_address: Set(data.creator),
      evaluation: Set(Some("track".to_string())),
      launch_class: Set(Some("mid_launch".to_string())),
      launch_liquidity: Set(data.base_info.lp_amount as f32),
      launch_liquidity_usd: Set(pool_sol_liquidity_usd as f32),
      ..Default::default()
    };
    if let Some(info) = token_info_from_dexscreener {
      if let Some(first_pair) = info.pairs.get(0) {
        mid_limit_launch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
      }
    } else {
      println!("retrying dex screener for mid limit launch.");
      sleep(Duration::from_secs(30)).await;

      match dexscreener::fetch_token_data(contract_address).await {
        Ok(data) => {
          if let Some(first_pair) = data.pairs.get(0) {
            mid_limit_launch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
          }
        }
        Err(_e) => {
          eprintln!("Failed to query dexscreener");
        }
      };
    }
    let _ = raydium_token_launches::Entity::insert(mid_limit_launch)
      .exec(&db)
      .await
      .map_err(|e| e.to_string());

    //TODO:  Check if it's a pump.fun if yes buy
  } else if pool_sol_liquidity >= normal_limit && pool_sol_liquidity < pro_limit {
    println!("Liquidity is between the normal limit and pro limit.");

    let mut good_launch = raydium_token_launches::ActiveModel {
      contract_address: Set(contract_address.clone()),
      creator_address: Set(data.creator),
      evaluation: Set(Some("track".to_string())),
      launch_class: Set(Some("pro_launch".to_string())),
      launch_liquidity: Set(data.base_info.lp_amount as f32),
      launch_liquidity_usd: Set(pool_sol_liquidity_usd as f32),
      ..Default::default()
    };
    let notification_message = format!(
      "a good launch {} with {} liquidity (${})",
      contract_address, pool_sol_liquidity, pool_sol_liquidity_usd
    );
    if let Some(info) = token_info_from_dexscreener {
      if let Some(first_pair) = info.pairs.get(0) {
        good_launch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
      }
    } else {
      println!("retrying dex screener for good launch.");
      sleep(Duration::from_secs(30)).await;

      match dexscreener::fetch_token_data(contract_address).await {
        Ok(data) => {
          if let Some(first_pair) = data.pairs.get(0) {
            good_launch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
          }
        }
        Err(_e) => {
          eprintln!("Failed to query dexscreener");
        }
      };
    }
    let _ = raydium_token_launches::Entity::insert(good_launch)
      .exec(&db)
      .await
      .map_err(|e| e.to_string());
    match notify_users(notification_message, &db).await {
      Ok(_) => println!("notified users of crazy launch"),
      Err(e) => eprintln!("An error occured: \n {:?}", e),
    };
  } else if pool_sol_liquidity >= pro_limit {
    let mut crazy_launch = raydium_token_launches::ActiveModel {
      contract_address: Set(contract_address.clone()),
      creator_address: Set(data.creator),
      evaluation: Set(Some("track".to_string())),
      launch_class: Set(Some("crazy_launch".to_string())),
      launch_liquidity: Set(data.base_info.lp_amount as f32),
      launch_liquidity_usd: Set(pool_sol_liquidity_usd as f32),
      ..Default::default()
    };
    let notification_message = format!(
      "a crazy launch {} with {} liquidity (${})",
      contract_address, pool_sol_liquidity, pool_sol_liquidity_usd
    );
    if let Some(info) = token_info_from_dexscreener {
      if let Some(first_pair) = info.pairs.get(0) {
        crazy_launch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
      }
    } else {
      println!("retrying dex screener for crazy launch.");
      sleep(Duration::from_secs(30)).await;

      match dexscreener::fetch_token_data(contract_address).await {
        Ok(data) => {
          if let Some(first_pair) = data.pairs.get(0) {
            crazy_launch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
          }
        }
        Err(_e) => {
          eprintln!("Failed to query dexscreener");
        }
      };
    }
    let _ = raydium_token_launches::Entity::insert(crazy_launch)
      .exec(&db)
      .await
      .map_err(|e| e.to_string());

    match notify_users(notification_message, &db).await {
      Ok(_) => println!("notified users of crazy launch"),
      Err(e) => eprintln!("An error occured: \n {:?}", e),
    };

    match buy_token_on_launch(&contract_address, db).await {
      Ok(_) => println!("attempted buy of crazy"),
      Err(e) => eprintln!("An error occured: \n {:?}", e),
    }
  }

  // TODO: let is_boosted_token = /* Your logic to determine if the token is boosted */;
}

// at the moment we do not have a system for actually storing the settings of each user so all users with TG get coins as the launch 😬
pub async fn buy_token_on_launch(ca: &str, db: DatabaseConnection) -> Result<()> {
  let users = users::Entity::find()
    .filter(users::Column::TgId.is_not_null())
    .all(&db) // Dereferencing Arc to get a reference to DatabaseConnection
    .await
    .context("Database error")?;

  let token = get_token_details(ca)
    .await
    .context("Failed to get token details:")?;

  let token_id = find_or_create_token(&db, &token, ca).await?;
  let mut tasks = vec![];
  for user in users {
    let trade_params = fetch_trade_parameters(ca).await?;
    let contract_address = ca.to_string();
    let db = db.clone();
    let task = task::spawn(async move {
      process_user_trade(user, &contract_address, token_id, &db, &trade_params).await
    });

    tasks.push(task);
  }

  // Await all tasks to complete
  for task in tasks {
    if let Err(e) = task.await {
      eprintln!("Error in task: {}", e);
    }
  }

  Ok(())
}

async fn fetch_trade_parameters(ca: &str) -> Result<TradeParams> {
  let sol_price = fetch_token_price("So11111111111111111111111111111111111111112")
    .await
    .map_err(|e| anyhow!("Failed to fetch SOL price: {}", e))?;

  let token_price = fetch_token_price(ca)
    .await
    .map_err(|e| anyhow!("Failed to fetch token price: {}", e))?;

  let buy_size_percentage = env::var("BUY_SIZE")
    .unwrap_or_else(|_| "10.0".to_string())
    .parse()
    .map_err(|e| anyhow!("Invalid BUY_SIZE: {}", e))?;

  let launch_size_lower_limit = env::var("LAUNCH_BUY_SIZE_LOWER_LIMIT")
    .unwrap_or_else(|_| "5.0".to_string())
    .parse()
    .map_err(|e| anyhow!("Invalid LAUNCH_BUY_SIZE_LOWER_LIMIT: {}", e))?;

  let launch_stop_loss = env::var("LAUNCH_STOP_LOSS_PERCENTAGE")
    .ok()
    .and_then(|value| value.parse::<f32>().ok())
    .unwrap_or(20.0);

  Ok(TradeParams {
    sol_price,
    token_price,
    buy_size_percentage,
    launch_size_lower_limit,
    launch_stop_loss,
  })
}

async fn process_user_trade(
  user: users::Model,
  ca: &str,
  token_id: i32,
  db: &DatabaseConnection,
  params: &TradeParams,
) -> Result<()> {
  // Get user's wallet
  let wallet = wallets::Entity::find()
    .filter(wallets::Column::UserId.eq(user.id))
    .one(db)
    .await?
    .ok_or_else(|| anyhow!("No wallet found for user {}", user.id))?;

  let wallet_sol_balance = get_wallet_sol_balance(&wallet.address)
    .await
    .map_err(|e| anyhow!("Failed to get wallet balance: {}", e))?;

  let wallet_sol_value_usd = params.sol_price * wallet_sol_balance;

  if wallet_sol_value_usd < params.launch_size_lower_limit {
    return Ok(());
  }

  let buy_size = wallet_sol_balance * (params.buy_size_percentage / 100.0);
  let buy_size_usd = buy_size * params.sol_price;

  if buy_size < params.launch_size_lower_limit {
    // create a stop loss with an arbitrary amount to track the sell price
    return Ok(());
  }

  let swap_result = execute_buy_trade(user.id, wallet.id, ca, buy_size, &wallet.address, db).await;

  if let Ok(attempt) = swap_result {
    record_transaction(db, user.id, wallet.id, ca, attempt, buy_size, buy_size_usd).await?;
  }

  create_stop_loss_order(db, user.id, wallet.id, token_id, ca, params).await?;

  // Notify user via Telegram

  if let Ok(tg_id_parsed) = user.tg_id.parse::<i64>() {
    let message = format!(
      "Token {} was bought at launch for {}, with ${:.2}",
      ca, params.token_price, buy_size_usd
    );

    if let Err(e) = notify_user_by_telegram(tg_id_parsed, &message).await {
      eprintln!("Error notifying user {}: {}", tg_id_parsed, e);
    }
  }

  Ok(())
}
