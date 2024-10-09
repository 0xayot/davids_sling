use sea_orm::ConnectOptions;
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::{env, time::Duration};

// pub type DBConnection = sea_orm::DatabaseConnection;
pub type DBConnection = Result<DatabaseConnection, DbErr>;

pub async fn connect_db() -> DBConnection {
  let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

  let mut opts = ConnectOptions::new(db_url.clone());

  opts
    .max_connections(100)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(8))
    .acquire_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8))
    .max_lifetime(Duration::from_secs(8))
    .sqlx_logging(true)
    .sqlx_logging_level(log::LevelFilter::Info)
    .set_schema_search_path("db_schema");

  // let _connection: sea_orm::DatabaseConnection = Database::connect(opts).await?;
  // let conn = Database::connect(&db_url).await.unwrap();
  // // //    conn
  // // let _db = Database::connect(db_url).await?;
  // // Ok(())
  let conn = Database::connect(&db_url).await.unwrap();
  // //    conn
  // let _db = Database::connect(db_url).await?;
  return Ok(conn);
}
