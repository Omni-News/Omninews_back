use chrono::{DateTime, NaiveDateTime};
use rocket::State;
use rss::{Channel, Item};
use sqlx::MySqlPool;

use crate::{
    model::{NewRssChannel, NewRssItem, RssLink},
    repository::{
        rss_channel_repository::{self},
        rss_item_repository,
    },
};

use super::morpheme_service::{create_morpheme_for_channel, validate_and_create_morpheme_for_rss};

pub async fn create_rss_and_morpheme(
    pool: &State<MySqlPool>,
    rss_link: RssLink,
) -> Result<u64, ()> {
    let link = rss_link.link;

    let response = reqwest::get(&link).await.unwrap();
    let body = response.text().await.unwrap();
    let channel = Channel::read_from(body.as_bytes()).unwrap();

    let rss_channel = make_rss_channel(channel.clone());
    let rss_channel_id = validate_and_create_rss_channel(pool, rss_channel.clone())
        .await
        .unwrap();
    create_morpheme_for_channel(pool, rss_channel, rss_channel_id).await;

    for item in channel.items() {
        let rss_item = make_rss_item(rss_channel_id, item);
        let rss_id = validate_and_create_rss_item(pool, rss_item.clone())
            .await
            .unwrap();
        validate_and_create_morpheme_for_rss(pool, rss_item, rss_id).await;
    }

    Ok(rss_channel_id)
}

// TODO 사용자가 rss를 눌렀을 때도 rank +1 처리.
async fn validate_and_create_rss_item(
    pool: &State<MySqlPool>,
    rss_item: NewRssItem,
) -> Result<u64, sqlx::Error> {
    let rss_link = rss_item.clone().rss_link.unwrap();
    if let Ok(rss_item) = rss_item_repository::select_rss_item_by_link(pool, rss_link).await {
        let mut update_rss_item = rss_item.clone();
        update_rss_item.rss_rank = rss_item.rss_rank.map(|e| e + 1);

        return rss_item_repository::update_rss_item_by_id(pool, update_rss_item).await;
    }
    rss_item_repository::insert_rss_item(pool, rss_item).await
}

async fn validate_and_create_rss_channel(
    pool: &State<MySqlPool>,
    rss_channel: NewRssChannel,
) -> Result<u64, sqlx::Error> {
    let channel_link = rss_channel.clone().channel_link.unwrap();
    if let Ok(rss_channel) =
        rss_channel_repository::select_rss_channel_by_link(pool, channel_link).await
    {
        let mut update_rss_channel = rss_channel.clone();
        update_rss_channel.channel_rank = rss_channel.channel_rank.map(|e| e + 1);

        return rss_channel_repository::update_rss_channel_by_id(pool, update_rss_channel).await;
    }
    rss_channel_repository::insert_rss_channel(pool, rss_channel).await
}

fn parse_pub_date(pub_date_str: Option<&str>) -> Option<NaiveDateTime> {
    pub_date_str
        .and_then(|date_str| DateTime::parse_from_rfc3339(date_str).ok())
        .map(|dt| dt.naive_utc())
        .or_else(|| {
            Some(
                NaiveDateTime::parse_from_str("1970-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S")
                    .ok()
                    .unwrap(),
            )
        })
}

fn make_rss_channel(channel: Channel) -> NewRssChannel {
    NewRssChannel {
        channel_title: Some(channel.title().to_string()),
        channel_link: Some(channel.link().to_string()),
        channel_description: Some(channel.description().to_string()),
        channel_image_url: Some(channel.image().unwrap().url().to_string()),
        channel_language: Some(channel.language().unwrap_or("None").to_string()),
        rss_generator: Some(channel.generator().unwrap_or("None").to_string()),
        channel_rank: Some(1),
    }
}
fn make_rss_item(rss_channel_id: u64, item: &Item) -> NewRssItem {
    NewRssItem {
        channel_id: Some(rss_channel_id as i32),
        rss_title: Some(item.title().unwrap_or("None").to_string()),
        rss_description: Some(item.description().unwrap_or("None").to_string()),
        rss_link: Some(item.link().unwrap_or("None").to_string()),
        rss_author: Some(item.author().unwrap_or("None").to_string()),
        rss_pub_date: parse_pub_date(item.pub_date()),
        rss_rank: Some(1),
    }
}
