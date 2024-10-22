use crate::{db, integrations::raydium::RaydiumPriceFetcher};
use entity::{token_prices as prices, tokens};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

pub async fn refresh_sol_token_prices() -> Result<(), Box<dyn std::error::Error>> {
  let db = db::connect_db()
    .await
    .expect("Failed to connect to the database");

  let tokens = tokens::Entity::find()
    .filter(tokens::Column::Chain.eq("solana"))
    .all(&db)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

  let mut tasks = vec![];

  for token in tokens {
    let db_clone = db.clone(); // Clone the DB connection for use in the task
    let contract_address = token.contract_address.clone();

    let task = tokio::spawn(async move {
      let raydium_client = RaydiumPriceFetcher::new();
      let token_price_sol = raydium_client
        .get_token_price_in_sol(&contract_address)
        .await
        .unwrap_or(0.0);

      match raydium_client
        .get_token_price_in_usd(&contract_address)
        .await
      {
        Ok(price) => {
          let current_price = prices::ActiveModel {
            contract_address: Set(contract_address),
            chain: Set("solana".to_string()),
            price: Set(Some(price as f32)),
            price_native: Set(Some(token_price_sol as f32)),
            ..Default::default()
          };

          if let Err(e) = prices::Entity::insert(current_price).exec(&db_clone).await {
            eprintln!(
              "Failed to insert price for {}: {}",
              token.contract_address, e
            );
          }
        }
        Err(err) => {
          eprintln!(
            "Failed to fetch price in USD for {}: {}",
            contract_address, err
          );
        }
      }
    });

    tasks.push(task);
  }

  futures::future::join_all(tasks).await;

  Ok(())
}
