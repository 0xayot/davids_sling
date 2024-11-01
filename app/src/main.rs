use actix_cors::Cors;
use actix_web::{get, middleware::Logger, App, HttpResponse, HttpServer, Responder};

use bot::{answer, Command};
use dotenvy::dotenv;
use integrations::raydium::RaydiumPriceFetcher;
use jobs::cron::cron::start_cron;
use std::env;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::join;
use utils::cache::set_memcache_string;

mod bot;
mod db;
mod gql;
mod integrations;
mod jobs;
mod routes;
mod utils;

use gql::graphql_server;

#[get("/")]
async fn hello() -> impl Responder {
  // Log the database URL for debugging purpose
  match env::var("DATABASE_URL") {
    Ok(db_url) => println!("DATABASE_URL: {}", db_url),
    Err(e) => eprintln!("Could not read DATABASE_URL: {}", e),
  }

  let raydium_client = RaydiumPriceFetcher::new();

  // let p = raydium_client
  //   .get_swap_quote(
  //     "So11111111111111111111111111111111111111112",
  //     "5z3EqYQo9HiCEs3R84RCDMu2n7anpDMxRhdK8PSWmrRC",
  //     "100000000",
  //     "50",
  //   )
  //   .await;

  // println!("{:?}", p.unwrap());
  // // print!("{}", p);

  // let x = raydium_client.get_priority_fee().await;

  // println!("{:?}", x.unwrap());
  // let input_mint = "5z3EqYQo9HiCEs3R84RCDMu2n7anpDMxRhdK8PSWmrRC";

  // let wrap_sol = input_mint == "So11111111111111111111111111111111111111112";

  // let swap_tx = raydium_client
  //   .get_swap_tx(
  //     "Hj3G1N1NXvvNaWE7KREM5vskNjpn4ofPvFHZ97gWh9cX",
  //     p.unwrap(),
  //     "So11111111111111111111111111111111111111112",
  //     input_mint,
  //     "E2s1dLtMtUx58tSC5h2cprDmP9259EBjRUo9XV2repft",
  //   )
  //   .await;

  // println!("{:?} {:?}", wrap_sol, swap_tx.unwrap());

  // let x = raydium_client
  //   .get_token_price_list(Some(
  //     "5z3EqYQo9HiCEs3R84RCDMu2n7anpDMxRhdK8PSWmrRC,So11111111111111111111111111111111111111112"
  //       .to_owned(),
  //   ))
  //   .await;

  // let z = raydium_client
  //   .get_token_price_in_usd("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
  //   .await
  //   .unwrap();

  // println!("{}", z);

  HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv().ok();
  pretty_env_logger::init();
  log::info!("Starting command bot and HTTP server...");

  let db = Arc::new(
    db::connect_db()
      .await
      .expect("Failed to connect to the database"),
  );

  let bot = Bot::from_env();

  // Set bot commands
  bot
    .set_my_commands(Command::bot_commands())
    .await
    .expect("Failed to set bot commands");

  log::info!("Starting HTTP server on port 9000");
  log::info!("GraphiQL playground: http://localhost:9000/graphiql");

  set_memcache_string("test".to_string(), "rest".to_string(), Some(60));

  // actix_rt::spawn(async move {
  //   start_cron().await;
  // });

  // Create and run the Actix server
  let server = HttpServer::new(move || {
    App::new()
      .app_data(db.clone())
      .configure(graphql_server)
      .configure(routes::init_routes)
      .service(hello)
      .wrap(Cors::permissive())
      .wrap(Logger::default())
  })
  .bind(("127.0.0.1", 9000))?
  .run();

  // Run the Teloxide bot
  let bot_handler = Command::repl(bot, answer);

  // Run the server and bot concurrently
  let (server_result, _) = join!(server, bot_handler);

  // Return the server result
  server_result
}
