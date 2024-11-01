use serde::{Deserialize, Serialize};
use std::env;
use tokio::time::{sleep, Duration};

use crate::integrations::dexscreener;
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

  // Wait for 5 seconds before calling dexscreener
  sleep(Duration::from_secs(5)).await;

  let mut token_info_from_dexscreener =
    match dexscreener::fetch_token_data(&data.base_info.address).await {
      Ok(data) => Some(data),
      Err(_e) => None,
    };

  if pool_sol_liquidity <= lower_limit {
    println!("loser liquidity.");
    if let Some(info) = token_info_from_dexscreener {
    } else {
      sleep(Duration::from_secs(120)).await;
      token_info_from_dexscreener =
        match dexscreener::fetch_token_data(&data.base_info.address).await {
          Ok(data) => Some(data),
          Err(_e) => None,
        };
      println!("Failed to fetch token data.");
    }
  } else if pool_sol_liquidity > lower_limit && pool_sol_liquidity < mid_limit {
    println!("Liquidity is meh!");
    sleep(Duration::from_secs(120)).await; // Replace with your saving logic
  } else if pool_sol_liquidity >= mid_limit && pool_sol_liquidity < normal_limit {
    println!("Liquidity is good.");
    sleep(Duration::from_secs(120)).await; // Replace with your saving logic
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
