use okapi::openapi3::OpenApi;
use rocket::http::Status;
use rocket::{serde::json::Json, State};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sqlx::MySqlPool;

use crate::auth_middleware::AuthenticatedUser;
use crate::dto::rss::response::{RssChannelResponseDto, RssItemResponseDto};
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

/// # subscribe_channel
///
/// Returns a success message if the user successfully subscribes to a channel.
#[openapi(tag = "Subscription")]
#[post("/channel_sub", data = "<channel_id>")]
pub async fn subscribe_channel(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    channel_id: Json<i32>,
) -> Result<&str, Status> {
    match subscription_service::subscribe_channel(pool, user.user_email, channel_id.into_inner())
        .await
    {
        Ok(_) => Ok("Success subscribe channel"),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # validate_already_subscribe_channel
///
/// Returns `true` if user is already subscribed to a channel based on the RSS link.
#[openapi(tag = "Subscription")]
#[get("/status?<channel_rss_link>")]
pub async fn validate_already_subscribe_channel(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    channel_rss_link: String,
) -> Result<Json<bool>, Status> {
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

/// get_subscribe_channels
///
/// Returns a list of channels that the user is subscribed to.
#[openapi(tag = "Subscription")]
#[get("/channels")]
pub async fn get_subscribe_channels(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<RssChannelResponseDto>>, Status> {
    match subscription_service::get_subscription_channels(pool, user.user_email).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # get_subscribe_items
///
/// Returns a list of items from subscribed channels based on the provided channel IDs.
#[openapi(tag = "Subscription")]
#[get("/items_sub?<channel_ids>")]
pub async fn get_subscribe_items(
    pool: &State<MySqlPool>,
    channel_ids: String,
) -> Result<Json<Vec<RssItemResponseDto>>, Status> {
    let channel_ids: Vec<i32> = channel_ids
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    match subscription_service::get_subscription_items(pool, channel_ids).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # unsubscribe_channel
///
/// Returns a success message if the user successfully unsubscribes from a channel.
#[openapi(tag = "Subscription")]
#[delete("/subscription/channel", data = "<channel_id>")]
pub async fn unsubscribe_channel(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    channel_id: Json<i32>,
) -> Result<&str, Status> {
    match subscription_service::unsubscribe_channel(pool, user.user_email, channel_id.into_inner())
        .await
    {
        Ok(_) => Ok("Success unsubscribe channel"),
        Err(_) => Err(Status::InternalServerError),
    }
}
