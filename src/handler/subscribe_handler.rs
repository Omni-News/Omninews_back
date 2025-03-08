use rocket::http::Status;
use rocket::{serde::json::Json, State};
use sqlx::MySqlPool;

use crate::model::rss::RssItem;
use crate::model::subscribe::ChannelLink;
use crate::service::bookmark_service;

#[post("/subscribe/items", data = "<channel_links>")]
pub async fn get_subscribe_items(
    pool: &State<MySqlPool>,
    channel_links: Json<Vec<ChannelLink>>,
) -> Result<Json<Vec<RssItem>>, Status> {
    match bookmark_service::get_subscribe_items(pool, channel_links.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
