//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  #[sea_orm(unique)]
  pub email: Option<String>,
  #[sea_orm(unique)]
  pub tg_id: String,
  pub tg_token: Option<String>,
  pub encrypted_password: String,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::trade_orders::Entity")]
  TradeOrders,
  #[sea_orm(has_many = "super::wallets::Entity")]
  Wallets,
}

impl Related<super::trade_orders::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::TradeOrders.def()
  }
}

impl Related<super::wallets::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Wallets.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
