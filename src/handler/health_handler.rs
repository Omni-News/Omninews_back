use okapi::openapi3::OpenApi;
use rocket::serde::json::Json;
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: health_check]
}

/// # health_check
///
/// Returns a simple "OK" response if service is running.
#[openapi(tag = "Health")]
#[get("/health")]
pub async fn health_check() -> Json<&'static str> {
    Json("Ok")
}
