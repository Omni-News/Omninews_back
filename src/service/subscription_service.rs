use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{
        error::OmniNewsError,
        rss::{RssChannel, RssItem},
        subscription::ChannelId,
    },
    repository::subscribe_repository,
};

use super::{rss::channel_service, user_service};

pub async fn get_subscription_channels(
    pool: &State<MySqlPool>,
    user_email: String,
) -> Result<Vec<RssChannel>, OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;
    match subscribe_repository::select_subscription_channels(pool, user_id).await {
        Ok(res) => Ok(res),
        Err(e) => {
            error!("Failed to select subscription channels: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn subscribe_channel(
    pool: &State<MySqlPool>,
    user_email: String,
    channel_id: ChannelId,
) -> Result<(), OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;

    if let Ok(res) = channel_service::is_channel_exist_by_id(pool, channel_id.channel_id).await {
        if !res {
            error!("Channel not found");
            return Err(OmniNewsError::NotFound("Channel not found".to_string()));
        }
    };

    match subscribe_repository::insert_user_subscribe_channel(pool, user_id, channel_id.channel_id)
        .await
    {
        Ok(_) => {
            let _ = channel_service::update_rss_channel_rank(pool, channel_id.channel_id, 1)
                .await
                .unwrap();
            Ok(())
        }
        Err(e) => {
            error!("Failed to subscribe channel: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn unsubscribe_channel(
    pool: &State<MySqlPool>,
    user_email: String,
    channel_id: ChannelId,
) -> Result<(), OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;

    if let Ok(res) = channel_service::is_channel_exist_by_id(pool, channel_id.channel_id).await {
        if !res {
            error!("Channel not found");
            return Err(OmniNewsError::NotFound("Channel not found".to_string()));
        }
    };

    match subscribe_repository::delete_subscribe_channel(pool, user_id, channel_id.channel_id).await
    {
        Ok(_) => {
            let _ = channel_service::update_rss_channel_rank(pool, channel_id.channel_id, -1)
                .await
                .unwrap();
            Ok(())
        }
        Err(e) => {
            error!("Failed to unsubscribe channel: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn get_subscription_items(
    pool: &State<MySqlPool>,
    channel_ids: Vec<ChannelId>,
) -> Result<Vec<RssItem>, OmniNewsError> {
    let channel_ids = channel_ids
        .iter()
        .map(|e| e.channel_id)
        .collect::<Vec<i32>>();

    match subscribe_repository::select_subscription_items(pool, channel_ids).await {
        Ok(res) => Ok(res),
        Err(e) => {
            error!("Failed to select subscription items: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn is_already_subscribe_channel(
    pool: &State<MySqlPool>,
    user_email: String,
    channel_rss_link: String,
) -> Result<bool, OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;
    let channel_id =
        match channel_service::find_rss_channel_by_rss_link(pool, channel_rss_link).await {
            Ok(res) => res.channel_id.unwrap_or_default(),
            Err(_) => {
                info!("Rss link is  new rss channel");
                return Ok(false);
            }
        };

    match subscribe_repository::is_already_subscribe_channel(pool, user_id, channel_id).await {
        Ok(res) => Ok(res),
        Err(e) => {
            error!("Failed to check if already subscribed: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
