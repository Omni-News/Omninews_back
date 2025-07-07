use okapi::openapi3::OpenApi;
use rocket_okapi::{get_nested_endpoints_and_docs, settings::OpenApiSettings};

pub mod error_handler;
pub mod folder_handler;
pub mod health_handler;
pub mod news_handler;
pub mod rss_handler;
pub mod search_handler;
pub mod subscription_handler;
pub mod user_handler;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    // TODO 이거 api경로 반영안됨. 전부 수정필요.수정하면서 예시 값같은거도 넣어주기.
    get_nested_endpoints_and_docs! {
        "/user" => user_handler::get_routes_and_docs(settings),
        "/rss" => rss_handler::get_routes_and_docs(settings),
        "/news" => news_handler::get_routes_and_docs(settings),
        "/search" => search_handler::get_routes_and_docs(settings),
        "/subscription" => subscription_handler::get_routes_and_docs(settings),
        "/folder" => folder_handler::get_routes_and_docs(settings),
        "/health" => health_handler::get_routes_and_docs(settings),
    }
}
