use okapi::openapi3::OpenApi;
use rocket::serde::json::Json;
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: health_check]
}

/// # 서비스 상태 확인 API
///
/// 서비스의 상태를 확인하는 간단한 헬스 체크 API입니다.
///
#[openapi(tag = "Health")]
#[get("/health")]
pub async fn health_check() -> Json<&'static str> {
    Json("Ok")
}
