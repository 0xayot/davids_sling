use anyhow::{anyhow, Context, Result};
use entity::{users, wallets};
use sea_orm::{DatabaseConnection, EntityTrait};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
  commitment_config::CommitmentConfig,
  signature::{Keypair, Signature},
  transaction::{Transaction, VersionedTransaction},
};
use std::env;
use std::time::Duration;

use crate::utils::{
  encryption::{decrypt_private_key, EncryptPKDetails},
  wallets::solana::keypair_from_private_key,
};

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 1000;

/// Execute a Raydium swap transaction
pub async fn execute_raydium_swap_tx(
  unsigned_tx: VersionedTransaction,
  keypair: &Keypair,
) -> Result<Signature> {
  let rpc_url =
    env::var("SOLANA_RPC_URL").context("Failed to retrieve SOLANA_RPC_URL from environment")?;
  let client = RpcClient::new_with_timeout(rpc_url, Duration::from_secs(30));

  let recent_blockhash = client.get_latest_blockhash()?;

  // Convert VersionedMessage to Message
  let message = match unsigned_tx.message {
    solana_sdk::message::VersionedMessage::Legacy(message) => message,
    _ => return Err(anyhow!("Unsupported message version")),
  };

  // Create a new Transaction from the Message
  let mut tx = Transaction::new_unsigned(message);

  // Sign the transaction with the recent blockhash

  tx.sign(&[keypair], recent_blockhash.to_owned());

  // Attempt to send transaction with retries
  let mut retries = 0;
  let mut last_error = None;

  while retries < MAX_RETRIES {
    match client.send_transaction(&tx) {
      Ok(signature) => {
        println!(
          "Transaction sent successfully with signature: {}",
          signature
        );
        return Ok(signature);
      }
      Err(err) => {
        println!(
          "Failed to send transaction (attempt {}): {}",
          retries + 1,
          err
        );
        last_error = Some(err);
        retries += 1;
        if retries < MAX_RETRIES {
          tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
        }
      }
    }
  }

  Err(anyhow!(
    "Failed to send transaction after {} attempts: {}",
    MAX_RETRIES,
    last_error.map_or_else(|| "Unknown error".to_string(), |e| e.to_string())
  ))
}

pub async fn confirm_executed_swap_tx(signature: &Signature) -> Result<bool> {
  let rpc_url =
    env::var("SOLANA_RPC_URL").context("Failed to retrieve SOLANA_RPC_URL from environment")?;

  let client = RpcClient::new_with_timeout(rpc_url, Duration::from_secs(60));

  // Get latest blockhash with retry
  let latest_blockhash = retry_operation(|| client.get_latest_blockhash())
    .await
    .context("Failed to get latest blockhash")?;

  // Confirm the transaction
  client
    .confirm_transaction_with_spinner(signature, &latest_blockhash, CommitmentConfig::finalized())
    .context("Failed to confirm transaction")?;

  Ok(true)
}

/// Result structure for swap transactions
#[derive(Debug)]
pub struct SwapTxResult {
  pub transaction_hash: String,
  pub success: bool,
}

/// Execute a swap transaction for a specific user
pub async fn execute_user_swap_tx(
  user_id: i32,
  wallet_id: i32,
  db: DatabaseConnection,
  unsigned_tx: VersionedTransaction,
) -> Result<SwapTxResult> {
  // Validate user existence
  let user = users::Entity::find_by_id(user_id)
    .one(&db)
    .await
    .context("Database error while fetching user")?
    .ok_or_else(|| anyhow!("User not found: {}", user_id))?;

  // Validate wallet existence
  let wallet = wallets::Entity::find_by_id(wallet_id)
    .one(&db)
    .await
    .context("Database error while fetching wallet")?
    .ok_or_else(|| anyhow!("Wallet not found: {}", wallet_id))?;

  // Prepare wallet details for decryption
  let encrypted_wallet_details = EncryptPKDetails {
    salt: wallet.salt,
    secret_key: wallet.secret_key,
    encrypted_private_key: wallet.encrypted_private_key,
  };

  // Decrypt private key and create keypair
  let decrypted_pk =
    decrypt_private_key(&encrypted_wallet_details).context("Failed to decrypt private key")?;

  let keypair =
    keypair_from_private_key(&decrypted_pk).context("Failed to create Keypair from private key")?;

  // Execute the swap transaction
  let signature = execute_raydium_swap_tx(unsigned_tx, &keypair)
    .await
    .context("Failed to execute swap transaction")?;

  // Confirm the transaction
  let is_confirmed = confirm_executed_swap_tx(&signature)
    .await
    .context("Failed to confirm transaction")?;

  Ok(SwapTxResult {
    transaction_hash: signature.to_string(),
    success: is_confirmed,
  })
}

/// Helper function to retry operations
async fn retry_operation<F, T, E>(operation: F) -> Result<T>
where
  F: Fn() -> std::result::Result<T, E>,
  E: std::error::Error + Send + Sync + 'static,
{
  let mut retries = 0;
  let mut last_error = None;

  while retries < MAX_RETRIES {
    match operation() {
      Ok(result) => return Ok(result),
      Err(err) => {
        println!("Operation failed (attempt {}): {}", retries + 1, err);
        last_error = Some(err);
        retries += 1;
        if retries < MAX_RETRIES {
          tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
        }
      }
    }
  }

  Err(anyhow!(
    "Operation failed after {} attempts: {}",
    MAX_RETRIES,
    last_error.unwrap()
  ))
}
