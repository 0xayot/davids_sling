use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};

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
