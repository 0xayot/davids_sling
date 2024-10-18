#![allow(dead_code)]
use anyhow::Result;

use crate::{integrations::raydium::RaydiumPriceFetcher, utils::cache::get_memcache_hash};

pub async fn fetch_token_price(ticker: &str) -> Result<f64> {
  // Attempt to get the cached price
  match get_memcache_hash("raydium_price") {
    Some(map) => {
      if let Some(&price) = map.get(ticker) {
        return Ok(price); // Return the cached price
      }
      // If the ticker is not found, continue to fetch the price
    }
    None => {
      // If there is no cached value, fetch it
    }
  }

  // add fail over to redis figure

  // api figure
  let raydium_client = RaydiumPriceFetcher::new();

  let token_price_in_usd = raydium_client.get_token_price_in_usd(ticker).await;

  match token_price_in_usd {
    Ok(price) => Ok(price),
    Err(err) => {
      eprintln!("Error fetching token price: {:?}", err);
      Err(err.into())
    }
  }
}

pub async fn fetch_token_sol_price(ticker: &str) -> Result<f64> {
  match get_memcache_hash("raydium_price") {
    Some(map) => {
      if let Some(&price) = map.get(ticker) {
        let sol_price = map
          .get("So11111111111111111111111111111111111111112")
          .unwrap();
        return Ok(price / sol_price);
      }
    }
    None => {}
  }

  let raydium_client = RaydiumPriceFetcher::new();

  let token_price_in_usd = raydium_client.get_token_price_in_sol(ticker).await;

  match token_price_in_usd {
    Ok(price) => Ok(price),
    Err(err) => {
      eprintln!("Error fetching token price: {:?}", err);
      Err(err.into())
    }
  }
}
