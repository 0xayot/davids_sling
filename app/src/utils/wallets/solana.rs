use base64::decode;
use solana_account_decoder::UiAccountData;
use solana_sdk::{
  bs58,
  program_pack::Pack,
  signer::{keypair::Keypair, Signer},
};

use solana_client::{rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use solana_sdk::pubkey::Pubkey;
use spl_token::state::{Account as TokenAccount, Mint};
use std::{env, str::FromStr};

use solana_client::nonblocking::rpc_client::RpcClient as AsyncClient;

use crate::utils::encryption::encrypt_private_key;

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
}

pub async fn get_spl_tokens_in_wallet(
  address: &str,
) -> Result<Vec<TokenInfo>, Box<dyn std::error::Error>> {
  // Connect to Solana network
  let rpc_url = env::var("SOLANA_RPC_URL")?;
  let client = AsyncClient::new(rpc_url);

  // Create a Pubkey from the address string
  let pubkey = Pubkey::from_str(address)?;

  // Fetch all token accounts owned by this address
  let token_accounts = client
    .get_token_accounts_by_owner(&pubkey, TokenAccountsFilter::ProgramId(spl_token::id()))
    .await?;

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


pub fn register_wallet_tokens