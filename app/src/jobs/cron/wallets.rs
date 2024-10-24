use crate::{db, utils::wallets::solana::register_wallet_tokens};
use entity::wallets;
use sea_orm::EntityTrait;

pub async fn update_wallet_token_list() -> Result<(), Box<dyn std::error::Error>> {
  let db = db::connect_db()
    .await
    .expect("Failed to connect to the database");

  let wallets = wallets::Entity::find()
    .all(&db)
    .await
    .map_err(|e| e.to_string())?;

  for wallet in wallets {
    tokio::spawn(async move {
      let _ = register_wallet_tokens(&wallet.address, wallet.user_id).await;
    });
  }
  Ok(())
}
