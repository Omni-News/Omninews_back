use okapi::openapi3::OpenApi;
use rocket::http::Status;
use rocket::{serde::json::Json, State};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sqlx::MySqlPool;

use crate::auth_middleware::AuthenticatedUser;
use crate::dto::rss::response::{RssChannelResponseDto, RssItemResponseDto};
use crate::dto::subscribe::request::{SubscribeRequestDto, SubscribeRssItemRequestDto};
use crate::service::subscription_service;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings:
        validate_already_subscribe_channel,
        get_subscribe_channels,
        get_subscribe_items,
        subscribe_channel,
        unsubscribe_channel
    ]
}

/// # 채널 구독 API
///
/// 사용자가 채널 ID를 통해 채널을 구독합니다.
///
/// ### `channel_id` 구독할 채널 ID (예: 3)
///
#[openapi(tag = "Subscription")]
#[post("/subscription/channel_sub", data = "<channel_id>")]
pub async fn subscribe_channel(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    channel_id: Json<SubscribeRequestDto>,
) -> Result<&str, Status> {
    match subscription_service::subscribe_channel(pool, user.user_email, channel_id.into_inner())
        .await
    {
        Ok(_) => Ok("Success subscribe channel"),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 이미 구독된 채널 확인 API
///
/// 이미 사용자가 구독한 채널인지 확인합니다.
///
/// ### `channel_rss_link` : 채널의 RSS 링크 (예: "https://example.com/feed.xml")
///
#[openapi(tag = "Subscription")]
#[get("/subscription/status?<channel_rss_link>")]
pub async fn validate_already_subscribe_channel(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    channel_rss_link: String,
) -> Result<Json<bool>, Status> {
    info!("channel_rss_link: {}", channel_rss_link);
    match subscription_service::is_already_subscribe_channel(
        pool,
        user.user_email,
        channel_rss_link,
    )
    .await
    {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 사용자 구독 채널 조회 API
///
/// 사용자가 구독한 채널 목록을 조회합니다.
///
#[openapi(tag = "Subscription")]
#[get("/subscription/channels")]
pub async fn get_subscribe_channels(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<RssChannelResponseDto>>, Status> {
    match subscription_service::get_subscription_channels(pool, user.user_email).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

// TODO 구독한 채널 가져와서 바로 아이템 조회되도록 로직 수정
/// # 사용자가 구독한 채널의 아이템 조회 API
///
/// 사용자가 구독한 채널의 아이템 목록을 조회합니다.
///
/// ### `channel_ids` : 구독한 채널 ID 목록 (예: "1, 2, 3")
/// ### `page` : 검색할 페이지 번호 (예: 1)
///
#[openapi(tag = "Subscription")]
#[get("/subscription/items?<data..>")]
pub async fn get_subscribe_items(
    pool: &State<MySqlPool>,
    data: SubscribeRssItemRequestDto,
    _auth: AuthenticatedUser,
) -> Result<Json<Vec<RssItemResponseDto>>, Status> {
    match subscription_service::get_subscription_items(pool, data).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 구독 취소 API
///
/// 사용자가 구독한 채널을 취소합니다.
///
/// ### `channel_id` : 구독 취소할 채널 ID (예: 3)
///
#[openapi(tag = "Subscription")]
#[delete("/subscription/channel", data = "<channel_id>")]
pub async fn unsubscribe_channel(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    channel_id: Json<SubscribeRequestDto>,
) -> Result<&str, Status> {
    match subscription_service::unsubscribe_channel(pool, user.user_email, channel_id.into_inner())
        .await
    {
        Ok(_) => Ok("Success unsubscribe channel"),
        Err(_) => Err(Status::InternalServerError),
    }
}
