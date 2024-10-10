use aes::Aes128;
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::env;

pub fn hash_password(password: &str) -> Result<String, BcryptError> {
  hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hashed: &str) -> Result<(), String> {
  // Use bcrypt or the hashing method you implemented to verify the password
  match verify(password, hashed) {
    Ok(is_valid) if is_valid => Ok(()),
    _ => Err("Password verification failed".to_string()),
  }
}

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

#[derive(Debug)]
pub struct EncryptPKDetails {
  pub salt: String,
  pub secret_key: String,
  pub encrypted_private_key: String,
}

pub fn encrypt_private_key(
  plaintext: &str,
) -> Result<EncryptPKDetails, Box<dyn std::error::Error>> {
  // Generate a random salt (IV)
  let mut salt: [u8; 16] = [0; 16];
  rand::thread_rng().fill(&mut salt);

  // Get the secret from the environment variable
  let secret = env::var("WALLET_SECRET")?;

  // Generate a key from secret and salt
  let mut hasher = Sha256::new();
  hasher.update(secret.as_bytes());
  hasher.update(&salt);
  let key = hasher.finalize();

  // Take first 16 bytes for AES-128
  let key = &key[0..16];

  // Initialize the cipher
  let cipher = Aes128Cbc::new_from_slices(key, &salt)?;

  // Encrypt the plaintext
  let ciphertext = cipher.encrypt_vec(plaintext.as_bytes());

  // Create and return the EncryptPKDetails struct
  let res = EncryptPKDetails {
    salt: hex::encode(salt),
    secret_key: hex::encode(key),
    encrypted_private_key: hex::encode(ciphertext),
  };

  Ok(res)
}

// Corresponding decryption function
pub fn decrypt_private_key(
  details: &EncryptPKDetails,
) -> Result<String, Box<dyn std::error::Error>> {
  let salt = hex::decode(&details.salt)?;
  let key = hex::decode(&details.secret_key)?;
  let ciphertext = hex::decode(&details.encrypted_private_key)?;

  let cipher = Aes128Cbc::new_from_slices(&key, &salt)?;
  let decrypted_bytes = cipher.decrypt_vec(&ciphertext)?;

  String::from_utf8(decrypted_bytes).map_err(|e| e.into())
}
