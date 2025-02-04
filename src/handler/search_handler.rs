use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use sqlx::MySqlPool;

use crate::model::rss::{RssChannel, RssItem};
use crate::model::search::SearchRequest;
use crate::service::user_service;

#[post("/search/rss", data = "<request>")]
pub async fn get_rss_list(
    pool: &State<MySqlPool>,
    request: Json<SearchRequest>,
) -> Result<Json<Vec<RssItem>>, Status> {
    if request.search_value.is_none() {
        return Err(Status::BadRequest);
    }

    match user_service::get_rss_list(pool, request.into_inner()).await {
        Ok(rss_list) => Ok(Json(rss_list)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/search/channel", data = "<request>")]
pub async fn get_channel_list(
    pool: &State<MySqlPool>,
    request: Json<SearchRequest>,
) -> Result<Json<Vec<RssChannel>>, Status> {
    if request.search_value.is_none() {
        return Err(Status::BadRequest);
    }

    match user_service::get_channel_list(pool, request.into_inner()).await {
        Ok(channel_list) => Ok(Json(channel_list)),
        Err(_) => {
            // logging
            Err(Status::InternalServerError)
        }
    }
}
