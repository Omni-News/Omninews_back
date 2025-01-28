use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// RSS
#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct RssLink {
    pub link: String,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct RssChannel {
    pub channel_title: String,
    pub channel_link: String,
    pub channel_description: String,
    pub channel_image_url: String,
    pub channel_language: String,
    pub rss_generator: String,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct RssItem {
    pub channel_id: u64,
    pub rss_title: String,
    pub rss_description: String,
    pub rss_link: String,
    pub rss_creator: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub rss_pub_date: DateTime<Utc>,
    pub rss_categories: Vec<String>,
}
