// use actix_web::{get, route, web, Error, HttpResponse, Responder};
// use juniper::http::{graphiql::graphiql_source, GraphQLRequest, GraphQLResponse};

// use crate::db::DBConnection;

// mod schemas;

// use schemas::root::{create_schema, Context, Schema};

// /// GraphQL endpoint
// #[route("/graphql", method = "GET", method = "POST")]
// pub async fn graphql(
//     db_connection: web::Data<DBConnection>,
//     schema: web::Data<Schema>,
//     data: web::Json<GraphQLRequest>,
// ) -> Result<HttpResponse, Error> {
//     let ctx = Context {
//         db_connection: db_connection.get_ref().clone(),
//     };

//     let res = data.execute(&schema, &ctx).await;

//     Ok(HttpResponse::Ok().json(res))
// }

use actix_web::{
    get, route,
    web::{self},
    HttpResponse, Responder,
};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use juniper::{EmptySubscription, FieldResult, RootNode};

// pub struct Context {
//     pub db: String,
// }

// impl juniper::Context for Context {}

#[derive(GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};

// use crate::db::DBConnection;

#[derive(GraphQLObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct Human {
    id: String,
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct NewHuman {
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

pub struct QueryRoot;

// #[juniper::graphql_object]
#[juniper::graphql_object]
impl QueryRoot {
    fn human(_id: String) -> FieldResult<Human> {
        Ok(Human {
            id: "1234".to_owned(),
            name: "Luke".to_owned(),
            appears_in: vec![Episode::NewHope],
            home_planet: "Mars".to_owned(),
        })
    }
}

pub struct MutationRoot;

// #[juniper::graphql_object]
#[juniper::graphql_object]
impl MutationRoot {
    fn create_human(new_human: NewHuman) -> FieldResult<Human> {
        Ok(Human {
            id: "1234".to_owned(),
            name: new_human.name,
            appears_in: new_human.appears_in,
            home_planet: new_human.home_planet,
        })
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot, EmptySubscription::new())
}

#[route("/graphql", method = "GET", method = "POST")]
async fn graphql(
    // _db: web::Data<DBConnection>,
    st: web::Data<Schema>,
    data: web::Json<GraphQLRequest>,
) -> impl Responder {
    // let ctx = Context {
    //     db: "db.get_ref().clone()".to_string(),
    // };

    let res = data.execute(&st, &()).await;
    HttpResponse::Ok().json(res)
}

/// GraphiQL UI
#[get("/graphiql")]
async fn graphql_playground() -> impl Responder {
    web::Html::new(graphiql_source("/graphql", None))
}

pub fn graphql_server(config: &mut web::ServiceConfig) {
    config
        .app_data(web::Data::new(create_schema()))
        .service(graphql)
        .service(graphql_playground);
}
