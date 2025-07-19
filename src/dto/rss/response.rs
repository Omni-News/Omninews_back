use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::rss::{RssChannel, RssItem};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssChannelResponseDto {
    #[schemars(example = "example_channel_id")]
    pub channel_id: Option<i32>,
    #[schemars(example = "example_channel_title")]
    pub channel_title: Option<String>,
    #[schemars(example = "example_channel_link")]
    pub channel_link: Option<String>,
    #[schemars(example = "example_channel_description")]
    pub channel_description: Option<String>,
    #[schemars(example = "example_channel_image_url")]
    pub channel_image_url: Option<String>,
    #[schemars(example = "example_channel_language")]
    pub channel_language: Option<String>,
    #[schemars(example = "example_rss_generator")]
    pub rss_generator: Option<String>,
    #[schemars(example = "example_channel_rank")]
    pub channel_rank: Option<i32>,
    #[schemars(example = "example_channel_rss_link")]
    pub channel_rss_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssItemResponseDto {
    #[schemars(example = "example_rss_id")]
    pub rss_id: Option<i32>,
    #[schemars(example = "example_channel_id")]
    pub channel_id: Option<i32>,
    #[schemars(example = "example_rss_title")]
    pub rss_title: Option<String>,
    #[schemars(example = "example_rss_description")]
    pub rss_description: Option<String>,
    #[schemars(example = "example_rss_link")]
    pub rss_link: Option<String>,
    #[schemars(example = "example_rss_author")]
    pub rss_author: Option<String>,
    #[schemars(example = "example_rss_pub_date")]
    pub rss_pub_date: Option<NaiveDateTime>,
    #[schemars(example = "example_rss_rank")]
    pub rss_rank: Option<i32>,
    #[schemars(example = "example_rss_image_link")]
    pub rss_image_link: Option<String>,
}

impl RssChannelResponseDto {
    pub fn from_model(channel: RssChannel) -> Self {
        RssChannelResponseDto {
            channel_id: channel.channel_id,
            channel_title: channel.channel_title,
            channel_link: channel.channel_link,
            channel_description: channel.channel_description,
            channel_image_url: channel.channel_image_url,
            channel_language: channel.channel_language,
            rss_generator: channel.rss_generator,
            channel_rank: channel.channel_rank,
            channel_rss_link: channel.channel_rss_link,
        }
    }

    pub fn from_model_list(channels: Vec<RssChannel>) -> Vec<Self> {
        channels.into_iter().map(Self::from_model).collect()
    }
}

impl RssItemResponseDto {
    pub fn from_model(item: RssItem) -> Self {
        RssItemResponseDto {
            rss_id: item.rss_id,
            channel_id: item.channel_id,
            rss_title: item.rss_title,
            rss_description: item.rss_description,
            rss_link: item.rss_link,
            rss_author: item.rss_author,
            rss_pub_date: item.rss_pub_date,
            rss_rank: item.rss_rank,
            rss_image_link: item.rss_image_link,
        }
    }

    pub fn from_model_list(items: Vec<RssItem>) -> Vec<Self> {
        items.into_iter().map(Self::from_model).collect()
    }
}

// channel
fn example_channel_id() -> i32 {
    12345
}
fn example_channel_title() -> &'static str {
    "Example RSS Channel"
}
fn example_channel_link() -> &'static str {
    "https://example.com/rss"
}
fn example_channel_description() -> &'static str {
    "This is an example RSS channel description."
}
fn example_channel_image_url() -> &'static str {
    "https://example.com/image.png"
}
fn example_channel_language() -> &'static str {
    "en"
}
fn example_rss_generator() -> &'static str {
    "Example RSS Generator"
}
fn example_channel_rank() -> i32 {
    1
}
fn example_channel_rss_link() -> &'static str {
    "https://example.com/rss/feed"
}

// item
fn example_rss_id() -> i32 {
    67890
}
fn example_rss_title() -> &'static str {
    "Example RSS Item Title"
}
fn example_rss_description() -> &'static str {
    "This is an example RSS item description."
}
fn example_rss_link() -> &'static str {
    "https://example.com/rss/item"
}
fn example_rss_author() -> &'static str {
    "John Doe"
}
fn example_rss_pub_date() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("1970-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S")
        .ok()
        .unwrap()
}
fn example_rss_rank() -> i32 {
    10
}
fn example_rss_image_link() -> &'static str {
    "https://example.com/rss/item/image.png"
}
