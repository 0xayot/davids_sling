use super::root::Context;
use crate::utils::{
  auth::generate_jwt,
  encryption::{self, verify_password},
  misc::generate_uuid,
};
use ::entity::{prelude::*, *};
use chrono::Utc;
use juniper::{graphql_object, GraphQLInputObject};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

#[derive(Default, Debug)]
pub struct User {
  pub id: i32,
  pub email: Option<String>,
  pub tg_id: Option<String>,
  pub tg_token: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

#[graphql_object(context = Context)]
impl User {
  fn id(&self) -> i32 {
    self.id
  }

  fn email(&self) -> &Option<String> {
    &self.email
  }

  fn tg_id(&self) -> &Option<String> {
    &self.tg_id
  }

  fn tg_token(&self) -> &Option<String> {
    &self.tg_token
  }

  fn created_at(&self) -> String {
    self.created_at.to_string() // Convert to String
  }

  fn updated_at(&self) -> String {
    self.updated_at.to_string() // Convert to String
  }

  // async fn wallets(&self, context: &Context) -> Vec<Wallet> {
  //   let mut db = &context.db;
  //   let wallets = Users::find()
  //     .filter(wallets::Column::UserId.eq(*&self.id))
  //     .all(&mut db)
  //     .await
  //     .unwrap_or_default();
  //   wallets
  // }
}

#[derive(GraphQLInputObject)]
#[graphql(description = "Input for creating a new user")]
pub struct NewUserInput {
  pub email: String,
  pub tg_id: String,
  pub password: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "Input for login user")]
pub struct LoginInput {
  pub email: String,
  pub password: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "Input for updating an existing user")]
pub struct UpdateUserInput {
  pub email: Option<String>,
  pub tg_token: Option<String>,
  pub password: Option<String>,
}

pub struct UserQuery;

#[graphql_object(context = Context)]
impl UserQuery {
  async fn user(context: &Context, id: i32) -> Result<Option<User>, String> {
    let user = users::Entity::find_by_id(id)
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

  // async fn users(context: &Context) -> Result<Vec<User>, String> {
  //     let users = users::Entity::find()
  //         .all(&context.db)
  //         .await
  //         .map_err(|e| e.to_string())?;

  //     Ok(users.into_iter().map(|u| User {
  //         id: u.id,
  //         email: u.email,
  //         tg_id: u.tg_id,
  //         tg_token: u.tg_token,
  //         created_at: u.created_at,
  //         updated_at: u.updated_at,
  //     }).collect())
  // }
}

pub struct UserMutation;

#[graphql_object(context = Context)]
impl UserMutation {
  async fn add_user(context: &Context, input: NewUserInput) -> Result<User, String> {
    // Check if the password is empty and handle the error
    if input.password.is_empty() {
      return Err("Password cannot be empty".to_string());
    }

    // Hash the password and handle potential errors
    let hashed_password = match encryption::hash_password(&input.password) {
      Ok(hash) => hash,
      Err(e) => return Err(format!("Error hashing password: {}", e)),
    };

    // Create the new user model
    let user = users::ActiveModel {
      email: Set(Some(input.email.clone())),
      tg_id: Set(input.tg_id),
      tg_token: Set(Some(generate_uuid())),
      encrypted_password: Set(hashed_password),
      ..Default::default()
    };

    // Attempt to insert the user into the database
    let result = Users::insert(user)
      .exec(&context.db)
      .await
      .map_err(|e| e.to_string())?; // Map database errors to string

    // Construct and return the user
    Ok(User {
      id: result.last_insert_id,
      email: Some(input.email),
      tg_id: None,
      tg_token: None,
      created_at: Utc::now().naive_utc().to_string(),
      updated_at: Utc::now().naive_utc().to_string(),
    })
  }

  async fn login(context: &Context, input: LoginInput) -> Result<String, String> {
    // Find the user by email
    let user = Users::find()
      .filter(users::Column::Email.eq(&input.email))
      .one(&context.db)
      .await
      .map_err(|e| e.to_string())?
      .ok_or("User not found".to_string())?;

    // Check if the provided password matches the hashed password
    if verify_password(&input.password, &user.encrypted_password).is_err() {
      return Err("Invalid password".to_string());
    }

    // Generate and return the JWT
    generate_jwt(&user).map_err(|e| e.to_string())
  }

  // async fn update_user(context: &Context, id: i32, input: UpdateUserInput) -> Result<User, String> {
  //     let mut user: users::ActiveModel = users::Entity::find_by_id(id)
  //         .one(&context.db)
  //         .await
  //         .map_err(|e| e.to_string())?
  //         .ok_or_else(|| "User not found".to_string())?
  //         .into();

  //     if let Some(email) = input.email {
  //         user.email = Set(Some(email));
  //     }
  //     if let Some(tg_token) = input.tg_token {
  //         user.tg_token = Set(Some(tg_token));
  //     }
  //     if let Some(password) = input.password {
  //         user.encrypted_password = Set(password); // Note: You should hash this before saving
  //     }

  //     let result = user.update(&context.db)
  //         .await
  //         .map_err(|e| e.to_string())?;

  //     Ok(User {
  //         id: result.id,
  //         email: result.email,
  //         tg_id: result.tg_id,
  //         tg_token: result.tg_token,
  //         created_at: result.created_at,
  //         updated_at: result.updated_at,
  //     })
  // }

  // async fn delete_user(context: &Context, id: i32) -> Result<bool, String> {
  //     let result = users::Entity::delete_by_id(id)
  //         .exec(&context.db)
  //         .await
  //         .map_err(|e| e.to_string())?;

  //     Ok(result.rows_affected > 0)
  // }
}

#[derive(GraphQLInputObject)]
#[graphql(description = "User Input")]
pub struct UserInput {
  pub tg_id: Option<String>,
  pub email: String,
  pub password: String,
}
