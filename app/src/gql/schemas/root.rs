use entity::users;
use juniper::{EmptySubscription, FieldResult, RootNode};
use sea_orm::{DatabaseConnection, EntityTrait};

use ::entity::prelude::*;

pub struct Context {
  pub db: DatabaseConnection,
  pub user: Option<users::Model>,
}
impl juniper::Context for Context {}

use super::{
  user::{UserMutation, UserQuery},
  wallet::{WalletMutation, WalletQuery},
};

pub struct Query;

// #[juniper::graphql_object]
#[juniper::graphql_object(Context = Context)]
impl Query {
  #[graphql(description = "Say hello")]
  async fn hello(context: &Context) -> FieldResult<String> {
    let db = &context.db;

    let all = Users::find().all(db).await?;

    println!("{:#?}", all);

    Ok("Hello, world!".to_string())
  }
  fn user() -> UserQuery {
    UserQuery
  }
  fn wallet() -> WalletQuery {
    WalletQuery
  }
}

pub struct Mutation;

// #[juniper::graphql_object]
#[juniper::graphql_object(Context = Context)]
impl Mutation {
  fn user() -> UserMutation {
    UserMutation
  }
  fn wallet() -> WalletMutation {
    WalletMutation
  }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
  Schema::new(Query {}, Mutation, EmptySubscription::new())
}
