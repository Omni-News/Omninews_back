use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sqlx::MySqlPool;

use crate::{
    config::webdriver::DriverPool,
    dto::{premium::rss::request::RssGenerateRequestDto, rss::response::RssChannelResponseDto},
    service::premium::premium_rss_service,
    utils::embedding_util::EmbeddingService,
};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: rss_generate]
}

#[openapi(tag = "Premium RSS Generation API")]
#[post("/premium/rss/generate", data = "<data>")]
/// # RSS Generation API
///
/// 사이트 종류와 링크를 입력받아 RSS 피드를 생성합니다.
///
/// ### `link`: Rss 피드를 생성할 사이트의 링크
/// ### `kind`: 사이트 종류 (예: "Instagram", "Medium", "Naver" 등)
pub async fn rss_generate(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
    driver_pool: &State<DriverPool>,
    data: Json<RssGenerateRequestDto>,
) -> Result<Json<RssChannelResponseDto>, Status> {
    match premium_rss_service::generate_rss(pool, embedding_service, driver_pool, data.into_inner())
        .await
    {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
