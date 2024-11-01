use entity::raydium_token_launches;
use sea_orm::{EntityTrait, Set};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::time::{sleep, Duration};

use crate::{db, integrations::dexscreener, utils::price::solana::fetch_token_price};
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
  // Assuming these are float values, parse them from environment variables
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

  // Copy out pool value
  let pool_sol_liquidity = data.quote_info.lp_amount;
  let sol_price = fetch_token_price(&data.quote_info.address).await.unwrap();
  let pool_sol_liquidity_usd = sol_price * pool_sol_liquidity;
  let contract_address = &data.base_info.address;

  // Wait for 5 seconds before calling dexscreener
  sleep(Duration::from_secs(5)).await;

  let mut token_info_from_dexscreener =
    match dexscreener::fetch_token_data(&data.base_info.address).await {
      Ok(data) => Some(data),
      Err(_e) => None,
    };

  if pool_sol_liquidity <= lower_limit {
    println!("loser liquidity.");

    let mut loserLaunch = raydium_token_launches::ActiveModel {
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
        loserLaunch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
      }
    } else {
      sleep(Duration::from_secs(120)).await;

      match dexscreener::fetch_token_data(contract_address).await {
        Ok(data) => {
          if let Some(first_pair) = data.pairs.get(0) {
            loserLaunch.meta = Set(Some(serde_json::to_value(first_pair).unwrap()));
          }
        }
        Err(_e) => {
          eprintln!("Failed to query dexscreener");
        }
      };
    }
    let _ = raydium_token_launches::Entity::insert(loserLaunch)
      .exec(&db)
      .await
      .map_err(|e| e.to_string());
  } else if pool_sol_liquidity > lower_limit && pool_sol_liquidity < mid_limit {
    // let mut loserLaunch = raydium_token_launches::ActiveModel {
    //   contract_address: Set(data.base_info.address),
    //   creator_address: Set(data.creator),
    //   evaluation: Set(Some("skip".to_string())),
    //   launch_class: Set(Some("below_limit".to_string())),
    //   launch_liquidity: Set(data.base_info.lp_amount as f32),
    //   launch_liquidity_usd: Set(pool_sol_liquidity_usd as f32),
    //   ..Default::default()
    // };
    // if let Some(info) = token_info_from_dexscreener {
    //   loserLaunch.meta = Set(Some(serde_json::to_value(info.pairs[0]).unwrap()));
    // } else {
    //   sleep(Duration::from_secs(120)).await;

    //   match dexscreener::fetch_token_data(&data.base_info.address).await {
    //     Ok(data) => {
    //       loserLaunch.meta = Set(Some(serde_json::to_value(data.pairs[0]).unwrap()));
    //     }
    //     Err(_e) => {
    //       eprintln!("Failed to query dexscreener");
    //     }
    //   };
    // }
    // raydium_token_launches::Entity::insert(loserLaunch)
    //   .exec(&db)
    //   .await
    //   .map_err(|e| e.to_string());

    // notify users that want launches
  } else if pool_sol_liquidity >= mid_limit && pool_sol_liquidity < normal_limit {
    println!("Liquidity is good.");
    sleep(Duration::from_secs(120)).await;

    // Replace with your saving logic
    // Check if it's a pump.fun if yes buy
  } else if pool_sol_liquidity >= normal_limit && pool_sol_liquidity < pro_limit {
    println!("Liquidity is between the normal limit and pro limit.");
  } else if pool_sol_liquidity >= pro_limit {
    println!("Liquidity is crazy");
    sleep(Duration::from_secs(240)).await;
    // attempt buy and send notification
  }

  // let is_boosted_token = /* Your logic to determine if the token is boosted */;
}