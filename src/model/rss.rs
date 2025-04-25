use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone)]
pub enum NewticleType {
    Channel,
    Rss,
    News,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RssLink {
    pub link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssId {
    pub rss_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelId {
    pub channel_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NewRssChannel {
    pub channel_title: Option<String>,
    pub channel_link: Option<String>,
    pub channel_description: Option<String>,
    pub channel_image_url: Option<String>,
    pub channel_language: Option<String>,
    pub rss_generator: Option<String>,
    pub channel_rank: Option<i32>,
    pub channel_rss_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RssChannel {
    pub channel_id: Option<i32>,
    pub channel_title: Option<String>,
    pub channel_link: Option<String>,
    pub channel_description: Option<String>,
    pub channel_image_url: Option<String>,
    pub channel_language: Option<String>,
    pub rss_generator: Option<String>,
    pub channel_rank: Option<i32>,
    pub channel_rss_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NewRssItem {
    pub channel_id: Option<i32>,
    pub rss_title: Option<String>,
    pub rss_description: Option<String>,
    pub rss_link: Option<String>,
    pub rss_author: Option<String>,
    pub rss_pub_date: Option<NaiveDateTime>,
    pub rss_rank: Option<i32>,
    pub rss_image_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RssItem {
    pub rss_id: Option<i32>,
    pub channel_id: Option<i32>,
    pub rss_title: Option<String>,
    pub rss_description: Option<String>,
    pub rss_link: Option<String>,
    pub rss_author: Option<String>,
    pub rss_pub_date: Option<NaiveDateTime>,
    pub rss_rank: Option<i32>,
    pub rss_image_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChannelRank {
    pub channel_id: i32,
    pub num: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRssRank {
    pub rss_id: i32,
    pub num: i32,
}
