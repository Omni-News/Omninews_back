use chrono::{DateTime, TimeZone, Utc};
use rocket::State;
use rss::Channel;
use sqlx::MySqlPool;

use crate::{
    model::{RssChannel, RssItem, RssLink},
    repository::rss_repository,
};

pub async fn create_rss(pool: &State<MySqlPool>, rss_link: RssLink) -> Result<(), ()> {
    let link = rss_link.link;

    let response = reqwest::get(&link).await.unwrap();
    let body = response.text().await.unwrap();

    let channel = Channel::read_from(body.as_bytes()).unwrap();

    let rss_channel = RssChannel {
        channel_title: channel.title().to_string(),
        channel_link: channel.link().to_string(),
        channel_description: channel.description().to_string(),
        channel_image_url: channel.image().unwrap().url().to_string(),
        channel_language: channel.language().unwrap_or("None").to_string(),
        rss_generator: channel.generator().unwrap_or("None").to_string(),
    };
    let rss_channel_id = create_rss_channel(pool, rss_channel).await.unwrap();
    let mut rss_item: Vec<RssItem> = Vec::new();
    for item in channel.items() {
        rss_item.push(RssItem {
            channel_id: rss_channel_id,
            rss_title: item.title().unwrap_or("None").to_string(),
            rss_description: item.description().unwrap_or("None").to_string(),
            rss_link: item.link().unwrap_or("None").to_string(),
            rss_creator: item.author().unwrap_or("None").to_string(),
            rss_pub_date: parse_pub_date(item.pub_date()),
            rss_categories: item
                .categories()
                .iter()
                .map(|cat| cat.name().to_string())
                .collect(),
        });
    }
    create_rss_item(pool, rss_item).await.unwrap();

    Ok(())
}

pub async fn create_rss_item(pool: &State<MySqlPool>, rss_item: Vec<RssItem>) -> Result<(), ()> {
    for item in rss_item {
        rss_repository::create_rss_item(pool, item).await.unwrap();
    }

    Ok(())
}

pub async fn create_rss_channel(
    pool: &State<MySqlPool>,
    rss_channel: RssChannel,
) -> Result<u64, sqlx::Error> {
    rss_repository::create_rss_channel(pool, rss_channel).await
}

pub fn parse_pub_date(pub_date_str: Option<&str>) -> DateTime<Utc> {
    pub_date_str
        .and_then(|date_str| DateTime::parse_from_rfc3339(date_str).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc.datetime_from_str("1970-01-01T00:00:00Z", "%+").unwrap())
}
