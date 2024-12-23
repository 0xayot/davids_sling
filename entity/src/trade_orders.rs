//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "trade_orders")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  pub user_id: i32,
  pub wallet_id: i32,
  pub token_id: i32,
  #[sea_orm(column_type = "Float")]
  pub reference_price: f32,
  #[sea_orm(column_type = "Float")]
  pub target_price: f32,
  #[sea_orm(column_type = "Float")]
  pub target_percentage: f32,
  pub contract_address: String,
  pub strategy: String,
  pub active: bool,
  pub created_by: String,
  pub metadata: Option<Json>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::tokens::Entity",
    from = "Column::TokenId",
    to = "super::tokens::Column::Id",
    on_update = "NoAction",
    on_delete = "NoAction"
  )]
  Tokens,
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

impl Related<super::tokens::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Tokens.def()
  }
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
