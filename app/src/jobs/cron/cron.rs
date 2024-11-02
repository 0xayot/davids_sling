//  https://github.com/patrick-fitzgerald/actix-web-cron-example/blob/main/src/main.rs

use crate::jobs::cron::{
  price::{refresh_sol_token_prices, track_launch_event_token_prices},
  trade_strat::default_stop_loss_strategy_solana,
  wallets::update_wallet_token_list,
};

use chrono::{Local, Utc};
use tokio_schedule::{every, Job};

use super::price::refresh_sol_tokens_to_watch;

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

  let refresh_sol_tokens_to_watch = every(3)
    .minutes()
    .in_timezone(&Utc)
    .perform(|| async { refresh_sol_tokens_to_watch().await });

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

  let run_default_stop_loss = every(1).minutes().in_timezone(&Utc).perform(|| async {
    println!(" running run_default_stop_loss - {:?}", Local::now());
    if let Err(err) = default_stop_loss_strategy_solana().await {
      eprintln!("Failed to refresh token prices: {:?}", err);
    }
  });

  let run_track_spied_launch = every(10).seconds().in_timezone(&Utc).perform(|| async {
    println!(" running track token launch lifespan - {:?}", Local::now());
    if let Err(err) = track_launch_event_token_prices().await {
      eprintln!("Failed to update prices {:?}", err);
    }
  });

  tokio::spawn(every_second);
  tokio::spawn(refresh_sol_tokens_to_watch);
  tokio::spawn(sol_price_update);
  tokio::spawn(update_spl_tokens_in_wallet);
  tokio::spawn(run_default_stop_loss);
  tokio::spawn(run_track_spied_launch);
}
