use anyhow::{Context, Result};
use entity::users;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use teloxide::prelude::*;
use teloxide::types::ChatId;

pub async fn notify_user_by_telegram(tg_id: i64, message: &str) -> Result<()> {
  // Create a bot instance using the bot token from your environment or configuration
  let bot = Bot::from_env();

  // Send the message
  bot
    .send_message(ChatId(tg_id), message)
    .await
    .context("Failed to send message")?;

  Ok(())
}

pub async fn notify_users(msg: String, db: &DatabaseConnection) -> Result<()> {
  let users = users::Entity::find()
    .filter(users::Column::TgId.is_not_null())
    .all(db) // Dereferencing Arc to get a reference to DatabaseConnection
    .await
    .context("Database error")?;

  let mut tasks = vec![];

  for user in users {
    let tg_id_string = user.tg_id.clone(); // Clone to avoid ownership issues
    let message = msg.clone();

    if let Ok(tg_id) = tg_id_string.parse::<i64>() {
      let task = tokio::spawn(async move {
        if let Err(e) = notify_user_by_telegram(tg_id, &message).await {
          eprintln!("Error notifying user {}: {}", tg_id, e);
        }
      });
      tasks.push(task);
    } else {
      eprintln!("Skipping user with invalid tg_id: {}", tg_id_string);
    }
  }
  futures::future::join_all(tasks).await;

  Ok(())
}
