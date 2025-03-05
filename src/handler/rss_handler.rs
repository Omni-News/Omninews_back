use log::info;
use percent_encoding::percent_decode_str;
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use sqlx::MySqlPool;

use crate::model::rss::{NewRssChannel, RssChannel, RssItem, RssLink};
use crate::service::rss::{channel_service, item_service};

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

#[get("/rss/items?<channel_title>")]
pub async fn get_rss_item_by_channel_title(
    pool: &State<MySqlPool>,
    channel_title: String,
) -> Result<Json<Vec<RssItem>>, Status> {
    let decoded = percent_decode_str(channel_title.as_str())
        .decode_utf8()
        .expect("유효한 UTF-8 문자열이 아닙니다");

    match item_service::get_rss_item_by_channel_title(pool, decoded.to_string()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/rss/recommend/channel")]
pub async fn get_recommend_channel(
    pool: &State<MySqlPool>,
) -> Result<Json<Vec<RssChannel>>, Status> {
    match channel_service::get_recommend_channel(pool).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/rss/recommend/item")]
pub async fn get_recommend_item(pool: &State<MySqlPool>) -> Result<Json<Vec<RssItem>>, Status> {
    match item_service::get_recommend_item(pool).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/rss/preview?<rss_link>")]
pub async fn get_rss_preview(rss_link: String) -> Result<Json<RssChannel>, Status> {
    match channel_service::get_rss_preview(rss_link).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/rss/exist?<rss_link>")]
pub async fn is_rss_exist(pool: &State<MySqlPool>, rss_link: String) -> Result<Json<bool>, Status> {
    match channel_service::is_rss_exist(pool, rss_link).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
