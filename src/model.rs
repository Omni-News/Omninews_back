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

// Morpheme
#[derive(Deserialize, Clone, FromRow)]
pub struct Morpheme {
    pub morpheme_word: String,
    pub morpheme_rank: u64,
}

// Newticle
#[derive(Deserialize, Clone, FromRow)]
pub struct MorphemeToSourceLink {
    pub morpheme_id: u64,
    pub channel_id: u64,
    pub rss_id: u64,
    pub news_id: u64,
    pub source_link: String,
}

impl MorphemeToSourceLink {
    pub fn new(newticle_type: &str, newticle_id: u64, morpheme_id: u64, link: &str) -> Self {
        match newticle_type {
            "channel" => MorphemeToSourceLink {
                morpheme_id,
                channel_id: newticle_id,
                rss_id: 0,
                news_id: 0,
                source_link: link.to_string(),
            },
            "rss" => MorphemeToSourceLink {
                morpheme_id,
                channel_id: 0,
                rss_id: newticle_id,
                news_id: 0,
                source_link: link.to_string(),
            },
            "news" => MorphemeToSourceLink {
                morpheme_id,
                channel_id: 0,
                rss_id: 0,
                news_id: newticle_id,
                source_link: link.to_string(),
            },
            _ => panic!("Invalid newticle type"),
        }
    }
}
