use actix_cors::Cors;
use actix_web::{get, middleware::Logger, App, HttpResponse, HttpServer, Responder};

use bot::{answer, Command};
use dotenvy::dotenv;
use jobs::cron::cron::start_cron;
use std::env;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::join;

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

  // let analyzer = PriceAnalyzer::new(3, 1.0);

  // let prices = vec![100.0, 102.0, 97.0, 108.0];

  // let trend = analyzer.analyze_trend(&prices);

  // println!("Dance {:?}", trend);

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

  actix_rt::spawn(async move {
    start_cron().await;
  });

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
