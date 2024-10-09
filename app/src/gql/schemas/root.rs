use entity::users;
use juniper::{EmptySubscription, FieldResult, RootNode};
use sea_orm::{DatabaseConnection, EntityTrait};

use ::entity::prelude::*;

pub struct Context {
  pub db: DatabaseConnection,
  pub user: Option<users::Model>,
}
impl juniper::Context for Context {}

#[derive(GraphQLEnum)]
enum Episode {
  NewHope,
  Empire,
  Jedi,
}

use juniper::{GraphQLEnum, GraphQLInputObject};

use super::user::{UserMutation, UserQuery};

#[derive(GraphQLInputObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct NewHuman {
  name: String,
  appears_in: Vec<Episode>,
  home_planet: String,
}

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
}

pub struct Mutation;

// #[juniper::graphql_object]
#[juniper::graphql_object(Context = Context)]
impl Mutation {
  fn user() -> UserMutation {
    UserMutation
  }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
  Schema::new(Query {}, Mutation, EmptySubscription::new())
}
