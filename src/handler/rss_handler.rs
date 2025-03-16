use rocket::serde::json::Json;
use rocket::{http::Status, State};
use sqlx::MySqlPool;

use crate::model::rss::{RssChannel, RssItem, RssLink, UpdateRssRank};
use crate::service::rss::{channel_service, item_service};

#[get("/rss/channel?<rss_link>")]
pub async fn get_rss_channel_by_link(
    pool: &State<MySqlPool>,
    rss_link: String,
) -> Result<Json<RssChannel>, Status> {
    match channel_service::get_rss_channel_by_link(pool, rss_link).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/rss/items?<channel_link>")]
pub async fn get_rss_item_by_channel_link(
    pool: &State<MySqlPool>,
    channel_link: String,
) -> Result<Json<Vec<RssItem>>, Status> {
    match item_service::get_rss_item_by_channel_link(pool, channel_link).await {
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

#[post("/rss/channel/rank", data = "<update_rss_rank>")]
pub async fn update_rss_channel_rank(
    pool: &State<MySqlPool>,
    update_rss_rank: Json<UpdateRssRank>,
) -> Result<&str, Status> {
    let update_rss_rank = update_rss_rank.into_inner();

    match channel_service::update_rss_channel_rank(
        pool,
        update_rss_rank.rss_link,
        update_rss_rank.num,
    )
    .await
    {
        Ok(_) => Ok("Success"),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/rss/item/rank", data = "<update_rss_rank>")]
pub async fn update_rss_item_rank(
    pool: &State<MySqlPool>,
    update_rss_rank: Json<UpdateRssRank>,
) -> Result<&str, Status> {
    let update_rss_rank = update_rss_rank.into_inner();

    match item_service::update_rss_item_by_rank(pool, update_rss_rank.rss_link, update_rss_rank.num)
        .await
    {
        Ok(_) => Ok("Success"),
        Err(_) => Err(Status::InternalServerError),
    }
}
