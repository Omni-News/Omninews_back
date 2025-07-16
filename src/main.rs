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
use handler::error_handler::error_catchers;
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

    tokio::spawn(async move {
        start_scheduler(&pool_scheduler).await;
    });

    let embedding_service = EmbeddingService::new();

    let exempt_paths = vec![
        "/user/login".to_string(),
        "/rapidoc/".to_string(),
        "/swagger-ui/".to_string(),
        // TODO 이거 잘 만들어보기. /aa , /aa/ 이렇게 구분. 끝 슬래시면 하위 포함
        "/".to_string(),
    ];
    let mut rocket = rocket::build()
        .manage(pool)
        .manage(embedding_service)
        .manage(AuthCache::new())
        .attach(AuthMiddleware::new(exempt_paths, pool_middleware))
        .mount("/rapidoc/", create_rapidoc())
        .mount("/swagger-ui/", create_swagger_ui())
        .attach(CORS)
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

async fn start_scheduler(pool: &MySqlPool) {
    use scheduler::{annoy_scheduler::*, news_scheduler::*};

    tokio::join!(
        delete_old_news_scheduler(pool),
        fetch_news_scheduler(pool),
        save_annoy_scheduler(pool)
    );
}

// TODO 테스트 목 데이터는 DTO파일에서 구현 가능, user에서 한개 만들어놨으니까 그거 보고 만들기,
// 응답도 만들어보기.

// TODO 현재 swerver가 localhost랑 example로 되어있는데, example를 서비스중인 ip로 바꿔서
// 테스트해보기
