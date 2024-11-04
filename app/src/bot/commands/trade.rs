#![allow(deprecated)]

use crate::{
  integrations::raydium::RaydiumPriceFetcher,
  utils::{swap::solana::execute_user_swap_txs, wallets::solana::get_wallet_sol_balance},
};
use ::entity::*;
use anyhow::{anyhow, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use teloxide::prelude::*;

pub async fn handle_buy_token(
  bot: Bot,
  msg: Message,
  db: &DatabaseConnection,
  ca: String,
  size: String,
) -> Result<Message> {
  let size_f64 = size
    .parse::<f64>()
    .map_err(|_| anyhow!("Invalid size format. Please provide a valid number"))?;

  let tg_user = msg
    .from()
    .ok_or_else(|| anyhow!("No Telegram user found"))?;

  // Find authorized user
  let authorized_user = users::Entity::find()
    .filter(users::Column::TgId.eq(&tg_user.id.to_string()))
    .one(db)
    .await?
    .ok_or_else(|| anyhow!("User not found in database"))?;

  // Find associated wallet
  let wallet = wallets::Entity::find()
    .filter(wallets::Column::UserId.eq(authorized_user.id))
    .filter(wallets::Column::Chain.eq("solana"))
    .one(db)
    .await?
    .ok_or_else(|| anyhow!("No Solana wallet found for user"))?;

  let wallet_sol_balance = get_wallet_sol_balance(&wallet.address)
    .await
    .map_err(|e| anyhow!("Failed to get wallet balance: {}", e))?;

  if wallet_sol_balance < size_f64 {
    return bot
      .send_message(
        msg.chat.id,
        format!(
          "Insufficient SOL balance. You have {} SOL but need {} SOL",
          wallet_sol_balance, size
        ),
      )
      .await
      .map_err(|e| anyhow!("Failed to send message: {}", e));
  }

  let raydium_client = RaydiumPriceFetcher::new();
  let quote = raydium_client
    .get_swap_quote(
      "So11111111111111111111111111111111111111112",
      &ca,
      &(size_f64 * 1_000_000_000.0).to_string(),
      "50",
    )
    .await
    .map_err(|e| anyhow!("Failed to get a quote: {}", e))?;

  let swap_tx = raydium_client
    .get_swap_tx(
      &wallet.address,
      quote,
      "So11111111111111111111111111111111111111112",
      &ca,
      &wallet.address,
    )
    .await
    .map_err(|e| anyhow!("Failed to get a tx: {}", e))?;

  let swap_result = execute_user_swap_txs(authorized_user.id, wallet.id, db.clone(), swap_tx).await;

  match swap_result {
    Ok(attempt) => {
      let transaction = onchain_transactions::ActiveModel {
        user_id: Set(authorized_user.id),
        wallet_id: Set(wallet.id),
        transaction_hash: Set(Some(attempt.transaction_hash.clone())),
        chain: Set("solana".to_string()),
        source: Set(Some("raydium".to_string())),
        status: Set(Some(
          if attempt.success {
            "confirmed"
          } else {
            "submitted"
          }
          .to_string(),
        )),
        r#type: Set(Some("swap".to_string())),
        value_native: Set(Some(0.0)),
        value_usd: Set(Some(0.0)),
        from_token: Set(Some(ca.clone())),
        to_token: Set(Some(
          "So11111111111111111111111111111111111111112".to_string(),
        )),
        ..Default::default()
      };

      if let Err(e) = onchain_transactions::Entity::insert(transaction)
        .exec(db)
        .await
      {
        eprintln!("Failed to record transaction: {}", e);
      }

      let status = if attempt.success {
        "confirmed"
      } else {
        "submitted"
      };
      bot
        .send_message(
          msg.chat.id,
          format!(
            "Swap {} ✅\nTransaction Hash: {}\nStatus: {}\nAmount: {} SOL",
            status, attempt.transaction_hash, status, size_f64
          ),
        )
        .await
        .map_err(|e| anyhow!("Failed to send success message: {}", e))
    }
    Err(e) => {
      eprintln!("Error executing swap: {}", e);
      bot
        .send_message(msg.chat.id, format!("Failed to execute swap: {}", e))
        .await
        .map_err(|e| anyhow!("Failed to send error message: {}", e))
    }
  }
}
