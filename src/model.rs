use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// RSS
#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct RssLink {
    pub link: String,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct NewRssChannel {
    pub channel_title: Option<String>,
    pub channel_link: Option<String>,
    pub channel_description: Option<String>,
    pub channel_image_url: Option<String>,
    pub channel_language: Option<String>,
    pub rss_generator: Option<String>,
    pub channel_rank: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct RssChannel {
    pub channel_id: Option<i32>,
    pub channel_title: Option<String>,
    pub channel_link: Option<String>,
    pub channel_description: Option<String>,
    pub channel_image_url: Option<String>,
    pub channel_language: Option<String>,
    pub rss_generator: Option<String>,
    pub channel_rank: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct NewRssItem {
    pub channel_id: Option<i32>,
    pub rss_title: Option<String>,
    pub rss_description: Option<String>,
    pub rss_link: Option<String>,
    pub rss_author: Option<String>,
    pub rss_pub_date: Option<NaiveDateTime>,
    pub rss_rank: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
pub struct RssItem {
    pub rss_id: Option<i32>,
    pub channel_id: Option<i32>,
    pub rss_title: Option<String>,
    pub rss_description: Option<String>,
    pub rss_link: Option<String>,
    pub rss_author: Option<String>,
    pub rss_pub_date: Option<NaiveDateTime>,
    pub rss_rank: Option<i32>,
}
// Morpheme

#[derive(Deserialize, Clone, FromRow)]
pub struct NewMorpheme {
    pub morpheme_word: Option<String>,
    pub morpheme_rank: Option<i32>,
}

#[derive(Deserialize, Clone, FromRow)]
pub struct Morpheme {
    pub morpheme_id: Option<i32>,
    pub morpheme_word: Option<String>,
    pub morpheme_rank: Option<i32>,
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
