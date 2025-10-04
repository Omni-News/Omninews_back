#[macro_use]
extern crate rocket;

mod auth_middleware;
mod config;
mod dto;
mod handler;
mod model;
mod repository;
mod service;
mod utils;

use auth_middleware::{AuthCache, AuthMiddleware, CORS};
use config::{
    env, logging, openapi::custom_openapi_spec, rapidoc::create_rapidoc, swagger::create_swagger_ui,
};
use handler::{config_handler::options_handler, error_handler::error_catchers};
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
use rocket_okapi::mount_endpoints_and_merged_docs;
use utils::embedding_util::EmbeddingService;

use crate::{
    config::webdriver::{DriverPool, DriverPoolConfig},
    handler::index::{app_ads, index},
    utils::db_util,
};

pub const CURRENT_VERSION: &str = "v1";

#[launch]
async fn rocket() -> _ {
    env::load_env();
    logging::load_logger();
    // setup the webdriver pool
    let dp_cfg = DriverPoolConfig::default();

    let driver_pool = DriverPool::new(dp_cfg);

    let pool = db_util::create_pool().await;
    let pool_middleware = pool.clone();

    let embedding_service = EmbeddingService::new();

    let exempt_paths = vec![
        // omninews
        "/v1/api/user/login".to_string(),
        "/v1/api/user/apple/login".to_string(),
        "/v1/api/user/refresh-token".to_string(),
        "/v1/api/health".to_string(),
        // openapi
        "/rapidoc/".to_string(),
        "/swagger-ui/".to_string(),
        // marketing
        "/".to_string(),
        "/app-ads.txt".to_string(),
        format!("/{CURRENT_VERSION}/openapi.json").to_owned(),
    ];

    let mut rocket = rocket::build()
        .manage(pool)
        .manage(embedding_service)
        .manage(AuthCache::new())
        .manage(driver_pool)
        .attach(CORS)
        .attach(AuthMiddleware::new(exempt_paths, pool_middleware))
        .mount("/rapidoc/", create_rapidoc())
        .mount("/swagger-ui/", create_swagger_ui())
        //config
        .mount("/", routes![options_handler])
        //marketing
        .mount("/", routes![index, app_ads])
        .mount("/static", FileServer::from("static"))
        .attach(Template::fairing())
        .register("/", error_catchers());

    let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();
    let custom_route_spec = (vec![], custom_openapi_spec());

    mount_endpoints_and_merged_docs! {
        rocket, format!("/{CURRENT_VERSION}").to_owned(), openapi_settings,
        "/external" => custom_route_spec,
        "/api" => handler::get_routes_and_docs(&openapi_settings),
    };

    rocket
}
