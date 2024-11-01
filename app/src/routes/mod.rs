// use crate::routes::webhook::raydium_token_event;
pub mod webhook;

pub fn init_routes(cfg: &mut actix_web::web::ServiceConfig) {
  cfg.service(webhook::raydium_token_event);
}
