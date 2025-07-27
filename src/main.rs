#[macro_use]
extern crate rocket;

mod auth_middleware;
mod config;
mod db_util;
mod dto;
mod global;
mod handler;
mod model;
mod repository;
mod scheduler;
mod service;
mod utils;

use auth_middleware::{AuthCache, AuthMiddleware, CORS};
use config::{
    env, logging, openapi::custom_openapi_spec, rapidoc::create_rapidoc, swagger::create_swagger_ui,
};
use handler::{config_handler::options_handler, error_handler::error_catchers};
use rocket_okapi::mount_endpoints_and_merged_docs;
use sqlx::MySqlPool;
use utils::embedding_util::EmbeddingService;

pub const CURRENT_VERSION: &str = "v1";

#[launch]
async fn rocket() -> _ {
    env::load_env();
    logging::load_logger();

    let pool = db_util::create_pool().await;
    let pool_scheduler = pool.clone();
    let pool_middleware = pool.clone();

    let embedding_service = EmbeddingService::new();
    let embedding_service_scheduler = embedding_service.clone();

    tokio::spawn(async move {
        start_scheduler(&pool_scheduler, &embedding_service_scheduler).await;
    });

    let exempt_paths = vec![
        // omninews
        "/v1/api/user/login".to_string(),
        "/v1/api/user/apple/login".to_string(),
        "/v1/api/user/refresh-token".to_string(),
        "/v1/api/health".to_string(),
        // openapi
        "/rapidoc/".to_string(),
        "/swagger-ui/".to_string(),
        format!("/{}/openapi.json", CURRENT_VERSION).to_owned(),
    ];
    let mut rocket = rocket::build()
        .manage(pool)
        .manage(embedding_service)
        .manage(AuthCache::new())
        .attach(CORS)
        .attach(AuthMiddleware::new(exempt_paths, pool_middleware))
        .mount("/rapidoc/", create_rapidoc())
        .mount("/swagger-ui/", create_swagger_ui())
        .mount("/", routes![options_handler])
        .register("/", error_catchers());

    let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();
    let custom_route_spec = (vec![], custom_openapi_spec());

    mount_endpoints_and_merged_docs! {
        rocket, format!("/{}", CURRENT_VERSION).to_owned(), openapi_settings,
        "/external" => custom_route_spec,
        "/api" => handler::get_routes_and_docs(&openapi_settings),
    };

    rocket
}

async fn start_scheduler(pool: &MySqlPool, embedding_service: &EmbeddingService) {
    use scheduler::{
        annoy_scheduler::*, news_scheduler::*, rss_info_update_scheduler::*,
        rss_notification_scheduler::*,
    };

    tokio::join!(
        delete_old_news_scheduler(pool),
        fetch_news_scheduler(pool),
        save_annoy_scheduler(pool),
        rss_notification_scheduler(pool, embedding_service),
        rss_info_update_scheduler(pool),
    );
}

// TODO 현재 swerver가 localhost랑 example로 되어있는데, example를 서비스중인 ip로 바꿔서
// 테스트해보기
