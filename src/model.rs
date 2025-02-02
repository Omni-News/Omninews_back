use chrono::NaiveDateTime;
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

#[derive(Serialize, Deserialize, Clone, FromRow, Debug)]
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
    pub rss_image_link: Option<String>,
}
// Morpheme

#[derive(Deserialize, Clone, FromRow, Debug)]
pub struct NewMorpheme {
    pub morpheme_word: Option<String>,
    pub morpheme_rank: Option<i32>,
}

#[derive(Deserialize, Clone, FromRow, Debug)]
pub struct Morpheme {
    pub morpheme_id: Option<i32>,
    pub morpheme_word: Option<String>,
    pub morpheme_rank: Option<i32>,
}

// Newticle
#[derive(Deserialize, Clone, FromRow, Debug)]
pub struct MorphemeToSourceLink {
    pub morpheme_id: Option<i32>,
    pub channel_id: Option<i32>,
    pub rss_id: Option<i32>,
    pub news_id: Option<i32>,
    pub source_link: Option<String>,
    pub source_rank: Option<i32>,
}

impl MorphemeToSourceLink {
    pub fn new(
        newticle_type: &str,
        newticle_id: Option<i32>,
        morpheme_id: Option<i32>,
        link: Option<String>,
        source_rank: Option<i32>,
    ) -> Self {
        match newticle_type {
            "channel" => MorphemeToSourceLink {
                morpheme_id,
                channel_id: newticle_id,
                rss_id: Some(0),
                news_id: Some(0),
                source_link: link,
                source_rank,
            },
            "rss" => MorphemeToSourceLink {
                morpheme_id,
                channel_id: Some(0),
                rss_id: newticle_id,
                news_id: Some(0),
                source_link: link,
                source_rank,
            },
            "news" => MorphemeToSourceLink {
                morpheme_id,
                channel_id: Some(0),
                rss_id: Some(0),
                news_id: newticle_id,
                source_link: link,
                source_rank,
            },
            _ => panic!("Invalid newticle type"),
        }
    }
}

// User Prompt
#[derive(Deserialize, Clone, Debug)]
pub enum SearchType {
    Accuracy,
    Popularity,
    Latest,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserPrompt {
    pub user_prompt: Option<String>,
    pub search_type: Option<SearchType>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewticleList {
    pub channel_list: Option<Vec<RssChannel>>,
    pub rss_list: Option<Vec<RssItem>>,
    pub news_list: Option<Vec<News>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct News {}
