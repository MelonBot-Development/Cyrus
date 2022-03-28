use actix_web::{
    HttpRequest,
    Responder,
    Scope,
    post,
    web
};
use actix_web::http::header::AUTHORIZATION;
use log::error;
use redis_async::resp_array;
use serde::{
    Deserialize,
    Serialize
};
use serde_json::json;

use crate::{
    HttpResponse,
    Redis
};
use crate::config::ApiConfig;

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum TopGGVoteType {
    Test,
    Upvote,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct TopGGVote {
    bot: String,
    user: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    vote_type: TopGGVoteType,
    query: Option<String>,
    #[serde(default)]
    is_weekend: bool,
}

pub fn voting() -> Scope {
    web::scope("/voting")
        .service(post_top_gg_vote)
}

#[post("/top-gg-vote")]
async fn post_top_gg_vote(
    req: HttpRequest,
    config: web::Data<ApiConfig>,
    webhook: web::Data<Webhook>,
    redis: web::Data<Redis>,
    payload: web::Json<TopGGVote>,
) -> impl Responder {
    let auth = match req.headers().get(AUTHORIZATION) {
        Some(value) => match value.to_str() {
            Err(e) => {
                error!("Error occurred while validating top.gg auth: {}", e);

                return HttpResponse::BadRequest()
                    .json(json!({ "message": "Unable to validate authorization", "success": false }));
            }
            Ok(value) => value.to_strings()
        },
        _ => return HttpResponse::Forbidden()
            .json(json!({ "message": "No authorization was provided", "success": false }))
    };

    if config.auth.top_gg != auth {
        return HttpResponse::Unauthorized()
            .json(json!({ "message": "Invalid authorization was provided", "success": false }));
    }

    let key_type = if payload.vote_type == TopGGVoteType::Test { "test" } else { "upvote" };
    let key = format!("top_gg.{}:{}.{}", key_type, payload.bot, payload.user);

    redis
        .get()
        .send_and_forget(resp_array!["SET", key, serde_json::to_string(&payload).unwrap(), "Ex", "720"]);

    HttpResponse::Ok().json(json!({ "message": "thanks :3", "success": true }))
}