use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

use super::m20241008_115542_create_user_table::User;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Wallet::Table)
          .if_not_exists()
          .col(pk_auto(Wallet::Id))
          .col(ColumnDef::new(Wallet::Title).string().not_null()) // Added not_null()
          .col(ColumnDef::new(Wallet::Chain).string().not_null())
          .col(ColumnDef::new(Wallet::Address).string().not_null())
          .col(
            ColumnDef::new(Wallet::EncryptedPrivateKey)
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(Wallet::SecretKey).string().not_null())
          .col(ColumnDef::new(Wallet::Salt).string().not_null())
          .col(ColumnDef::new(Wallet::EncryptionSchema).string().not_null()) // Corrected spelling
          .col(ColumnDef::new(Wallet::UserId).integer().not_null())
          .col(
            ColumnDef::new(Wallet::CreatedAt)
              .timestamp()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .col(
            ColumnDef::new(Wallet::UpdatedAt)
              .timestamp()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-wallet-user_id")
              .from(Wallet::Table, Wallet::UserId)
              .to(User::Table, User::Id),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Wallet::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
pub enum Wallet {
  #[sea_orm(iden = "wallets")]
  Table,
  Id,
  Title,
  Chain,
  Address,
  EncryptedPrivateKey,
  SecretKey,
  Salt,
  EncryptionSchema,
  UserId,
  CreatedAt,
  UpdatedAt,
}
