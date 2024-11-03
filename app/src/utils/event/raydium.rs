use anyhow::{Context, Result};
use entity::{raydium_token_launches, users};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::time::{sleep, Duration};

use crate::{
  db,
  integrations::dexscreener::{self},
  utils::{notifications::notify_user_by_telegram, price::solana::fetch_token_price},
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

  let lower_limit: f64 = env::var("LOWER_LAUNCH_LIMIT")
    .unwrap_or_else(|_| "30.0".to_string())
    .parse()
    .expect("LOWER_LAUNCH_LIMIT must be a valid float");

  let mid_limit: f64 = env::var("MID_LAUNCH_LIMIT")
    .unwrap_or_else(|_| "70.0".to_string())
    .parse()
    .expect("MID_LAUNCH_LIMIT must be a valid float");

  let normal_limit: f64 = env::var("NORMAL_LAUNCH_LIMIT")
    .unwrap_or_else(|_| "100.0".to_string())
    .parse()
    .expect("NORMAL_LAUNCH_LIMIT must be a valid float");

  let pro_limit: f64 = env::var("PRO_LAUNCH_LIMIT")
    .unwrap_or_else(|_| "250.0".to_string())
    .parse()
    .expect("PRO_LAUNCH_LIMIT must be a valid float");

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
    match notify_users_of_launch(notification_message, db).await {
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

    match notify_users_of_launch(notification_message, db).await {
      Ok(_) => println!("notified users of crazy launch"),
      Err(e) => eprintln!("An error occured: \n {:?}", e),
    };
  }

  // TODO: let is_boosted_token = /* Your logic to determine if the token is boosted */;
}

pub async fn notify_users_of_launch(msg: String, db: DatabaseConnection) -> Result<()> {
  let users = users::Entity::find()
    .filter(users::Column::TgId.is_not_null())
    .all(&db) // Dereferencing Arc to get a reference to DatabaseConnection
    .await
    .context("Database error")?;

  let mut tasks = vec![];

  for user in users {
    let tg_id_string = user.tg_id.clone(); // Clone to avoid ownership issues
    let message = msg.clone();

    // Attempt to parse tg_id, skip iteration if it fails
    if let Ok(tg_id) = tg_id_string.parse::<i64>() {
      let task = tokio::spawn(async move {
        if let Err(e) = notify_user_by_telegram(tg_id, &message).await {
          eprintln!("Error notifying user {}: {}", tg_id, e);
        }
      });
      tasks.push(task);
    } else {
      eprintln!("Skipping user with invalid tg_id: {}", tg_id_string);
    }
  }
  futures::future::join_all(tasks).await;

  Ok(())
}
