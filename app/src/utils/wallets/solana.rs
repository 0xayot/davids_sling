use solana_sdk::{
  bs58,
  signer::{keypair::Keypair, Signer},
};

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
