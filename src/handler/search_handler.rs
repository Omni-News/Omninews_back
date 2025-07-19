use okapi::openapi3::OpenApi;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sqlx::MySqlPool;

use crate::auth_middleware::AuthenticatedUser;
use crate::dto::rss::response::{RssChannelResponseDto, RssItemResponseDto};
use crate::dto::search::request::SearchRequestDto;
use crate::service::rss::{channel_service, item_service};
use crate::EmbeddingService;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_rss_list, get_channel_list ]
}

/// # 검색 내용으로 RSS 목록 조회 API
///
/// 검색 내용에 해당하는 RSS 아이템 목록을 반환합니다.
///
/// ### `search_value` : 검색어 (예: "AI", "경제")
///
/// ### `search_type` : 검색 타입 (예: "Accuracy", "Popularity", "Latest")
///
#[openapi(tag = "검색 API")]
#[get("/search/item?<request..>")]
pub async fn get_rss_list(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    request: SearchRequestDto,
    _auth: AuthenticatedUser,
) -> Result<Json<Vec<RssItemResponseDto>>, Status> {
    if request.search_value.is_none() {
        return Err(Status::BadRequest);
    }

    match item_service::get_rss_list(pool, model, request).await {
        Ok(rss_list) => Ok(Json(rss_list)),
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
#[openapi(tag = "검색 API")]
#[get("/search/channels?<request..>")]
pub async fn get_channel_list(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    request: SearchRequestDto,
    _auth: AuthenticatedUser,
) -> Result<Json<Vec<RssChannelResponseDto>>, Status> {
    if request.search_value.is_none() {
        return Err(Status::BadRequest);
    }

    match channel_service::get_channel_list(pool, model, request).await {
        Ok(channel_list) => Ok(Json(channel_list)),
        Err(_) => Err(Status::InternalServerError),
    }
}
