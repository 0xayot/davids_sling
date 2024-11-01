use actix_web::{post, web::Json, HttpRequest, HttpResponse, Responder};

use crate::utils::{auth::authorize_davids_sight, event::raydium::handle_token_created_event};

#[post("/webhooks/raydium_token_event")]
async fn raydium_token_event(
  req: HttpRequest,
  body: Json<crate::utils::event::raydium::RaydiumTokenEvent>,
) -> impl Responder {
  if !authorize_davids_sight(&req) {
    return HttpResponse::NotFound().finish();
  }

  println!("Request Body: {:?}", body.0);

  // call handle raydium token event

  tokio::spawn(async {
    handle_token_created_event(body.into_inner()).await;
  });

  HttpResponse::Ok().finish()
}
