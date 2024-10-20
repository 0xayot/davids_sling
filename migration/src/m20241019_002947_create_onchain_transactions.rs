use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

use super::m20241008_115542_create_user_table::User;
use crate::m20241008_121835_create_wallet_table::Wallet;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(OnchainTransaction::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(OnchainTransaction::Id)
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(OnchainTransaction::UserId)
              .integer()
              .not_null(),
          )
          .col(
            ColumnDef::new(OnchainTransaction::WalletId)
              .integer()
              .not_null(),
          )
          .col(ColumnDef::new(OnchainTransaction::TransactionHash).string())
          .col(
            ColumnDef::new(OnchainTransaction::Chain)
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(OnchainTransaction::Source).string())
          .col(ColumnDef::new(OnchainTransaction::Status).string())
          .col(ColumnDef::new(OnchainTransaction::Type).string())
          .col(ColumnDef::new(OnchainTransaction::ValueNative).float())
          .col(ColumnDef::new(OnchainTransaction::ValueUsd).float())
          .col(ColumnDef::new(OnchainTransaction::FromToken).string())
          .col(ColumnDef::new(OnchainTransaction::ToToken).string())
          .col(
            ColumnDef::new(OnchainTransaction::CreatedAt)
              .timestamp()
              .default(Expr::current_timestamp())
              .not_null(),
          )
          .col(
            ColumnDef::new(OnchainTransaction::UpdatedAt)
              .timestamp()
              .default(Expr::current_timestamp())
              .not_null(),
          )
          .to_owned(),
      )
      .await?;

    // Add foreign key constraints
    manager
      .create_foreign_key(
        ForeignKey::create()
          .name("fk_onchain_transactions_user")
          .from(OnchainTransaction::Table, OnchainTransaction::UserId)
          .to(User::Table, User::Id)
          .to_owned(),
      )
      .await?;

    manager
      .create_foreign_key(
        ForeignKey::create()
          .name("fk_onchain_transactions_wallet")
          .from(OnchainTransaction::Table, OnchainTransaction::WalletId)
          .to(Wallet::Table, Wallet::Id)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(OnchainTransaction::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
pub enum OnchainTransaction {
  Table,
  Id,
  UserId,
  WalletId,
  TransactionHash,
  Chain,
  Source,
  Status,
  Type,
  ValueNative,
  ValueUsd,
  FromToken,
  ToToken,
  CreatedAt,
  UpdatedAt,
}
