#![allow(dead_code)]
use actix_web::HttpRequest;
use anyhow::Ok;
use entity::users;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use std::{
  env,
  time::{SystemTime, UNIX_EPOCH},
};

use ::entity::prelude::*;

// Claims struct to be encoded in the JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub sub: i32, // User ID
  pub tg_token: String,
  pub exp: usize,
}

#[derive(Debug)]
pub enum JwtError {
  Encoding(jsonwebtoken::errors::Error),
  InvalidToken(jsonwebtoken::errors::Error),
  Expired,
  MissingBearer,
  MalformedHeader,
  MissingToken,
}

impl std::fmt::Display for JwtError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      JwtError::Encoding(e) => write!(f, "JWT encoding error: {}", e),
      JwtError::MissingToken => write!(f, "Telegram token is missing"),
      JwtError::InvalidToken(e) => write!(f, "Invalid token: {}", e),
      JwtError::Expired => write!(f, "Token has expired"),
      JwtError::MissingBearer => write!(f, "Authorization header must start with 'Bearer'"),
      JwtError::MalformedHeader => write!(f, "Malformed authorization header"),
    }
  }
}

impl std::error::Error for JwtError {}

pub fn generate_jwt(user: &users::Model) -> Result<String, JwtError> {
  let tg_token = user.tg_token.clone().ok_or(JwtError::MissingToken)?;

  let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

  // Set expiration time to 7 days from now
  let expiration = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs()
    + 7 * 24 * 60 * 60;

  let claims = Claims {
    sub: user.id,
    tg_token,
    exp: expiration as usize,
  };

  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )
  .map_err(JwtError::Encoding)
}

pub fn verify_jwt(token: &str) -> Result<Claims, JwtError> {
  let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
  let validation = Validation::new(Algorithm::HS256);

  decode::<Claims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &validation,
  )
  .map(|token_data| token_data.claims)
  .map_err(JwtError::InvalidToken)
}

pub async fn req_user(req: HttpRequest, db: &DatabaseConnection) -> Option<users::Model> {
  let headers = req.headers();

  // Check for the "authorization" header
  let value = headers.get("authorization")?.to_str().ok()?;

  // Ensure the token starts with "Bearer "
  if !value.starts_with("Bearer ") {
    return None;
  }

  // Extract the token
  let token = &value[7..].trim();

  // Verify the JWT and extract claims
  let claims = verify_jwt(token).ok()?;

  // Find the user by ID and return
  Users::find_by_id(claims.sub).one(db).await.ok()?
}

pub fn authorize_davids_sight(req: &HttpRequest) -> bool {
  use std::result::Result::Ok;

  let secret = match env::var("DAVIDS_SIGHT_KEY") {
    Ok(key) => key,
    Err(_) => return false, // Return false if the environment variable is not set
  };

  // Attempt to get the "x-davids-pouch-key" header
  if let Some(api_key) = req.headers().get("x-davids-pouch-key") {
    if let Ok(api_key_str) = api_key.to_str() {
      return secret == api_key_str;
    }
  }

  false
}
