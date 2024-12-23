//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "onchain_transactions")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  pub user_id: i32,
  pub wallet_id: i32,
  pub transaction_hash: Option<String>,
  pub chain: String,
  pub source: Option<String>,
  pub status: Option<String>,
  pub r#type: Option<String>,
  #[sea_orm(column_type = "Float", nullable)]
  pub value_native: Option<f32>,
  #[sea_orm(column_type = "Float", nullable)]
  pub value_usd: Option<f32>,
  pub from_token: Option<String>,
  pub to_token: Option<String>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::UserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Users,
  #[sea_orm(
    belongs_to = "super::wallets::Entity",
    from = "Column::WalletId",
    to = "super::wallets::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Wallets,
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Users.def()
  }
}

impl Related<super::wallets::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Wallets.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
