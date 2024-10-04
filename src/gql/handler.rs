use actix_web::{get, route, web, Error, HttpResponse, Responder};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};

// Adjust the import to reference the correct path to `Schema` and `Context`
use crate::gql::schemas::root::{Context, Schema}; // Make sure to use the correct paths

use crate::db::DBConnection;

use super::schemas::root::create_schema;

/// GraphQL endpoint
#[route("/graphql", method = "GET", method = "POST")]
pub async fn graphql(
    db_connection: web::Data<DBConnection>,
    schema: web::Data<Schema>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let ctx = Context {
        db_connection: db_connection.get_ref().to_owned(),
    };

    let res = data.execute(&schema, &ctx).await;

    Ok(HttpResponse::Ok().json(res))
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
