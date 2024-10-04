use actix_cors::Cors;
use actix_web::{get, middleware::Logger, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use std::env;

mod db;
mod gql;

use gql::handler::graphql_server;

#[get("/")]
async fn hello() -> impl Responder {
    match env::var("DATABASE_URL") {
        Ok(db_url) => println!("DATABASE_URL: {}", db_url),
        Err(e) => eprintln!("Could not read DATABASE_URL: {}", e),
    }
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load environment variables from .env file

    let db = db::connect_db()
        .await
        .expect("Failed to connect to the database");

    log::info!("starting HTTP server on port 8080");
    log::info!("GraphiQL playground: http://localhost:8080/graphiql");

    HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .configure(graphql_server)
            .wrap(Cors::permissive())
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 9000))?
    .run()
    .await
}
