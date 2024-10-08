use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(User::Table)
          .if_not_exists()
          .col(pk_auto(User::Id))
          .col(ColumnDef::new(User::Email).string().null())
          .col(ColumnDef::new(User::TgId).integer().not_null())
          .col(ColumnDef::new(User::EncryptedPassword).string().not_null())
          .col(
            ColumnDef::new(User::CreatedAt)
              .timestamp()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .col(
            ColumnDef::new(User::UpdatedAt)
              .timestamp()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(User::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
pub enum User {
  #[sea_orm(iden = "users")]
  Table,
  Id,
  Email,
  TgId,
  EncryptedPassword,
  CreatedAt,
  UpdatedAt,
}
