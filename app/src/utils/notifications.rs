use anyhow::{Context, Result};
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
