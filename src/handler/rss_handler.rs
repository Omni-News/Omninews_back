use rocket::serde::json::Json;
use rocket::{http::Status, State};
use sqlx::MySqlPool;

use crate::model::RssLink;
use crate::service::rss_service;

#[post("/rss", data = "<rss_link>")]
pub async fn create_rss(
    pool: &State<MySqlPool>,
    rss_link: Json<RssLink>,
) -> Result<Status, Status> {
    match rss_service::create_rss(pool, rss_link.into_inner()).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}
