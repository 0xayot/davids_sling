use std::env;

use serde::{Deserialize, Serialize};

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
  println!("Liquidity is between the mid limit and normal limit.");

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

  // Differentiate actions based on the value of the liquidity
  if pool_sol_liquidity <= lower_limit {
    // delay 2 mins  and save the token
  } else if pool_sol_liquidity > lower_limit && pool_sol_liquidity < mid_limit {

    // delay 2 mins  and save the token
  } else if pool_sol_liquidity >= mid_limit && pool_sol_liquidity < normal_limit {
    // delay 2 mins  and save the token
    // check if its a pump.fun if yes buy
  } else if pool_sol_liquidity >= normal_limit && pool_sol_liquidity < pro_limit {
    // delay 2 mins  and save the token
    // check if its a pump.fun if yes buy
  } else if pool_sol_liquidity >= pro_limit {
    // delay 2 mins  and save the token
    // check if the liquidit is above pro limit in usd
    // check if its a pump.fun if yes buy
  }

  let token_info_from_dexscreener =
    match dexscreener::fetch_token_data(&data.base_info.address).await {
      Ok(data) => Some(data),
      Err(_e) => None,
    };

  // Use token_info_from_dexscreener as an Option<ResponseData>
  if let Some(info) = token_info_from_dexscreener {
    // Process the retrieved token info
  } else {
    // Handle the case where fetching token data failed
  }

  // let is_boosted_token = /* Your logic to determine if the token is boosted */;
}
