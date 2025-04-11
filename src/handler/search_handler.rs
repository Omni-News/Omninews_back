use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use sqlx::MySqlPool;

use crate::model::rss::{RssChannel, RssItem};
use crate::model::search::SearchRequest;
use crate::service::rss::{channel_service, item_service};
use crate::EmbeddingService;

#[get("/search/rss?<request..>")]
pub async fn get_rss_list(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    request: SearchRequest,
) -> Result<Json<Vec<RssItem>>, Status> {
    if request.search_value.is_none() {
        return Err(Status::BadRequest);
    }

    match item_service::get_rss_list(pool, model, request).await {
        Ok(rss_list) => Ok(Json(rss_list)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/search/channel?<request..>")]
pub async fn get_channel_list(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    request: SearchRequest,
) -> Result<Json<Vec<RssChannel>>, Status> {
    if request.search_value.is_none() {
        return Err(Status::BadRequest);
    }

    match channel_service::get_channel_list(pool, model, request).await {
        Ok(channel_list) => Ok(Json(channel_list)),
        Err(_) => Err(Status::InternalServerError),
    }
}
