use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::rss::{RssChannel, RssItem};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssChannelResponseDto {
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssItemResponseDto {
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
