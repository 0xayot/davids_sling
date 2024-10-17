use crate::{
  gql::schemas::{root::Context, user::User},
  utils::wallets::solana::{
    generate_wallet, recover_wallet_from_private_key, register_wallet_tokens,
  },
};
use ::entity::{prelude::*, *};
use juniper::{graphql_object, GraphQLEnum, GraphQLInputObject};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

/// Wallet
#[derive(Default, Debug)]
pub struct Wallet {
  pub id: i32,
  pub title: String,
  pub chain: String,
  pub address: String,
  pub user_id: i32,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(GraphQLEnum)]
enum Chain {
  Solana,
  Base,
  BNB,
  Sui,
  Tron,
}

/// GraphQL object implementation for Wallet
#[graphql_object(context = Context)]
impl Wallet {
  fn id(&self) -> i32 {
    self.id
  }

  fn title(&self) -> &str {
    &self.title
  }

  fn chain(&self) -> &str {
    &self.chain
  }

  fn address(&self) -> &str {
    &self.address
  }

  fn user_id(&self) -> i32 {
    self.user_id
  }

  fn created_at(&self) -> String {
    self.created_at.to_string()
  }

  fn updated_at(&self) -> String {
    self.updated_at.to_string()
  }

  async fn user(&self, context: &Context) -> Result<Option<User>, String> {
    let user = users::Entity::find_by_id(self.user_id)
      .one(&context.db)
      .await
      .map_err(|e| e.to_string())?;

    Ok(user.map(|u| User {
      id: u.id,
      email: u.email,
      tg_id: Some(u.tg_id),
      tg_token: u.tg_token,
      created_at: u.created_at.to_string(),
      updated_at: u.updated_at.to_string(),
    }))
  }
}

#[derive(GraphQLInputObject)]
pub struct NewWalletInput {
  pub title: String,
  pub chain: String,
  pub address: Option<String>,
}

pub struct WalletQuery;

#[graphql_object(context = Context)]
impl WalletQuery {
  async fn wallet(context: &Context, id: i32) -> Result<Option<Wallet>, String> {
    let wallet = wallets::Entity::find_by_id(id)
      .one(&context.db)
      .await
      .map_err(|e| e.to_string())?;

    Ok(wallet.map(|w| Wallet {
      id: w.id,
      title: w.title,
      chain: w.chain,
      address: w.address,
      user_id: w.user_id,
      created_at: w.created_at.to_string(),
      updated_at: w.updated_at.to_string(),
    }))
  }

  async fn wallets(context: &Context) -> Result<Vec<Wallet>, String> {
    let wallet_user = context.user.as_ref().ok_or("User not found")?;
    let user_id = wallet_user.id;

    let wallets = Wallets::find()
      .filter(wallets::Column::UserId.eq(user_id)) // Remove & from &user_id
      .all(&context.db)
      .await
      .map_err(|e| e.to_string())?;

    Ok(
      wallets
        .into_iter()
        .map(|w| Wallet {
          id: w.id,
          title: w.title,
          chain: w.chain,
          address: w.address,
          user_id: w.user_id,
          created_at: w.created_at.to_string(),
          updated_at: w.updated_at.to_string(),
        })
        .collect(),
    )
  }
}

pub struct WalletMutation;

#[graphql_object(context = Context)]
impl WalletMutation {
  async fn create_wallet(context: &Context, input: NewWalletInput) -> Result<Wallet, String> {
    let wallet_user = context.user.as_ref().ok_or("User not found")?;

    // Create the wallet model
    let mut wallet = wallets::ActiveModel {
      title: Set(input.title),
      chain: Set(input.chain.clone()),
      user_id: Set(wallet_user.id),
      encryption_schema: Set("default".to_string()),
      ..Default::default()
    };

    match &input.address {
      Some(address) => {
        // Recover wallet from private key
        let signer = recover_wallet_from_private_key(address).ok_or("Invalid private key")?;

        let existing_wallet = wallets::Entity::find()
          .filter(wallets::Column::Chain.eq(input.chain.clone()))
          .filter(wallets::Column::Address.eq(signer.public_key.clone()))
          .one(&context.db)
          .await
          .map_err(|e| e.to_string())?;

        if existing_wallet.is_some() {
          return Err("A wallet with the same chain and address already exists.".to_string());
        }

        wallet.address = Set(signer.public_key);
        if let Some(salt) = signer.salt {
          wallet.salt = Set(salt);
        }
        if let Some(secret_key) = signer.secret_key {
          wallet.secret_key = Set(secret_key);
        }
        if let Some(encrypted_private_key) = signer.encrypted_private_key {
          wallet.encrypted_private_key = Set(encrypted_private_key);
        }
      }
      None => {
        let signer = generate_wallet();

        let existing_wallet = wallets::Entity::find()
          .filter(wallets::Column::Chain.eq(input.chain.clone()))
          .filter(wallets::Column::Address.eq(signer.public_key.clone()))
          .one(&context.db)
          .await
          .map_err(|e| e.to_string())?;

        if existing_wallet.is_some() {
          return Err("A wallet with the same chain and address already exists.".to_string());
        }

        // Set the wallet fields
        wallet.address = Set(signer.public_key);
        if let Some(salt) = signer.salt {
          wallet.salt = Set(salt);
        }
        if let Some(secret_key) = signer.secret_key {
          wallet.secret_key = Set(secret_key);
        }
        if let Some(encrypted_private_key) = signer.encrypted_private_key {
          wallet.encrypted_private_key = Set(encrypted_private_key);
        }
      }
    }

    let result = Wallets::insert(wallet)
      .exec(&context.db)
      .await
      .map_err(|e| e.to_string())?;

    let record = wallets::Entity::find_by_id(result.last_insert_id)
      .one(&context.db)
      .await
      .map_err(|e| e.to_string())?
      .ok_or("Failed to retrieve wallet after insertion")?;

    // Clone the address to ensure it lives long enough
    let address = record.address.clone();
    let user_id = wallet_user.id.clone();

    tokio::spawn(async move {
      let _ = register_wallet_tokens(&address, user_id).await;
    });

    // Return the Wallet struct using the original record
    Ok(Wallet {
      id: record.id,
      title: record.title,
      chain: record.chain,
      address: record.address,
      user_id: record.user_id,
      created_at: record.created_at.to_string(),
      updated_at: record.updated_at.to_string(),
    })
  }
}
