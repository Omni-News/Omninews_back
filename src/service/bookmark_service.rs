use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{bookmark::ChannelLink, error::OmniNewsError, rss::RssItem},
    repository::bookmark_repository,
};

use super::rss::channel_service;

pub async fn get_bookmark_items(
    pool: &State<MySqlPool>,
    channel_links: Vec<ChannelLink>,
) -> Result<Vec<RssItem>, OmniNewsError> {
    let mut channel_ids: Vec<i32> = Vec::new();

    for channel_link in channel_links {
        match channel_service::get_rss_channel_by_link(pool, channel_link.link).await {
            Ok(channel) => channel_ids.push(channel.channel_id.unwrap()),
            Err(e) => {
                eprintln!("[Service] Failed to select rss channel by link: {:?}", e);
                return Err(e);
            }
        }
    }

    match bookmark_repository::select_bookmark_items(pool, channel_ids).await {
        Ok(res) => Ok(res),
        Err(e) => Err(OmniNewsError::Database(e)),
    }
}
