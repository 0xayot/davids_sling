//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tokens")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  #[sea_orm(unique)]
  pub contract_address: String,
  #[sea_orm(unique)]
  pub token_public_key: Option<String>,
  pub chain: String,
  pub decimals: Option<i32>,
  pub name: Option<String>,
  pub metadata: Option<Json>,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::trade_orders::Entity")]
  TradeOrders,
}

impl Related<super::trade_orders::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::TradeOrders.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}