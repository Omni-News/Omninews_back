use rocket::http::Status;
use rocket::{serde::json::Json, State};
use sqlx::MySqlPool;

use crate::auth_middleware::AuthenticatedUser;
use crate::model::rss::RssItem;
use crate::model::subscription::ChannelId;
use crate::service::subscription_service;

/**
* @param SubscribeChannelInput
* channel_id: i32,
*/
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

// TODO  프론트 api 엔드포인트 맞춰 바꾸기
#[post("/subscription/items", data = "<channel_links>")]
pub async fn get_subscribe_items(
    pool: &State<MySqlPool>,
    channel_links: Json<Vec<ChannelId>>,
) -> Result<Json<Vec<RssItem>>, Status> {
    match subscription_service::get_subscription_items(pool, channel_links.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
