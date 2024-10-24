pub use sea_orm_migration::prelude::*;

mod m20241008_115542_create_user_table;
mod m20241008_121835_create_wallet_table;
mod m20241014_175742_create_price_table;
mod m20241014_191627_create_token_table;
mod m20241014_194139_create_trade_orders_table;
mod m20241019_002947_create_onchain_transactions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
            Box::new(m20241008_115542_create_user_table::Migration),
            Box::new(m20241008_121835_create_wallet_table::Migration),
            Box::new(m20241014_175742_create_price_table::Migration),
            Box::new(m20241014_191627_create_token_table::Migration),
            Box::new(m20241014_194139_create_trade_orders_table::Migration),
            Box::new(m20241019_002947_create_onchain_transactions::Migration),
        ]
  }
}
