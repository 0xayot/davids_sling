use anyhow::Result;

use crate::{integrations::raydium::RaydiumPriceFetcher, utils::cache::get_memcache_hash};

pub async fn fetch_token_price(ticker: &str) -> Result<f64> {
  // Attempt to get the cached price
  match get_memcache_hash("raydium_price") {
    Some(map) => {
      if let Some(&price) = map.get(ticker) {
        println!("price {}", price);
        return Ok(price); // Return the cached price
      }
      // If the ticker is not found, continue to fetch the price
    }
    None => {
      // If there is no cached value, fetch it
    }
  }

  // add fail over to redis figure
  let raydium_client = RaydiumPriceFetcher::new();

  let token_price_in_usd = raydium_client.get_token_price_in_usd(ticker).await;

  match token_price_in_usd {
    Ok(price) => Ok(price), // Return the fetched price
    Err(err) => {
      eprintln!("Error fetching token price: {:?}", err);
      Err(err.into()) // Return the error
    }
  }
}
