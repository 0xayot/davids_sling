use crate::integrations::raydium::RaydiumPriceFetcher;

pub async fn refresh_sol_token_prices() -> Result<(), Box<dyn std::error::Error>> {
  let raydium_client = RaydiumPriceFetcher::new();
  match raydium_client.get_token_price_list().await {
    Ok(_) => {
      println!("Updated token prices from Raydium");
      Ok(())
    }
    Err(err) => {
      eprintln!("Error fetching token price: {:?}", err);
      Err(err.into())
    }
  }
}

pub async fn refresh_sol_token_price(
  db: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
  let tokens = tokens::Entity::find()
    .filter(tokens::Column::Chain.eq("solana".to_string()))
    .all(db)
    .await?;

  for token in tokens {
    tokio::spawn(async {
      let raydium_client = RaydiumPriceFetcher::new();
      let token_price_sol = raydium_client
        .get_token_price_in_sol(token.contract_address)
        .await
        .unwrap_or(0.0);
      match raydium_client
        .get_token_price_in_usd(token.contract_address)
        .await
      {
        Ok(price) => {
          let current_price = prices::ActiveModel {
            contract_address: Set(token.contract_address),
            chain: Set("solana".to_string()),
            name: None,
            price: Set(price as f32),
            price_native: Set(Some(token_price_sol))..Default::default(),
          };
          prices::Entity::insert(current_price)
            .exec(&db)
            .await
            .map_err(|e| e.to_string())?;
          continue;
        }
        Err(err) => continue,
      }
    })
  }
}
