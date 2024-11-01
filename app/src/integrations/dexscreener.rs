#![allow(dead_code)]
#![allow(non_snake_case)]

use anyhow::{Error, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{self, Value};

#[derive(Deserialize, Debug)]
pub struct ResponseData {
  pub schemaVersion: String,
  pub pairs: Vec<Pair>,
}

#[derive(Deserialize, Debug)]
pub struct Pair {
  pub baseToken: Token,
  pub chainId: String,
  pub dexId: String,
  pub fdv: f64,
  pub liquidity: Liquidity,
  pub marketCap: f64,
  pub pairAddress: String,
  pub pairCreatedAt: Option<u64>, // Make optional for flexibility
  pub priceChange: PriceChange,
  pub priceNative: String,
  pub priceUsd: String,
  pub quoteToken: Token,
  pub url: String,
  pub volume: Volume,
  pub boosts: Option<Boosts>, // Optional to handle cases where it might be missing
  pub info: Option<Info>,     // Optional for cases where it might be missing
}

#[derive(Deserialize, Debug)]
pub struct Token {
  pub address: String,
  pub name: String,
  pub symbol: String,
}

#[derive(Deserialize, Debug)]
pub struct Liquidity {
  pub usd: f64,
  pub base: f64,
  pub quote: f64,
}

#[derive(Deserialize, Debug)]
pub struct PriceChange {
  pub h1: f64,
  pub h24: f64,
  pub h6: f64,
  pub m5: f64,
}

#[derive(Deserialize, Debug)]
pub struct Volume {
  pub h1: f64,
  pub h24: f64,
  pub h6: f64,
  pub m5: f64,
}

#[derive(Deserialize, Debug)]
pub struct Boosts {
  pub active: u32,
}

#[derive(Deserialize, Debug)]
pub struct Info {
  pub imageUrl: Option<String>,
  pub socials: Option<Vec<Social>>,
  pub websites: Option<Vec<Website>>,
}

#[derive(Deserialize, Debug)]
pub struct Social {
  pub r#type: String, // Use r#type to avoid keyword collision
  pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct Website {
  pub label: String,
  pub url: String,
}

pub async fn fetch_token_data(token_addresses: &str) -> Result<ResponseData, Error> {
  let url = format!(
    "https://api.dexscreener.com/latest/dex/tokens/{}",
    token_addresses
  );
  let client = Client::new();

  let response = client.get(&url).send().await?;

  if !response.status().is_success() {
    let status = response.status();
    let body = response.text().await?;
    return Err(Error::msg(format!(
      "Request failed with status: {} and body: {}",
      status, body
    )));
  }

  let body = response.text().await?;

  let json_value: Value = serde_json::from_str(&body)?;

  if let Some(pairs) = json_value.get("pairs") {
    if pairs.is_null() {
      return Err(Error::msg("Pairs field is null."));
    }
  } else {
    return Err(Error::msg("Pairs field is missing."));
  }
  let data: ResponseData = serde_json::from_value(json_value).map_err(|e| {
    Error::msg(format!(
      "Failed to parse JSON response into ResponseData struct: {}. Original response was: {}",
      e, body
    ))
  })?;

  for pair in &data.pairs {
    if pair.priceChange.h1.is_nan() {
      return Err(Error::msg(format!(
        "Missing or invalid field in pair: {}. Price change h1 is NaN.",
        pair.pairAddress
      )));
    }
  }

  Ok(data)
}
