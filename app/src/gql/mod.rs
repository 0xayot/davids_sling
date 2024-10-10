use crate::{db, utils::auth::req_user};
use actix_web::{get, route, web, HttpRequest, HttpResponse, Responder};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};

mod schemas;

use schemas::root::{create_schema, Context, Schema};

#[route("/graphql", method = "GET", method = "POST")]
async fn graphql(
  req: HttpRequest,
  st: web::Data<Schema>,
  data: web::Json<GraphQLRequest>,
) -> impl Responder {
  let db = db::connect_db()
    .await
    .expect("Failed to connect to the database");

  let user = req_user(req, &db).await;

  let ctx = Context { db, user };

  let res = data.execute(&st, &ctx).await;
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
