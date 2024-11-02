use crate::{
  db,
  integrations::{dexscreener, raydium::RaydiumPriceFetcher},
  utils::cache,
};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use entity::{raydium_token_launches, token_prices as prices, tokens};
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
    let db_clone = db.clone();
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

pub async fn refresh_sol_tokens_to_watch() -> () {
  let db = db::connect_db()
    .await
    .expect("Failed to connect to the database");
  let tokens = tokens::Entity::find()
    .filter(tokens::Column::Chain.eq("solana"))
    .all(&db)
    .await
    .map_err(|e| format!("Database error: {}", e))
    .unwrap();

  let mut token_addresses = String::from("So11111111111111111111111111111111111111112");

  for token in tokens {
    let contract_address = token.contract_address;
    if token_addresses != "So11111111111111111111111111111111111111112" {
      token_addresses.push(',');
      token_addresses.push_str(&contract_address);
    }
  }
  cache::set_memcache_string("token_addresses".to_owned(), token_addresses, Some(5 * 3));
}

use std::error::Error;

pub async fn track_launch_event_token_prices() -> Result<(), Box<dyn Error>> {
  let db = db::connect_db()
    .await
    .expect("Failed to connect to the database");
  let today = Utc::now().date_naive(); // This gives you a Date<Utc>

  // Create NaiveDate for today without time zone info
  let naive_today = NaiveDate::from_ymd_opt(today.year(), today.month(), today.day())
    .expect("Failed to create NaiveDate");

  // Define the start and end of the day
  let start_of_day = naive_today.and_hms_opt(0, 0, 0);

  let launches = raydium_token_launches::Entity::find()
    .filter(raydium_token_launches::Column::Evaluation.eq("track"))
    .filter(raydium_token_launches::Column::CreatedAt.gt(start_of_day))
    // .filter(raydium_token_launches::Column::CreatedAt.(end_of_day))
    .all(&db)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

  println!("\n track launch \n {:?}", launches.len());

  for launch in launches {
    match dexscreener::fetch_token_data(&launch.contract_address).await {
      Ok(data) => {
        let db_clone = db.clone();

        if let Some(pair) = data.pairs.get(0) {
          let price_usd = pair.priceUsd.parse::<f32>().unwrap_or(0.0);
          let price_sol = pair.priceNative.parse::<f32>().unwrap_or(0.0);
          let liquidity = pair.liquidity.usd;

          let current_price = prices::ActiveModel {
            contract_address: Set(launch.contract_address.clone()),
            chain: Set("solana".to_string()),
            price: Set(Some(price_usd)),
            price_native: Set(Some(price_sol)),
            ..Default::default()
          };

          if let Err(e) = prices::Entity::insert(current_price).exec(&db_clone).await {
            eprintln!(
              "Failed to insert price for {}: {}",
              launch.contract_address, e
            );
          }

          let rug_liquidity = liquidity * 0.8;
          // If the token loses more than 70% of its launch liquidity, it has rugged
          if liquidity <= rug_liquidity {
            let created_at: DateTime<Utc> = launch.created_at.and_utc();

            let lifespan = (Utc::now() - created_at).num_seconds() as i32;

            let lifespan_update_model = raydium_token_launches::ActiveModel {
              id: Set(launch.id),
              lifespan: Set(Some(lifespan)),
              evaluation: Set(Some("rugged".to_string())),
              ..Default::default()
            };

            if let Err(e) = raydium_token_launches::Entity::update(lifespan_update_model)
              .exec(&db_clone)
              .await
            {
              eprintln!(
                "Failed to update lifespan and evaluation for {}: {}",
                launch.contract_address, e
              );
            }
          } else {
            // Update the meta field of the launch if it's empty with a JSON of the pair
            if launch.meta.is_none() {
              let meta_update_model = raydium_token_launches::ActiveModel {
                id: Set(launch.id),
                meta: Set(Some(serde_json::to_value(pair).unwrap())),
                ..Default::default()
              };

              if let Err(e) = raydium_token_launches::Entity::update(meta_update_model)
                .exec(&db_clone)
                .await
              {
                eprintln!(
                  "Failed to update meta for {}: {}",
                  launch.contract_address, e
                );
              }
            }
          }
        } else {
          eprintln!(
            "No pairs found for contract address: {}",
            launch.contract_address
          );
        }
      }
      Err(e) => {
        eprintln!(
          "Failed to fetch token data for {}: {}",
          launch.contract_address, e
        );
      }
    }
  }

  Ok(())
}
