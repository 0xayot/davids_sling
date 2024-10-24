//  https://github.com/patrick-fitzgerald/actix-web-cron-example/blob/main/src/main.rs

use crate::jobs::cron::{
  price::refresh_sol_token_prices, trade_strat::default_stop_loss_strategy_solana,
  wallets::update_wallet_token_list,
};

use chrono::{Local, Utc};
use tokio_schedule::{every, Job};

pub async fn start_cron() {
  let every_second = every(1).seconds().in_timezone(&Utc).perform(|| async {
    println!("schedule_task job - {:?}", Local::now());
  });
  // every_second.await;

  let sol_price_update = every(3).seconds().in_timezone(&Utc).perform(|| async {
    println!("sol_price_update job - {:?}", Local::now());
    if let Err(err) = refresh_sol_token_prices().await {
      eprintln!("Failed to refresh token prices: {:?}", err);
    }
  });

  let update_spl_tokens_in_wallet = every(3).minutes().in_timezone(&Utc).perform(|| async {
    println!(
      "running update_spl_tokens_in_wallet job - {:?}",
      Local::now()
    );
    if let Err(err) = update_wallet_token_list().await {
      eprintln!("Failed to refresh token prices: {:?}", err);
    }
  });

  let run_default_stop_loss = every(1).minutes().in_timezone(&Utc).perform(|| async {
    println!(" running run_default_stop_loss - {:?}", Local::now());
    if let Err(err) = default_stop_loss_strategy_solana().await {
      eprintln!("Failed to refresh token prices: {:?}", err);
    }
  });

  tokio::spawn(every_second);
  tokio::spawn(sol_price_update);
  tokio::spawn(update_spl_tokens_in_wallet);
  tokio::spawn(run_default_stop_loss);
  // sol_price_update.await;
}
