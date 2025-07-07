use okapi::openapi3::OpenApi;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sqlx::MySqlPool;

use crate::dto::rss::response::{RssChannelResponseDto, RssItemResponseDto};
use crate::dto::search::request::SearchRequestDto;
use crate::service::rss::{channel_service, item_service};
use crate::EmbeddingService;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_rss_list, get_channel_list ]
}

/// # get_rss_list
///
/// Returns a list of RSS items based on the search request.
#[openapi(tag = "Search")]
#[get("/item?<request..>")]
pub async fn get_rss_list(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    request: SearchRequestDto,
) -> Result<Json<Vec<RssItemResponseDto>>, Status> {
    if request.search_value.is_none() {
        return Err(Status::BadRequest);
    }

    match item_service::get_rss_list(pool, model, request).await {
        Ok(rss_list) => Ok(Json(rss_list)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # get_channel_list
///
/// Returns a list of RSS channels based on the search request.
#[openapi(tag = "Search")]
#[get("/channels?<request..>")]
pub async fn get_channel_list(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    request: SearchRequestDto,
) -> Result<Json<Vec<RssChannelResponseDto>>, Status> {
    if request.search_value.is_none() {
        return Err(Status::BadRequest);
    }

    match channel_service::get_channel_list(pool, model, request).await {
        Ok(channel_list) => Ok(Json(channel_list)),
        Err(_) => Err(Status::InternalServerError),
    }
}
