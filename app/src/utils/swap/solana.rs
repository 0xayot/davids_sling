#![allow(dead_code)]
use anyhow::{anyhow, Context, Result};
use entity::wallets;
use sea_orm::{DatabaseConnection, EntityTrait};
use solana_client::nonblocking::rpc_client::RpcClient as AsyncClient;

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

// deprecated
pub async fn execute_raydium_swap_tx(
  unsigned_tx: VersionedTransaction,
  keypair: &Keypair,
) -> Result<Signature> {
  let rpc_url =
    env::var("SOLANA_RPC_URL").context("Failed to retrieve SOLANA_RPC_URL from environment")?;
  let client = AsyncClient::new(rpc_url);

  let recent_blockhash = client.get_latest_blockhash().await?;

  // let message = match &unsigned_tx.message {
  //   VersionedMessage::V0(message) => {
  //     let instructions: Vec<Instruction> = message
  //       .instructions
  //       .iter()
  //       .map(|compiled_ix| {
  //         // Ensure the program_id_index is within bounds
  //         let program_id_index = compiled_ix.program_id_index as usize;
  //         if program_id_index >= message.account_keys.len() {
  //           eprintln!(
  //             "Error: program_id_index {} out of bounds for account_keys of length {}",
  //             program_id_index,
  //             message.account_keys.len()
  //           );
  //           return Err(anyhow!("Program ID index out of bounds"));
  //         }

  //         let program_id = message.account_keys[program_id_index];

  //         let accounts: Vec<AccountMeta> = compiled_ix
  //           .accounts
  //           .iter()
  //           .map(|&index| {
  //             let index_usize = index as usize;
  //             // Check index bounds for account_keys
  //             if index_usize >= message.account_keys.len() {
  //               eprintln!(
  //                 "Error: Account index {} out of bounds for account_keys of length {}",
  //                 index_usize,
  //                 message.account_keys.len()
  //               );
  //               return AccountMeta::new(Pubkey::default(), false); // Handle this appropriately
  //             }

  //             let pubkey = message.account_keys[index_usize];
  //             // Check if the account is writable and/or signer
  //             let is_signer = message.header.num_required_signatures as usize > index_usize;
  //             let is_writable = message.is_maybe_writable(index_usize);

  //             if is_writable {
  //               AccountMeta::new(pubkey, is_signer)
  //             } else {
  //               AccountMeta::new_readonly(pubkey, is_signer)
  //             }
  //           })
  //           .collect();

  //         Ok(Instruction {
  //           program_id,
  //           accounts,
  //           data: compiled_ix.data.clone(),
  //         })
  //       })
  //       .collect::<Result<Vec<Instruction>, _>>()?; // Collect errors if any

  //     Message::new(&instructions, Some(&keypair.pubkey()))
  //   }
  //   _ => return Err(anyhow!("Unsupported message version")),
  // };

  let message = match unsigned_tx.message {
    solana_sdk::message::VersionedMessage::Legacy(message) => message,
    e => {
      eprintln!("Error: Unsupported message version in transaction: {:?}", e);
      return Err(anyhow!("Unsupported message version"));
    }
  };

  // Create a new Transaction from the Message
  let mut tx = Transaction::new_unsigned(message);

  tx.sign(&[keypair], recent_blockhash.to_owned());

  // Attempt to send transaction with retries
  let mut retries = 0;
  let mut last_error = None;

  while retries < MAX_RETRIES {
    match client.send_transaction(&tx).await {
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

  let client = AsyncClient::new_with_timeout(rpc_url, Duration::from_secs(60));

  // Get latest blockhash with retry
  let latest_blockhash = client.get_latest_blockhash().await?;
  // Confirm the transaction
  client
    .confirm_transaction_with_spinner(signature, &latest_blockhash, CommitmentConfig::finalized())
    .await
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
  _user_id: i32,
  wallet_id: i32,
  db: DatabaseConnection,
  unsigned_tx: VersionedTransaction,
) -> Result<SwapTxResult> {
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

  let decrypted_pk =
    decrypt_private_key(&encrypted_wallet_details).context("Failed to decrypt private key")?;

  let keypair =
    keypair_from_private_key(&decrypted_pk).context("Failed to create Keypair from private key")?;

  // Execute the swap transaction
  let signature = match execute_raydium_swap_tx(unsigned_tx, &keypair).await {
    Ok(sig) => sig,
    Err(e) => {
      eprintln!("Failed to execute Raydium swap transaction: {}", e);
      return Err(anyhow!("Transaction execution failed: {}", e));
    }
  };

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

pub async fn execute_raydium_swap_txs(
  unsigned_txs: Vec<VersionedTransaction>,
  keypair: &Keypair,
) -> Result<Vec<Signature>> {
  let rpc_url =
    env::var("SOLANA_RPC_URL").context("Failed to retrieve SOLANA_RPC_URL from environment")?;
  let client = AsyncClient::new(rpc_url);

  let recent_blockhash = client.get_latest_blockhash().await?;

  let mut signatures = Vec::new();

  for unsigned_tx in unsigned_txs {
    let message = match unsigned_tx.message {
      solana_sdk::message::VersionedMessage::Legacy(message) => message,
      e => {
        eprintln!("Error: Unsupported message version in transaction: {:?}", e);
        return Err(anyhow!("Unsupported message version"));
      }
    };

    let mut tx = Transaction::new_unsigned(message);
    tx.sign(&[keypair], recent_blockhash.to_owned());

    // Attempt to send transaction with retries
    let mut retries = 0;
    let mut last_error = None;

    while retries < MAX_RETRIES {
      match client.send_transaction(&tx).await {
        Ok(signature) => {
          println!(
            "Transaction sent successfully with signature: {}",
            signature
          );
          signatures.push(signature);
          break; // Break out of retry loop on success
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

    if let Some(err) = last_error {
      return Err(anyhow!(
        "Failed to send transaction after {} attempts: {}",
        MAX_RETRIES,
        err
      ));
    }
  }

  Ok(signatures)
}

pub async fn execute_user_swap_txs(
  _user_id: i32,
  wallet_id: i32,
  db: DatabaseConnection,
  unsigned_txs: Vec<VersionedTransaction>,
) -> Result<SwapTxResult> {
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

  // Execute the swap transactions
  let signatures = match execute_raydium_swap_txs(unsigned_txs, &keypair).await {
    Ok(sigs) => sigs,
    Err(e) => {
      eprintln!("Failed to execute Raydium swap transactions: {}", e);
      return Err(anyhow!("Transaction execution failed: {}", e));
    }
  };

  let last_signature = signatures
    .last()
    .ok_or_else(|| anyhow!("No signatures found"))?;

  let is_confirmed = confirm_executed_swap_tx(last_signature)
    .await
    .context("Failed to confirm transaction")?;

  Ok(SwapTxResult {
    transaction_hash: last_signature.to_string(),
    success: is_confirmed,
  })
}
