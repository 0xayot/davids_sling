use sea_orm_migration::{prelude::*, schema::pk_auto};

#[derive(DeriveMigrationName)]
pub struct Migration;

use super::m20241008_115542_create_user_table::User;
use crate::m20241008_121835_create_wallet_table::Wallet;
use crate::m20241014_191627_create_token_table::Token;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(TradeOrder::Table)
          .if_not_exists()
          .col(pk_auto(TradeOrder::Id))
          .col(ColumnDef::new(TradeOrder::UserId).integer().not_null())
          .col(ColumnDef::new(TradeOrder::WalletId).integer().not_null())
          .col(ColumnDef::new(TradeOrder::TokenId).integer().not_null())
          .col(
            ColumnDef::new(TradeOrder::ReferencePrice)
              .float()
              .not_null()
              .default(0.0),
          )
          .col(
            ColumnDef::new(TradeOrder::TargetPrice)
              .float()
              .not_null()
              .default(0.0),
          )
          .col(
            ColumnDef::new(TradeOrder::TargetPercentage)
              .float()
              .not_null()
              .default(0.0),
          )
          .col(
            ColumnDef::new(TradeOrder::ContractAddress)
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(TradeOrder::Strategy).string().not_null())
          .col(
            ColumnDef::new(TradeOrder::Active)
              .boolean()
              .not_null()
              .default(true),
          )
          .col(ColumnDef::new(TradeOrder::CreatedBy).string().not_null())
          .col(ColumnDef::new(TradeOrder::Metadata).json())
          .col(
            ColumnDef::new(TradeOrder::CreatedAt)
              .date_time()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .col(
            ColumnDef::new(TradeOrder::UpdatedAt)
              .date_time()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_trade_orders_user")
              .from(TradeOrder::Table, TradeOrder::UserId)
              .to(User::Table, User::Id),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_trade_orders_wallet")
              .from(TradeOrder::Table, TradeOrder::WalletId)
              .to(Wallet::Table, Wallet::Id),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_trade_orders_token")
              .from(TradeOrder::Table, TradeOrder::TokenId)
              .to(Token::Table, Token::Id),
          )
          .to_owned(),
      )
      .await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(TradeOrder::Table).to_owned())
      .await?;
    Ok(())
  }
}

#[derive(DeriveIden)]
pub enum TradeOrder {
  #[sea_orm(iden = "trade_orders")]
  Table,
  Id,
  UserId,
  WalletId,
  TokenId,
  ReferencePrice,
  TargetPrice,
  TargetPercentage,
  ContractAddress,
  Strategy,
  Active,
  CreatedBy,
  Metadata,
  CreatedAt,
  UpdatedAt,
}
