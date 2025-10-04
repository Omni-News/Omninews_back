use okapi::openapi3::OpenApi;
use rocket_okapi::{get_nested_endpoints_and_docs, settings::OpenApiSettings};

pub mod apple_server_notification;
pub mod config_handler;
pub mod error_handler;
pub mod folder_handler;
pub mod health_handler;
pub mod index;
pub mod news_handler;
pub mod omninews_subscription_handler;
pub mod premium;
pub mod rss_handler;
pub mod search_handler;
pub mod subscription_handler;
pub mod user_handler;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    get_nested_endpoints_and_docs! {
        "/" => user_handler::get_routes_and_docs(settings),
        "/" => rss_handler::get_routes_and_docs(settings),
        "/" => news_handler::get_routes_and_docs(settings),
        "/" => search_handler::get_routes_and_docs(settings),
        "/" => subscription_handler::get_routes_and_docs(settings),
        "/" => folder_handler::get_routes_and_docs(settings),
        "/" => health_handler::get_routes_and_docs(settings),
        "/" => omninews_subscription_handler::get_routes_and_docs(settings),

        // premium
        "/" => premium::rss_generate::get_routes_and_docs(settings),
        "/" => premium::rss_summary::get_routes_and_docs(settings),
        "/" => apple_server_notification::notification::get_routes_and_docs(settings),

    }
}
