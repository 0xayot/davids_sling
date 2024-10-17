use sea_orm_migration::{prelude::*, schema::pk_auto};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Token::Table)
          .if_not_exists()
          .col(pk_auto(Token::Id))
          .col(
            ColumnDef::new(Token::ContractAddress)
              .string()
              .not_null()
              .unique_key(),
          )
          .col(ColumnDef::new(Token::Chain).string().not_null())
          .col(ColumnDef::new(Token::Decimals).integer())
          .col(ColumnDef::new(Token::Name).string())
          .col(ColumnDef::new(Token::Metadata).json())
          .col(
            ColumnDef::new(Token::CreatedAt)
              .date_time()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .col(
            ColumnDef::new(Token::UpdatedAt)
              .date_time()
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
      .drop_table(Table::drop().table(Token::Table).to_owned())
      .await?;
    Ok(())
  }
}

#[derive(DeriveIden)]
pub enum Token {
  #[sea_orm(iden = "tokens")]
  Table,
  Id,
  ContractAddress,
  Chain,
  Decimals,
  Name,
  Metadata,
  CreatedAt,
  UpdatedAt,
}
