use okapi::openapi3::OpenApi;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sqlx::MySqlPool;

use crate::auth_middleware::AuthenticatedUser;
use crate::dto::news::request::ApiNewsRequestDto;
use crate::dto::news::response::NewsApiResponseDto;
use crate::dto::search::request::SearchRequestDto;
use crate::dto::search::response::SearchResponseDto;
use crate::service::{channel_service, item_service, news_service};
use crate::EmbeddingService;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_rss_list, get_channel_list, get_news_by_api ]
}

/// # 검색 내용으로 RSS 목록 조회 API
///
/// 검색 내용에 해당하는 RSS 아이템 목록을 반환합니다.
///
/// ### `search_value` : 검색어 (예: "AI", "경제")
///
/// ### `search_type` : 검색 타입 (예: "Accuracy", "Popularity", "Latest")
///
/// ### `page_size` : 프론트에서 요청하는 페이지 번호, 반환 데이터는 기본 20개 (예 : 3, 10)
///
#[openapi(tag = "검색 API")]
#[get("/search/item?<request..>")]
pub async fn get_rss_list(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    request: SearchRequestDto,
    _auth: AuthenticatedUser,
) -> Result<Json<SearchResponseDto>, Status> {
    if request.search_value.is_none() {
        return Err(Status::BadRequest);
    }

    match item_service::get_rss_list(pool, model, request).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 검색 내용으로 RSS 채널 조회 API
///
/// 검색 내용에 해당하는 RSS 채널 목록을 반환합니다.
///
/// ### `search_value` : 검색어 (예: "AI", "경제")
///
/// ### `search_type` : 검색 타입 (예: "Accuracy", "Popularity", "Latest")
///
/// ### `page_size` : 프론트에서 요청하는 페이지 번호, 반환 데이터는 기본 20개 (예 : 3, 10)
///
#[openapi(tag = "검색 API")]
#[get("/search/channels?<request..>")]
pub async fn get_channel_list(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    request: SearchRequestDto,
    _auth: AuthenticatedUser,
) -> Result<Json<SearchResponseDto>, Status> {
    if request.search_value.is_none() {
        return Err(Status::BadRequest);
    }

    match channel_service::get_channel_list(pool, model, request).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 외부 API 뉴스 조회 API
///
/// 제공된 파라미터에 따라 외부 API에서 뉴스 목록을 검색하여 반환합니다.
///
/// ### `query` : 검색어 (예: "AI", "경제")
///
/// ### `display` : 뉴스 개수 (예: 1 ~ 1000)
///
/// ### `sort` : 정렬 기준 (예: sim => 정확도순, date => 날짜순)
///
#[openapi(tag = "검색 API")]
#[get("/search/news_api?<params..>")]
pub async fn get_news_by_api(
    params: ApiNewsRequestDto,
    _auth: AuthenticatedUser,
) -> Result<Json<Vec<NewsApiResponseDto>>, Status> {
    match news_service::get_news_by_api(params).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
