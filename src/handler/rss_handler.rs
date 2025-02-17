use rocket::serde::json::Json;
use rocket::{http::Status, State};
use sqlx::MySqlPool;

use crate::model::rss::RssLink;
use crate::service::rss::channel_service;

#[post("/rss", data = "<rss_link>")]
pub async fn create_rss(
    pool: &State<MySqlPool>,
    rss_link: Json<RssLink>,
) -> Result<Json<i32>, Status> {
    if rss_link.link.is_empty() {
        return Err(Status::BadRequest);
    }

    match channel_service::create_rss_and_morpheme(pool, rss_link.into_inner()).await {
        Ok(channel_id) => Ok(Json(channel_id)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/rss/all", data = "<rss_links>")]
pub async fn create_rss_all(
    pool: &State<MySqlPool>,
    rss_links: Json<Vec<RssLink>>,
) -> Result<Json<bool>, Status> {
    if rss_links.is_empty() {
        return Err(Status::BadRequest);
    }

    match channel_service::create_rss_all(pool, rss_links.into_inner()).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(Status::InternalServerError),
    }
}
