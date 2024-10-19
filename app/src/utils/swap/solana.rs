use anyhow::{Context, Result};
use entity::{users, wallets};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
  commitment_config::CommitmentConfig,
  pubkey::Pubkey,
  signature::{Keypair, Signer},
  transaction::VersionedTransaction,
};
use std::env;
use std::str::FromStr;

pub async fn execute_raydium_swap_tx(
  unsigned_tx: VersionedTransaction,
  keypair: &Keypair,
) -> Result<String> {
  let rpc_url =
    env::var("SOLANA_RPC_URL").context("Failed to retrieve SOLANA_RPC_URL from environment")?;

  // Create an RPC client
  let client = RpcClient::new(rpc_url);

  // Sign the transaction
  let mut signed_tx = unsigned_tx;
  signed_tx.sign(&[keypair], signed_tx.message.recent_blockhash);

  // Send the transaction
  let signature = client
    .send_transaction(&signed_tx)
    .context("Failed to send transaction")?;

  println!("Transaction sent with signature: {}", signature);
  Ok(signature.to_string())
}

pub async fn confirm_executed_swap_tx(signature: &str) -> Result<bool> {
  let rpc_url =
    env::var("SOLANA_RPC_URL").context("Failed to retrieve SOLANA_RPC_URL from environment")?;

  let client = RpcClient::new(rpc_url);
  let confirmation = client
    .confirm_transaction_with_spinner(
      signature,
      &client.get_latest_blockhash()?,
      CommitmentConfig::finalized(),
    )
    .context("Failed to confirm transaction")?;

  if confirmation {
    println!("Transaction confirmed and finalized!");
    Ok(true)
  } else {
    Err(anyhow::anyhow!("Transaction was not confirmed"))
  }
}

#[derive(Debug)]
pub struct SwapTxResult {
  pub transaction_hash: String,
  pub success: bool,
}

pub async fn execute_user_swap_tx(
  user_id: i32,
  wallet_id: i32,
  db: DatabaseConnection,
  unsigned_tx: VersionedTransaction,
) -> Result<SwapTxResult, String> {
  let user = users::Entity::find_by_id(user_id)
    .one(db)
    .await
    .map_err(|e| e.to_string())?;

  if user.is_none() {
    return Err("User doesn't exist".to_string());
  }

  let wallet_record = wallets::Entity::find_by_id(wallet_id)
    .one(db)
    .await
    .map_err(|e| e.to_string())?;

  let wallet = match wallet_record {
    Some(w) => w,
    None => return Err("Wallet doesn't exist".to_string()),
  };

  let encrypted_wallet_details = EncryptPKDetails {
    salt: wallet.salt,
    secret_key: wallet.secret_key,
    encrypt_private_key: wallet.encrypted_private_key,
  };

  let decrypted_pk = decrypt_private_key(encrypted_wallet_details)
    .map_err(|e| format!("Error decrypting private key: {}", e))?;

  let keypair = keypair_from_base58_string(decrypted_pk);

  let tx_completion_attempt = execute_raydium_swap_tx(unsigned_tx, &keypair)
    .await
    .map_err(|e| format!("Error executing swap transaction: {}", e))?;

  let is_confirmed = confirm_executed_swap_tx(&tx_completion_attempt)
    .await
    .map_err(|e| format!("Error confirming transaction: {}", e))?;

  let response = SwapTxResult {
    transaction_hash: tx_completion_attempt,
    success: is_confirmed,
  };

  Ok(response)
}
