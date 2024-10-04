use sea_orm::{ConnectOptions, Database};
use std::env;
use std::time::Duration;

pub type DBConnection = Result<sea_orm::DatabaseConnection, sea_orm::DbErr>;

pub async fn connect_db() -> DBConnection {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut opts = ConnectOptions::new(db_url.clone());

    opts.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("db_schema");

    // Pass opts directly without dereferencing
    Database::connect(opts).await
}
