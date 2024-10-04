use crate::db::DBConnection;
use juniper::{graphql_object, EmptySubscription, RootNode};

pub struct Context {
    pub db_connection: DBConnection,
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[graphql_object(Context = Context)]
impl QueryRoot {
    #[graphql(description = "Say hello")]
    fn hello() -> String {
        return "hello world".to_string();
    }
}

pub struct MutationRoot;

#[graphql_object(Context = Context)]
impl MutationRoot {
    fn hello(_name: String) -> String {
        return "hello world".to_string();
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}
