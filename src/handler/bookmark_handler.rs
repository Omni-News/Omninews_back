use rocket::http::Status;
use rocket::{serde::json::Json, State};
use sqlx::MySqlPool;

use crate::model::bookmark::ChannelLink;
use crate::model::rss::RssItem;
use crate::service::bookmark_service;

#[post("/bookmark/items", data = "<channel_links>")]
pub async fn get_bookmark_items(
    pool: &State<MySqlPool>,
    channel_links: Json<Vec<ChannelLink>>,
) -> Result<Json<Vec<RssItem>>, Status> {
    match bookmark_service::get_bookmark_items(pool, channel_links.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
