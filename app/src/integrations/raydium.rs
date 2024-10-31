#![allow(dead_code)]
#![allow(non_snake_case)]
use base64::decode;
use std::{collections::HashMap, env};

use anyhow::{Context, Result};
use bincode::deserialize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::transaction::VersionedTransaction;

use crate::utils::cache::{
  self, get_memcache_string_hash, set_memcache_hashmap, set_memcache_string_hashmap,
};

pub struct RaydiumPriceFetcher {
  client: Client,
}

#[derive(Debug, Deserialize)]
struct RaydiumMintPriceResponse {
  id: String,
  success: bool,
  data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RaydiumGasPrices {
  vh: String,
  h: String,
  m: String,
}

impl RaydiumGasPrices {
  pub fn to_hashmap(&self) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("vh".to_string(), self.vh.clone());
    map.insert("h".to_string(), self.h.clone());
    map.insert("m".to_string(), self.m.clone());
    map
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RaydiumSwapResponse {
  pub id: String,
  pub success: bool,
  pub version: String,
  pub data: SwapData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwapData {
  #[serde(rename = "swapType")]
  pub swap_type: String,
  #[serde(rename = "inputMint")]
  pub input_mint: String,
  #[serde(rename = "inputAmount")]
  pub input_amount: String,
  #[serde(rename = "outputMint")]
  pub output_mint: String,
  #[serde(rename = "outputAmount")]
  pub output_amount: String,
  #[serde(rename = "otherAmountThreshold")]
  pub other_amount_threshold: String,
  #[serde(rename = "slippageBps")]
  pub slippage_bps: u32,
  #[serde(rename = "priceImpactPct")]
  pub price_impact_pct: f64,
  #[serde(rename = "routePlan")]
  pub route_plan: Vec<RoutePlan>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoutePlan {
  #[serde(rename = "poolId")]
  pub pool_id: String,
  #[serde(rename = "inputMint")]
  pub input_mint: String,
  #[serde(rename = "outputMint")]
  pub output_mint: String,
  #[serde(rename = "feeMint")]
  pub fee_mint: String,
  #[serde(rename = "feeRate")]
  pub fee_rate: u32,
  #[serde(rename = "feeAmount")]
  pub fee_amount: String,
  #[serde(rename = "remainingAccounts")]
  pub remaining_accounts: Vec<String>,
  #[serde(rename = "lastPoolPriceX64")]
  pub last_pool_price_x64: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SwapRequest {
  computeUnitPriceMicroLamports: String,
  swapResponse: Value,
  txVersion: String,
  wallet: String,
  wrapSol: bool,
  unwrapSol: bool,
  inputAccount: Option<String>,
  outputAccount: Option<String>,
}

#[derive(Deserialize)]
struct SwapResponse {
  data: Vec<TransactionData>,
  id: String,
  success: bool,
  version: String,
}

#[derive(Deserialize)]
struct TransactionData {
  transaction: String,
}

impl RaydiumPriceFetcher {
  pub fn new() -> Self {
    RaydiumPriceFetcher {
      client: Client::new(),
    }
  }

  pub async fn get_token_price_list(
    &self,
    addresses: Option<String>,
  ) -> Result<HashMap<String, f64>> {
    let token_list = if let Some(addresses) = addresses {
      addresses
    } else {
      // Retrieve from cache
      cache::get_memcache_string("token_addresses")
        .unwrap_or_else(|| String::from("So11111111111111111111111111111111111111112"))
    };

    let url = format!("https://api-v3.raydium.io/mint/price?mints={}", token_list);

    let response = self
      .client
      .get(&url)
      .send()
      .await
      .context("Failed to send request to get token prices")?;

    let json: RaydiumMintPriceResponse = response
      .json()
      .await
      .context("Failed to parse token prices response as JSON")?;

    let price_map = self.parse_price_response(json.data)?;

    set_memcache_hashmap("raydium_price".to_owned(), price_map.clone(), Some(5));

    Ok(price_map)
  }

  pub async fn get_priority_fee(&self) -> Result<RaydiumGasPrices> {
    match get_memcache_string_hash("solana_gas_prices") {
      Some(map) => {
        let vh = map.get("vh").map(|v| v.clone());
        let h = map.get("h").map(|v| v.clone());
        let m = map.get("m").map(|v| v.clone());

        if vh.is_none() || h.is_none() || m.is_none() {
          return Err(anyhow::anyhow!("Missing required fields in gas prices"));
        }

        Ok(RaydiumGasPrices {
          vh: vh.unwrap(),
          h: h.unwrap(),
          m: m.unwrap(),
        })
      }
      None => {
        let url = "https://api-v3.raydium.io/main/auto-fee";
        let response = self
          .client
          .get(url)
          .send()
          .await
          .context("error getting gas prices")?;

        let json: Value = response
          .json()
          .await
          .context("Failed to parse swap request response as JSON")?;

        let gas_prices = json["data"]["default"].clone();
        let result = RaydiumGasPrices {
          vh: gas_prices["vh"].to_string(),
          h: gas_prices["h"].to_string(),
          m: gas_prices["m"].to_string(),
        };

        set_memcache_string_hashmap(
          "solana_gas_prices".to_owned(),
          result.to_hashmap(),
          Some(5 * 60),
        );

        Ok(result)
      }
    }
  }

  pub async fn get_swap_quote(
    &self,
    input_mint: &str,
    output_mint: &str,
    amount: &str,
    slippage: &str,
  ) -> Result<Value> {
    let url_base = env::var("RAYDIUM_SWAP_URL")
      .unwrap_or_else(|_| "https://transaction-v1.raydium.io".to_string());

    let url = format!(
      "{}/compute/swap-base-in?inputMint={}&outputMint={}&amount={}&slippageBps={}&txVersion=V0",
      url_base, input_mint, output_mint, amount, slippage
    );
    let response = self
      .client
      .get(&url)
      .send()
      .await
      .context("Failed to send a request")?;

    if !response.status().is_success() {
      return Err(anyhow::anyhow!(
        "API request failed with status: {}",
        response.status()
      ));
    }

    let json: Value = response
      .json()
      .await
      .context("Failed to parse swap request response as JSON")?;

    Ok(json)
  }

  pub async fn get_swap_tx(
    &self,
    taker_address: &str,
    swap_quote: Value,
    input_mint: &str,
    output_mint: &str,
    tokenPk: &str,
  ) -> Result<VersionedTransaction> {
    let url_base = env::var("RAYDIUM_SWAP_URL")
      .unwrap_or_else(|_| "https://transaction-v1.raydium.io".to_string());

    let url = format!("{}/transaction/swap-base-in", url_base);
    let wrap_sol = input_mint == "So11111111111111111111111111111111111111112";
    let unwrap_sol = output_mint == "So11111111111111111111111111111111111111112";

    let gas = self.get_priority_fee().await.unwrap();

    let mut request_body = SwapRequest {
      computeUnitPriceMicroLamports: gas.h,
      swapResponse: swap_quote,
      txVersion: "V0".to_string(),
      wallet: taker_address.to_string(),
      wrapSol: wrap_sol,
      unwrapSol: unwrap_sol,
      inputAccount: None,
      outputAccount: None,
    };
    if !wrap_sol {
      request_body.inputAccount = Some(tokenPk.to_string());
    };

    if !unwrap_sol {
      request_body.outputAccount = Some(tokenPk.to_string());
    };

    let response = self.client.post(url).json(&request_body).send().await?;

    // Ensure the response is successful
    response.error_for_status_ref()?;

    let swap_response: SwapResponse = response.json().await?;
    let all_tx_buf: Vec<Vec<u8>> = swap_response
      .data
      .iter()
      .map(|tx| decode(&tx.transaction))
      .collect::<std::result::Result<Vec<_>, _>>()
      .context("Failed to decode base64 transaction data")?;

    let versioned_tx: VersionedTransaction =
      deserialize(&all_tx_buf[0]).context("Failed to deserialize VersionedTransaction")?;

    println!("Deserialized transaction: {:?}", versioned_tx);

    Ok(versioned_tx)
  }

  fn parse_price_response(&self, json: HashMap<String, String>) -> Result<HashMap<String, f64>> {
    let mut price_map: HashMap<String, f64> = HashMap::new();

    for (key, value) in json {
      if let Ok(num) = value.parse::<f64>() {
        price_map.insert(key, num);
      }
    }

    Ok(price_map)
  }

  pub async fn get_token_price_in_sol(&self, token_mint_address: &str) -> Result<f64> {
    let price_map = self
      .get_token_price_list(Some(token_mint_address.to_string()))
      .await?;

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
    let price_map = self
      .get_token_price_list(Some(token_mint_address.to_string()))
      .await?;

    // Extract the price in USD for the token
    let price_usd = price_map
      .get(token_mint_address)
      .context("Failed to get token price in USD from cached values")?;

    Ok(*price_usd)
  }
}
