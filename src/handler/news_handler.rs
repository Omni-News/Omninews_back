use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sqlx::MySqlPool;

use crate::{
    dto::news::{request::NewsRequestDto, response::NewsResponseDto},
    service::news_service,
};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_news]
}

/// # 뉴스 카테고리별 조회 API
///
/// 제공된 카테고리에 해당하는 뉴스 목록을 반환합니다.
///
/// ### `category` : 뉴스 카테고리 (예: "정치", "경제", "사회", "생활/문화", "세계", "IT/과학")
///
#[openapi(tag = "뉴스 API")]
#[get("/news?<value..>")]
pub async fn get_news(
    pool: &State<MySqlPool>,
    value: NewsRequestDto,
) -> Result<Json<Vec<NewsResponseDto>>, Status> {
    match news_service::get_news(pool, value).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
