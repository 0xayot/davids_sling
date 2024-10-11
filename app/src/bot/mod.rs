use ::entity::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::{
  db,
  utils::{
    encryption,
    misc::generate_uuid,
    wallets::solana::{generate_wallet, recover_wallet_from_private_key},
  },
};
#[derive(BotCommands, Clone)]
#[command(
  rename_rule = "lowercase",
  description = "These commands are supported:"
)]
pub enum Command {
  #[command(description = "display this text.")]
  Help,
  #[command(description = "Start with tg token")]
  WatchSolWallet(String),
  #[command(
    description = "Welcome add your email space password",
    parse_with = "split"
  )]
  Start { email: String, password: String },
  #[command(
    description = "Welcome add your email space tGtoken",
    parse_with = "split"
  )]
  TgToken { email: String, tg_token: String },
  #[command(description = "Add wallet with pk space title", parse_with = "split")]
  AddSolWallet { pk: String, title: String },
  #[command(description = "Create new sol wallet")]
  CreateSolWallet,
}

pub async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
  let db = db::connect_db()
    .await
    .expect("Failed to connect to the database");
  println!("{:?}", msg);

  match cmd {
    Command::Help => {
      bot
        .send_message(msg.chat.id, Command::descriptions().to_string())
        .await?
    }
    Command::WatchSolWallet(address) => {
      bot
        .send_message(msg.chat.id, format!("Your address is @{address}."))
        .await?
    }
    Command::TgToken { email, tg_token } => {
      // Query to find the existing user
      let existing_user_result = users::Entity::find()
        .filter(users::Column::Email.eq(&email))
        .filter(users::Column::TgToken.eq(&tg_token))
        .one(&db)
        .await;

      let tg_user = msg.from.unwrap();

      let response_message = match existing_user_result {
        Ok(Some(user)) => {
          let mut active_model: entity::users::ActiveModel = user.into();
          active_model.tg_id = Set(tg_user.id.to_string());

          let _ = active_model.update(&db).await.map_err(|e| e.to_string());

          "Welcome!"
        }
        Ok(None) => "Oops, invalid action.",
        Err(_e) => "Error:",
      };

      // Send the response message
      bot.send_message(msg.chat.id, response_message).await?
    }

    Command::Start { email, password } => {
      let existing_user_result = users::Entity::find()
        .filter(users::Column::Email.eq(&email))
        .one(&db)
        .await;

      let tg_user = msg.from.unwrap();

      let response_message = match existing_user_result {
        Ok(Some(_user)) => "Oops, invalid action.",
        Ok(None) => {
          let hashed_password = match encryption::hash_password(&password) {
            Ok(hash) => hash,
            Err(_) => "Error creating user.".to_owned(),
          };

          let new_user = users::ActiveModel {
            email: Set(Some(email.clone())),
            tg_id: Set(tg_user.id.to_string()),
            tg_token: Set(Some(generate_uuid())),
            encrypted_password: Set(hashed_password),
            ..Default::default()
          };

          // Insert the new user into the database
          match new_user.insert(&db).await {
            Ok(_) => "Welcome!",
            Err(_) => "Error creating user.",
          }
        }
        Err(_) => "Database error occurred.",
      };

      // Send the response message
      bot.send_message(msg.chat.id, response_message).await?
    }

    Command::AddSolWallet { pk, title } => {
      let tg_user = msg.from.unwrap();

      // Query for the authorized user based on Telegram ID
      let authorized_user = users::Entity::find()
        .filter(users::Column::TgId.eq(&tg_user.id.to_string()))
        .one(&db)
        .await;

      let response_message = match authorized_user {
        Ok(Some(user)) => {
          let signer = recover_wallet_from_private_key(&pk).unwrap();
          let existing_wallet = wallets::Entity::find()
            .filter(wallets::Column::Chain.eq("Solana".to_string()))
            .filter(wallets::Column::Address.eq(signer.public_key.clone()))
            .one(&db)
            .await;

          match existing_wallet {
            Ok(Some(_)) => "Wallet already exists.",
            Ok(None) => {
              // Create the new wallet if it doesn't exist
              let wallet = wallets::ActiveModel {
                title: Set(title.clone()),
                chain: Set("Solana".to_string()),
                user_id: Set(user.id),
                salt: Set(signer.salt.unwrap()),
                secret_key: Set(signer.secret_key.unwrap()),
                encrypted_private_key: Set(signer.encrypted_private_key.unwrap()),
                address: Set(signer.public_key),
                encryption_schema: Set("From_PK".to_string()),
                ..Default::default()
              };

              match wallet.insert(&db).await {
                Ok(_) => "Wallet created successfully!",
                Err(_) => "Error inserting wallet.",
              }
            }
            Err(_) => "Database error occurred while checking wallet existence.",
          }
        }
        Ok(None) => "Oops, invalid action. Begin at /start.",
        Err(_) => "Database error occurred.",
      };

      // Send the response message
      bot.send_message(msg.chat.id, response_message).await?
    }

    Command::CreateSolWallet => {
      let tg_user = msg.from.unwrap();

      // Query for the authorized user based on Telegram ID
      let authorized_user = users::Entity::find()
        .filter(users::Column::TgId.eq(&tg_user.id.to_string()))
        .one(&db)
        .await;

      let response_message = match authorized_user {
        Ok(Some(user)) => {
          let signer = generate_wallet();
          let existing_wallet = wallets::Entity::find()
            .filter(wallets::Column::Chain.eq("Solana".to_string()))
            .filter(wallets::Column::Address.eq(signer.public_key.clone()))
            .one(&db)
            .await;

          match existing_wallet {
            Ok(Some(_)) => "Wallet already exists.",
            Ok(None) => {
              // Create the new wallet if it doesn't exist
              let wallet = wallets::ActiveModel {
                title: Set("Davids Sling Wallet".to_string()),
                chain: Set("Solana".to_string()),
                user_id: Set(user.id),
                salt: Set(signer.salt.unwrap()),
                secret_key: Set(signer.secret_key.unwrap()),
                encrypted_private_key: Set(signer.encrypted_private_key.unwrap()),
                address: Set(signer.public_key),
                encryption_schema: Set("Sling Wallet".to_string()),
                ..Default::default()
              };

              match wallet.insert(&db).await {
                Ok(_) => "Wallet created successfully!",
                Err(_) => "Error inserting wallet.",
              }
            }
            Err(_) => "Database error occurred while checking wallet existence.",
          }
        }
        Ok(None) => "Oops, invalid action. Begin at /start.",
        Err(_) => "Database error occurred.",
      };

      // Send the response message
      bot.send_message(msg.chat.id, response_message).await?
    }
  };

  Ok(())
}
