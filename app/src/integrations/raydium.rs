pub async fn get_token_price_in_sol(token_mint_address: &str) -> Result<f64> {
  let url = format!(
    "https://api.raydium.io/v2/main/price?tokens={}",
    token_mint_address
  );

  let response = reqwest::get(&url).await?;
  let json: Value = response.json().await?;

  // Extract the price in USD
  let price_usd = json[token_mint_address]["price"]
    .as_f64()
    .ok_or_else(|| anyhow::anyhow!("Failed to get token price"))?;

  // Get SOL price in USD
  let sol_url =
    "https://api.raydium.io/v2/main/price?tokens=So11111111111111111111111111111111111111112";
  let sol_response = reqwest::get(sol_url).await?;
  let sol_json: Value = sol_response.json().await?;
  let sol_price_usd = sol_json["So11111111111111111111111111111111111111112"]["price"]
    .as_f64()
    .ok_or_else(|| anyhow::anyhow!("Failed to get SOL price"))?;

  // Calculate token price in SOL
  let price_in_sol = price_usd / sol_price_usd;

  Ok(price_in_sol)
}

pub async fn get_token_price_in_usd(token_mint_address: &str) -> Result<f64> {
  let url = format!(
    "https://api.raydium.io/v2/main/price?tokens={}",
    token_mint_address
  );

  let response = reqwest::get(&url).await?;
  let json: Value = response.json().await?;

  // Extract the price in USD
  let price_usd = json[token_mint_address]["price"]
    .as_f64()
    .ok_or_else(|| anyhow::anyhow!("Failed to get token price"))?;

  Ok(price_in_usd)
}
