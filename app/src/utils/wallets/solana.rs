#![allow(dead_code)]
use ::entity::prelude::*;
use entity::{tokens, trade_orders, wallets};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use solana_account_decoder::UiAccountData;
use solana_sdk::{
  bs58,
  signer::{keypair::Keypair, Signer},
};

use solana_client::{rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use solana_sdk::pubkey::Pubkey;

use std::{env, str::FromStr};

use solana_client::nonblocking::rpc_client::RpcClient as AsyncClient;

use crate::{
  db,
  utils::{encryption::encrypt_private_key, price::solana::fetch_token_price},
};

#[derive(Debug)]
pub struct SolanaKeyPair {
  pub public_key: String,
  pub private_key: String,
  pub salt: Option<String>,
  pub secret_key: Option<String>,
  pub encrypted_private_key: Option<String>,
}

pub fn generate_wallet() -> SolanaKeyPair {
  let wallet = Keypair::new();

  let public_key = wallet.pubkey();

  // Get the private key as bytes
  let private_key = wallet.to_bytes();

  let private_key_bs58 = bs58::encode(&private_key).into_string();

  let encrypted_details = encrypt_private_key(&private_key_bs58).unwrap();

  let res = SolanaKeyPair {
    private_key: private_key_bs58,
    public_key: public_key.to_string(),
    salt: Some(encrypted_details.salt),
    secret_key: Some(encrypted_details.secret_key),
    encrypted_private_key: Some(encrypted_details.encrypted_private_key),
  };

  return res;
}

pub fn recover_wallet_from_private_key(private_key: &str) -> Option<SolanaKeyPair> {
  let decoded = bs58::decode(private_key).into_vec().ok()?;

  let keypair = Keypair::from_bytes(&decoded).ok()?;

  let encrypted_details = encrypt_private_key(&private_key.to_string()).unwrap();

  let res = SolanaKeyPair {
    private_key: private_key.to_string(),
    public_key: keypair.pubkey().to_string(),
    salt: Some(encrypted_details.salt),
    secret_key: Some(encrypted_details.secret_key),
    encrypted_private_key: Some(encrypted_details.encrypted_private_key),
  };

  Some(res)
}

#[derive(Debug)]
pub struct TokenInfo {
  mint_address: String,
  token_balance: f64,
  decimals: u8,
  mint_public_key: String,
}

#[derive(Debug)]
pub struct SlingTokenInfo {
  pub mint_address: String,
  pub token_usd_balance: f64,
  pub token_balance: f64,
  pub price: f64,
  pub decimals: u8,
  pub public_key: String,
}

pub async fn get_spl_tokens_in_wallet(
  address: &str,
) -> Result<Vec<TokenInfo>, Box<dyn std::error::Error + Send>> {
  // Connect to Solana network
  let rpc_url =
    env::var("SOLANA_RPC_URL").map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
  let client = AsyncClient::new(rpc_url);

  // Create a Pubkey from the address string
  let pubkey =
    Pubkey::from_str(address).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

  // Fetch all token accounts owned by this address
  let token_accounts = client
    .get_token_accounts_by_owner(&pubkey, TokenAccountsFilter::ProgramId(spl_token::id()))
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

  // Process and return the token information
  let mut tokens = Vec::new();
  for account in token_accounts {
    // Extract the mint address, amount, and decimals from the account data
    if let UiAccountData::Json(parsed_account) = account.account.data {
      // Access the parsed token account info
      let info = &parsed_account.parsed["info"];

      let mint = info["mint"].as_str().unwrap_or_default().to_string();
      let amount_str = info["tokenAmount"]["amount"].as_str().unwrap_or("0");
      let decimals = info["tokenAmount"]["decimals"].as_u64().unwrap_or(0) as u8;

      let token_amount = amount_str.parse::<f64>().unwrap_or(0.0) / 10f64.powi(decimals as i32);

      if token_amount > 0.0 {
        tokens.push(TokenInfo {
          mint_address: mint,
          token_balance: token_amount,
          decimals,
          mint_public_key: account.pubkey,
        });
      }
    }
  }

  Ok(tokens)
}

pub fn get_wallet_sol_balance(address: &str) -> f64 {
  let owner_pubkey = Pubkey::from_str(address).unwrap();
  let rpc_url = env::var("SOLANA_RPC_URL");
  let connection = RpcClient::new(rpc_url.unwrap());
  return connection.get_balance(&owner_pubkey).unwrap() as f64;
}

pub async fn register_wallet_tokens(
  address: &str,
  user_id: i32,
  // db: &DatabaseConnection,
) -> Result<(), String> {
  let mut tokens_to_watch: Vec<SlingTokenInfo> = Vec::new();

  match get_spl_tokens_in_wallet(address).await {
    Ok(tokens) => {
      for token in tokens {
        let token_usd_price = match fetch_token_price(&token.mint_address).await {
          Ok(price) => price,
          Err(_e) => continue,
        };

        let token_usd_value = token_usd_price * token.token_balance;

        let min_watchlist_token_amount: f64 = env::var("MINIMUM_WATCHLIST_TOKEN_USD_AMOUNT")
          .ok()
          .and_then(|value| value.parse::<f64>().ok())
          .unwrap_or(10.0);

        if token_usd_value > min_watchlist_token_amount {
          tokens_to_watch.push(SlingTokenInfo {
            mint_address: token.mint_address,
            token_usd_balance: token_usd_value,
            price: token_usd_price,
            token_balance: token.token_balance,
            decimals: token.decimals,
            public_key: token.mint_public_key,
          });
        }
      }
    }
    Err(e) => eprintln!("Error getting tokens for user: {}", e),
  };

  //  this setsup  default stop loss on a wallet
  let db = db::connect_db()
    .await
    .expect("Failed to connect to the database");

  let wallet_in_db = wallets::Entity::find()
    .filter(wallets::Column::Address.eq(address))
    .one(&db)
    .await
    .map_err(|e| e.to_string())?;

  // Check if wallet exists in the database
  if let Some(wallet) = wallet_in_db {
    // This sets up the default stop loss on a wallet
    for token in tokens_to_watch {
      let default_stop_loss: f32 = env::var("DEFAULT_STOP_LOSS_PERCENTAGE")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(40.0);

      let stop_loss_target_price = token.price * ((default_stop_loss / 100.0) as f64);

      // Check if there's an existing strategy for the token
      let existing_strat = trade_orders::Entity::find()
        .filter(trade_orders::Column::WalletId.eq(wallet.id)) // Use wallet.id here
        .filter(trade_orders::Column::UserId.eq(user_id))
        .filter(trade_orders::Column::Strategy.eq("stop_loss"))
        .one(&db)
        .await
        .map_err(|e| e.to_string())?;

      // If an existing strategy is found, skip to the next token
      if existing_strat.is_some() {
        continue;
      }

      // Check if the token exists in the database
      let existing_token = tokens::Entity::find()
        .filter(tokens::Column::ContractAddress.eq(&token.mint_address))
        .one(&db)
        .await
        .map_err(|e| e.to_string())?;

      let token_id = if let Some(existing_token) = existing_token {
        existing_token.id
      } else {
        let new_token = tokens::ActiveModel {
          contract_address: Set(token.mint_address.clone()),
          token_public_key: Set(Some(token.public_key)),
          chain: Set("solana".to_string()),
          decimals: Set(Some(token.decimals as i32)),
          name: Set(None),
          metadata: Set(None),
          ..Default::default()
        };

        let saved_token = Tokens::insert(new_token)
          .exec(&db)
          .await
          .map_err(|e| e.to_string())?;

        saved_token.last_insert_id
      };

      let new_trade_order = trade_orders::ActiveModel {
        user_id: Set(user_id),
        wallet_id: Set(wallet.id),
        token_id: Set(token_id),
        contract_address: Set(token.mint_address),
        reference_price: Set(token.price as f32),
        target_percentage: Set(default_stop_loss),
        target_price: Set(stop_loss_target_price as f32),
        strategy: Set("stop_loss".to_string()),
        created_by: Set("app".to_string()),
        metadata: Set(None),
        ..Default::default()
      };

      trade_orders::Entity::insert(new_trade_order)
        .exec(&db)
        .await
        .map_err(|e| e.to_string())?;
    }
  }

  Ok(())
}
