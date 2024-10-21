pub async fn update_wallet_token_list(
  db: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
  let wallets = wallets::Entity::find()
    .all(db)
    .await
    .map_err(|e| e.to_string())?;

  for wallet in wallets {
    tokio::spawn(async {
      let _ = register_wallet_tokens(&wallet.address).await;
    })
  }
  Ok(())
}
