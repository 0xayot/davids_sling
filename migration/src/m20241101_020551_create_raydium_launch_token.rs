use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(RaydiumTokenLaunch::Table)
          .if_not_exists()
          .col(pk_auto(RaydiumTokenLaunch::Id))
          .col(
            ColumnDef::new(RaydiumTokenLaunch::ContractAddress)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(RaydiumTokenLaunch::CreatorAddress)
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(RaydiumTokenLaunch::Evaluation).string())
          .col(ColumnDef::new(RaydiumTokenLaunch::LaunchClass).string())
          .col(
            ColumnDef::new(RaydiumTokenLaunch::LaunchLiquidity)
              .float()
              .not_null(),
          )
          .col(
            ColumnDef::new(RaydiumTokenLaunch::LaunchLiquidityUsd)
              .float()
              .not_null(),
          )
          .col(ColumnDef::new(RaydiumTokenLaunch::LaunchPriceUsd).float())
          .col(ColumnDef::new(RaydiumTokenLaunch::RuggedAt).integer())
          .col(ColumnDef::new(RaydiumTokenLaunch::Lifespan).integer())
          .col(ColumnDef::new(RaydiumTokenLaunch::Meta).json())
          .col(ColumnDef::new(RaydiumTokenLaunch::HasBoost).boolean())
          .col(
            ColumnDef::new(RaydiumTokenLaunch::CreatedAt)
              .timestamp()
              .default(Expr::current_timestamp())
              .not_null(),
          )
          .col(
            ColumnDef::new(RaydiumTokenLaunch::UpdatedAt)
              .timestamp()
              .default(Expr::current_timestamp())
              .not_null(),
          )
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(RaydiumTokenLaunch::Table).to_owned())
      .await?;

    Ok(())
  }
}

#[derive(DeriveIden)]
pub enum RaydiumTokenLaunch {
  #[sea_orm(iden = "raydium_token_launches")]
  Table,
  Id,
  ContractAddress,
  CreatorAddress,
  Evaluation,
  LaunchClass,
  LaunchLiquidity,
  LaunchLiquidityUsd,
  LaunchPriceUsd,
  RuggedAt,
  Lifespan,
  Meta,
  HasBoost,
  CreatedAt,
  UpdatedAt,
}
