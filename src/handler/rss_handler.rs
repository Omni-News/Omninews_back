use okapi::openapi3::OpenApi;
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sqlx::MySqlPool;

use crate::dto::rss::request::{CreateRssRequestDto, UpdateRssRankRequestDto};
use crate::dto::rss::response::{RssChannelResponseDto, RssItemResponseDto};
use crate::service::rss::{channel_service, item_service};
use crate::EmbeddingService;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_channel_id_by_rss_link,
        get_rss_channel_by_id, get_rss_item_by_channel_id, get_recommend_channel,
        get_recommend_item, get_rss_preview, is_rss_exist, create_channel, create_rss_all,
        update_rss_item_rank]
}

/// # create_channel
///
/// Returns channel id if the RSS link is valid and the channel is created successfully.
#[openapi(tag = "Rss")]
#[post("/channel", data = "<link>")]
pub async fn create_channel(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    link: Json<CreateRssRequestDto>,
) -> Result<Json<i32>, Status> {
    if link.rss_link.is_empty() {
        return Err(Status::BadRequest);
    }

    match channel_service::create_rss_and_embedding(pool, model, link.into_inner()).await {
        Ok(channel_id) => Ok(Json(channel_id)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # create_rss_all
///
/// Returns `true` if all RSS links are processed successfully, otherwise `false`.
#[openapi(tag = "Rss")]
#[post("/all", data = "<links>")]
pub async fn create_rss_all(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    links: Json<Vec<CreateRssRequestDto>>,
) -> Result<Json<bool>, Status> {
    if links.is_empty() {
        return Err(Status::BadRequest);
    }

    match channel_service::create_rss_all(pool, model, links.into_inner()).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # get_channel_id_by_rss_link
///
/// Returns the channel ID associated with the provided RSS link.
#[openapi(tag = "Rss")]
#[get("/id?<channel_rss_link>")]
pub async fn get_channel_id_by_rss_link(
    pool: &State<MySqlPool>,
    channel_rss_link: String,
) -> Result<Json<i32>, Status> {
    match channel_service::find_rss_channel_by_rss_link(pool, channel_rss_link).await {
        Ok(res) => Ok(Json(res.channel_id.unwrap())),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # get_rss_channel_by_id
///
/// Returns the RSS channel details for the given channel ID.
#[openapi(tag = "Rss")]
#[get("/channel?<channel_id>")]
pub async fn get_rss_channel_by_id(
    pool: &State<MySqlPool>,
    channel_id: i32,
) -> Result<Json<RssChannelResponseDto>, Status> {
    match channel_service::find_rss_channel_by_id(pool, channel_id).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # get_rss_item_by_channel_id
///
/// Returns a list of RSS items associated with the specified channel ID.
#[openapi(tag = "Rss")]
#[get("/items?<channel_id>")]
pub async fn get_rss_item_by_channel_id(
    pool: &State<MySqlPool>,
    channel_id: i32,
) -> Result<Json<Vec<RssItemResponseDto>>, Status> {
    match item_service::get_rss_item_by_channel_id(pool, channel_id).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # get_recommend_channel
///
/// Returns a list of recommended RSS channels.
#[openapi(tag = "Rss")]
#[get("/recommend/channel")]
pub async fn get_recommend_channel(
    pool: &State<MySqlPool>,
) -> Result<Json<Vec<RssChannelResponseDto>>, Status> {
    match channel_service::get_recommend_channel(pool).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # get_recommend_item
///
/// Returns a list of recommended RSS items.
#[openapi(tag = "Rss")]
#[get("/recommend/item")]
pub async fn get_recommend_item(
    pool: &State<MySqlPool>,
) -> Result<Json<Vec<RssItemResponseDto>>, Status> {
    match item_service::get_recommend_item(pool).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # get_rss_preview
///
/// Returns a preview of the RSS channel based on the provided RSS link.
#[openapi(tag = "Rss")]
#[get("/preview?<rss_link>")]
pub async fn get_rss_preview(
    pool: &State<MySqlPool>,
    rss_link: String,
) -> Result<Json<RssChannelResponseDto>, Status> {
    match channel_service::get_rss_preview(pool, rss_link).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # is_rss_exist
///
/// Returns `true` if the RSS channel exists for the provided RSS link, otherwise `false`.
#[openapi(tag = "Rss")]
#[get("/exist?<rss_link>")]
pub async fn is_rss_exist(pool: &State<MySqlPool>, rss_link: String) -> Result<Json<bool>, Status> {
    match channel_service::is_channel_exist_by_link(pool, rss_link).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

//#[put("/rss/channel/rank", data = "<update_rss_rank>")]
//pub async fn update_rss_channel_rank(
//    pool: &State<MySqlPool>,
//    update_rss_rank: Json<UpdateChannelRank>,
//) -> Result<&str, Status> {
//    let update_rss_rank = update_rss_rank.into_inner();
//
//    match channel_service::update_rss_channel_rank(
//        pool,
//        update_rss_rank.channel_id,
//        update_rss_rank.num,
//    )
//    .await
//    {
//        Ok(_) => Ok("Success"),
//        Err(_) => Err(Status::InternalServerError),
//    }
//}

/// # update_rss_item_rank
///
/// Returns OK status if the RSS item rank is updated successfully, otherwise returns an error.
#[openapi(tag = "Rss")]
#[put("/item/rank", data = "<update_rss_rank>")]
pub async fn update_rss_item_rank(
    pool: &State<MySqlPool>,
    update_rss_rank: Json<UpdateRssRankRequestDto>,
) -> Result<Status, Status> {
    match item_service::update_rss_item_rank(pool, update_rss_rank.into_inner()).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}
