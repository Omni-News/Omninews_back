use sqlx::MySqlPool;

use crate::{
    dto::{
        rss::response::{RssChannelResponseDto, RssItemResponseDto},
        subscribe::request::{SubscribeRequestDto, SubscribeRssItemRequestDto},
    },
    model::error::OmniNewsError,
    repository::subscribe_repository,
    subscription_error, subscription_info,
};

use super::{channel_service, user_service};

pub async fn get_subscription_channels(
    pool: &MySqlPool,
    user_email: String,
) -> Result<Vec<RssChannelResponseDto>, OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;
    match subscribe_repository::select_subscription_channels(pool, user_id).await {
        Ok(res) => Ok(RssChannelResponseDto::from_model_list(res)),
        Err(e) => {
            subscription_error!("Failed to select subscription channels: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn subscribe_channel(
    pool: &MySqlPool,
    user_email: String,
    channel_id: SubscribeRequestDto,
) -> Result<(), OmniNewsError> {
    let channel_id = channel_id.channel_id.unwrap_or_default();
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;

    if let Ok(res) = channel_service::is_channel_exist_by_id(pool, channel_id).await {
        if !res {
            subscription_error!("Channel not found");
            return Err(OmniNewsError::NotFound("Channel not found".to_string()));
        }
    };

    match subscribe_repository::insert_user_subscribe_channel(pool, user_id, channel_id).await {
        Ok(_) => {
            let _ = channel_service::update_rss_channel_rank(pool, channel_id, 1)
                .await
                .unwrap();
            Ok(())
        }
        Err(e) => {
            subscription_error!("Failed to subscribe channel: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn unsubscribe_channel(
    pool: &MySqlPool,
    user_email: String,
    channel_id: SubscribeRequestDto,
) -> Result<(), OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;
    let channel_id = channel_id.channel_id.unwrap_or_default();

    if let Ok(res) = channel_service::is_channel_exist_by_id(pool, channel_id).await {
        if !res {
            subscription_error!("Channel not found");
            return Err(OmniNewsError::NotFound("Channel not found".to_string()));
        }
    };

    match subscribe_repository::delete_subscribe_channel(pool, user_id, channel_id).await {
        Ok(_) => {
            let _ = channel_service::update_rss_channel_rank(pool, channel_id, -1)
                .await
                .unwrap();
            Ok(())
        }
        Err(e) => {
            subscription_error!("Failed to unsubscribe channel: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn get_subscription_items(
    pool: &MySqlPool,
    data: SubscribeRssItemRequestDto,
) -> Result<Vec<RssItemResponseDto>, OmniNewsError> {
    let channel_ids: Vec<i32> = data
        .channel_ids
        .unwrap()
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let size = 20;
    let page = data.page.unwrap_or(1);
    let offset = (page - 1) * size;

    match subscribe_repository::select_subscription_items(pool, channel_ids, offset, size).await {
        Ok(res) => Ok(RssItemResponseDto::from_model_list(res)),
        Err(e) => {
            subscription_error!("Failed to select subscription items: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn is_already_subscribe_channel(
    pool: &MySqlPool,
    user_email: String,
    channel_rss_link: String,
) -> Result<bool, OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;
    subscription_info!("User ID: {}", user_id);
    subscription_info!("channel_rss_link: {}", channel_rss_link);
    let channel_id =
        match channel_service::find_rss_channel_by_rss_link(pool, channel_rss_link).await {
            Ok(res) => res.channel_id.unwrap_or_default(),
            Err(_) => {
                subscription_info!("Rss link is  new rss channel");
                return Ok(false);
            }
        };

    match subscribe_repository::is_already_subscribe_channel(pool, user_id, channel_id).await {
        Ok(res) => Ok(res),
        Err(e) => {
            subscription_error!("Failed to check if already subscribed: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
