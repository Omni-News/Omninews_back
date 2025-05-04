use rocket::http::Status;
use rocket::{serde::json::Json, State};
use sqlx::MySqlPool;

use crate::auth_middleware::AuthenticatedUser;
use crate::model::rss::{RssChannel, RssItem};
use crate::model::subscription::ChannelId;
use crate::service::subscription_service;

#[get("/subscription/status?<channel_rss_link>")]
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

#[get("/subscription/channels")]
pub async fn get_subscribe_channels(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<RssChannel>>, Status> {
    match subscription_service::get_subscription_channels(pool, user.user_email).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/subscription/items", data = "<channel_id>")]
pub async fn get_subscribe_items(
    pool: &State<MySqlPool>,
    channel_id: Json<Vec<ChannelId>>,
) -> Result<Json<Vec<RssItem>>, Status> {
    match subscription_service::get_subscription_items(pool, channel_id.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/subscription/channel", data = "<channel_id>")]
pub async fn subscribe_channel(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    channel_id: Json<ChannelId>,
) -> Result<&str, Status> {
    match subscription_service::subscribe_channel(pool, user.user_email, channel_id.into_inner())
        .await
    {
        Ok(_) => Ok("Success subscribe channel"),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[delete("/subscription/channel", data = "<channel_id>")]
pub async fn unsubscribe_channel(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    channel_id: Json<ChannelId>,
) -> Result<&str, Status> {
    match subscription_service::unsubscribe_channel(pool, user.user_email, channel_id.into_inner())
        .await
    {
        Ok(_) => Ok("Success unsubscribe channel"),
        Err(_) => Err(Status::InternalServerError),
    }
}
