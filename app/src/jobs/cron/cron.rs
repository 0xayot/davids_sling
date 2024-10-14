//  https://github.com/patrick-fitzgerald/actix-web-cron-example/blob/main/src/main.rs

use crate::jobs::cron::price::refresh_sol_token_prices;

use chrono::{Local, Utc};
use tokio_schedule::{every, Job};

pub async fn start_cron() {
  let every_second = every(1).seconds().in_timezone(&Utc).perform(|| async {
    println!("schedule_task event - {:?}", Local::now());
  });
  // every_second.await;
  tokio::spawn(every_second);

  let sol_price_update = every(3).seconds().in_timezone(&Utc).perform(|| async {
    println!("sol_price_update event - {:?}", Local::now());
    if let Err(err) = refresh_sol_token_prices().await {
      eprintln!("Failed to refresh token prices: {:?}", err);
    }
  });
  tokio::spawn(sol_price_update);
  // sol_price_update.await;
}
