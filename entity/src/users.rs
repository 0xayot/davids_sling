//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i32,
  pub email: Option<String>,
  pub tg_id: i32,
  pub encrypted_password: String,
  pub created_at: DateTime,
  pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::wallets::Entity")]
  Wallets,
}

impl Related<super::wallets::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Wallets.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
