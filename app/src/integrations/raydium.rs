use std::collections::HashMap;

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

use crate::utils::cache::set_memcache_hashmap;

pub struct RaydiumPriceFetcher {
  client: Client,
}

impl RaydiumPriceFetcher {
  pub fn new() -> Self {
    RaydiumPriceFetcher {
      client: Client::new(),
    }
  }

  pub async fn get_token_price_list(&self) -> Result<HashMap<String, f64>> {
    let url =
      "https://api.raydium.io/v2/main/price?tokens=So11111111111111111111111111111111111111112";

    // Fetch token price
    let response = self
      .client
      .get(&*url)
      .send()
      .await
      .context("Failed to send request to get token prices")?;
    let json: Value = response
      .json()
      .await
      .context("Failed to parse token prices response as JSON")?;

    let price_map = self.parse_price_response(json)?;

    set_memcache_hashmap("raydium_price".to_owned(), price_map.clone(), Some(5));

    Ok(price_map)
  }

  fn parse_price_response(&self, json: Value) -> Result<HashMap<String, f64>> {
    let mut price_map: HashMap<String, f64> = HashMap::new();

    // Assuming the JSON is an object
    if let Value::Object(obj) = json {
      for (key, value) in obj {
        if let Some(num) = value.as_f64() {
          price_map.insert(key, num);
        }
      }
    }

    Ok(price_map)
  }

  pub async fn get_token_price_in_sol(&self, token_mint_address: &str) -> Result<f64> {
    let price_map = self.get_token_price_list().await?;

    let price_usd = price_map
      .get(token_mint_address)
      .context("Failed to get token price in USD from cached values")?;

    let sol_price_usd = price_map
      .get("So11111111111111111111111111111111111111112")
      .context("Failed to get SOL price in USD from cached values")?;

    // Calculate token price in SOL
    let price_in_sol = price_usd / sol_price_usd;

    Ok(price_in_sol)
  }

  pub async fn get_token_price_in_usd(&self, token_mint_address: &str) -> Result<f64> {
    // Fetch token price list to ensure prices are updated in cache
    let price_map = self.get_token_price_list().await?;

    // Extract the price in USD for the token
    let price_usd = price_map
      .get(token_mint_address)
      .context("Failed to get token price in USD from cached values")?;

    Ok(*price_usd)
  }
}
