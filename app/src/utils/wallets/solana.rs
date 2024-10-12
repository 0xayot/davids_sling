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
struct TokenInfo {
  mint_address: String,
  token_balance: f64,
  decimals: u8,
}

pub fn get_spl_tokens_in_wallet(
  address: &str,
) -> Result<Vec<TokenInfo>, Box<dyn std::error::Error>> {
  // Connect to Solana network
  let rpc_url = env::var("SOLANA_RPC_URL")?;
  let client = RpcClient::new(rpc_url);

  // Create a Pubkey from the address string
  let pubkey = Pubkey::from_str(address)?;

  // Fetch all token accounts owned by this address
  let token_accounts =
    client.get_token_accounts_by_owner(&pubkey, TokenAccountsFilter::ProgramId(spl_token::id()))?;

  // Process and return the token information
  let mut tokens = Vec::new();
  for account in token_accounts {
    // Handle UiAccountData
    if let UiAccountData::Binary(data, _) = account.account.data {
      // Decode the base64-encoded string to raw bytes
      let decoded_data = decode(data)?;

      // Unpack the token account
      let token_account: TokenAccount = TokenAccount::unpack(&decoded_data)?;

      // Fetch the mint account data
      let mint_account = client.get_account(&token_account.mint)?;
      let mint: Mint = Mint::unpack(&mint_account.data)?;

      let token_amount = token_account.amount as f64 / 10f64.powi(mint.decimals as i32);

      tokens.push(TokenInfo {
        mint_address: token_account.mint.to_string(),
        token_balance: token_amount,
        decimals: mint.decimals,
      });
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
