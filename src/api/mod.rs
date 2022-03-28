mod voting;

use actix_web::{
    HttpResponse,
    Responder,
    Scope,
    get,
    web
};
use serde_json::json;

pub fn api() -> Scope {
    web::scope("/api")
        .service(index)
        .service(voting::voting())
}

#[get("")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({ "message": "hello from the other side xD", "success": true }))
}