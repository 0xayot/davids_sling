use sea_orm_migration::{prelude::*, schema::pk_auto};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(TokenPrice::Table)
          .if_not_exists()
          .col(pk_auto(TokenPrice::Id))
          .col(
            ColumnDef::new(TokenPrice::ContractAddress)
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(TokenPrice::Chain).string().not_null())
          .col(ColumnDef::new(TokenPrice::Name).string())
          .col(ColumnDef::new(TokenPrice::Price).float().default(0.0))
          .col(ColumnDef::new(TokenPrice::PriceNative).float().default(0.0))
          .col(
            ColumnDef::new(TokenPrice::CreatedAt)
              .timestamp()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .col(
            ColumnDef::new(TokenPrice::UpdatedAt)
              .timestamp()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .to_owned(),
      )
      .await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(TokenPrice::Table).to_owned())
      .await?;
    Ok(())
  }
}

#[derive(DeriveIden)]
enum TokenPrice {
  #[sea_orm(iden = "token_prices")]
  Table,
  Id,
  ContractAddress,
  Chain,
  Name,
  Price,
  PriceNative,
  CreatedAt,
  UpdatedAt,
}
