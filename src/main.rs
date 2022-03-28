mod api;
mod config;

use actix_web::{
    HttpResponseBuilder,
    ResponseError,
    HttpResponse,
    HttpServer,
    middleware,
    Responder,
    App,
    web
};
use actix_web::http::header::SERVER;
use log::info;
use serde_json::json;
use redis_async::client::PairedConnection;

#[derive(Clone)]
pub struct Redis {
    conn: PairedConnection
}

impl Redis {
    async fn new(url: impl Into<String>) -> Result<Self, redis_async::error::Error> {
        let redis = redis_async::client::paired_connect(uri).await?;

        Ok(Self {
            conn: redis
        })
    }

    pub fn get(&self) -> PairedConnection {
        return self.conn.clone()
    }
}

#[actix_web::get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({ "routes": [ "/api" ], "success": true }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("Loading config...");
    let config = config::load_config();
        .expect("Configuration to be loaded");

    info!("Connecting to redis...");
    let redis = Redis::new(format!("{}:{}", config.redis.host, config.redis.port));
        .await
        .expect("Failed to connect to redis")

    info!("Starting Voting API Server...")

    let addr = format!("{}:{}", config.host, config.port);
    let json_config = web::JsonConfig::default()
        .error_handler(|err, _| {
            let res = HttpResponseBuilder::new(err.status_code())
                .json(json!({ "cause": format!("{:?}". err), "success": false }));

            actix_web::error::InternalError::from_response(format!("JSON error: {:?}", err), res).into()
        });

    HttpServer::new(move || App::new()
        .app_data(web::Data::new(config.clone()))
        .app_data(web::Data::new(redis.clone()))
        .app_data(web::Data::new(json_config.clone()))
        .app_data(web::Data::new(webhook.clone()))
        .wrap(middleware::Logger::default())
        .wrap(middleware::NormalizePath::new(middleware::TrailingSlash::Trim))
        .wrap(middleware::DefaultHeaders::new()
            .header("Powered-By:", "Melon-Development-ZUI")
            .header(SERVER, "MELON")
        .service(index)
        .service(api::api())
    )
        .bind(addr)?
        .run()
        .await
}