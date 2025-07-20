use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::dto::rss::response::{RssChannelResponseDto, RssItemResponseDto};

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct SearchResponseDto {
    #[schemars(example = "example_channels")]
    pub channels: Option<Vec<RssChannelResponseDto>>,
    #[schemars(example = "example_items")]
    pub items: Option<Vec<RssItemResponseDto>>,
    #[schemars(example = "example_total")]
    pub total: Option<i32>,
    #[schemars(example = "example_page")]
    pub page: Option<i32>,
    #[schemars(example = "example_has_next")]
    pub has_next: Option<bool>,
}

impl SearchResponseDto {
    pub fn new(
        channels: Vec<RssChannelResponseDto>,
        items: Vec<RssItemResponseDto>,
        total: i32,
        page: i32,
        has_next: bool,
    ) -> Self {
        Self {
            channels: Some(channels),
            items: Some(items),
            total: Some(total),
            page: Some(page),
            has_next: Some(has_next),
        }
    }
}

fn example_channels() -> Vec<RssChannelResponseDto> {
    vec![
        RssChannelResponseDto {
            channel_id: Some(1),
            channel_title: Some("Example Channel".to_string()),
            channel_link: Some("https://example.com".to_string()),
            channel_description: Some("An example RSS channel".to_string()),
            channel_image_url: Some("https://example.com/image.png".to_string()),
            channel_language: Some("en".to_string()),
            rss_generator: Some("Example Generator".to_string()),
            channel_rank: Some(1),
            channel_rss_link: Some("https://example.com/rss".to_string()),
        },
        RssChannelResponseDto {
            channel_id: Some(2),
            channel_title: Some("Example Channel".to_string()),
            channel_link: Some("https://example.com".to_string()),
            channel_description: Some("An example RSS channel".to_string()),
            channel_image_url: Some("https://example.com/image.png".to_string()),
            channel_language: Some("en".to_string()),
            rss_generator: Some("Example Generator".to_string()),
            channel_rank: Some(1),
            channel_rss_link: Some("https://example.com/rss".to_string()),
        },
    ]
}

fn example_items() -> Vec<RssItemResponseDto> {
    vec![
        RssItemResponseDto {
            rss_id: Some(1),
            channel_id: Some(1),
            rss_title: Some("Example Item".to_string()),
            rss_description: Some("An example RSS item".to_string()),
            rss_link: Some("https://example.com/item".to_string()),
            rss_author: Some("Author Name".to_string()),
            rss_pub_date: None, // Example without a date
            rss_rank: Some(1),
            rss_image_link: Some("https://example.com/item_image.png".to_string()),
        },
        RssItemResponseDto {
            rss_id: Some(2),
            channel_id: Some(1),
            rss_title: Some("Example Item".to_string()),
            rss_description: Some("An example RSS item".to_string()),
            rss_link: Some("https://example.com/item".to_string()),
            rss_author: Some("Author Name".to_string()),
            rss_pub_date: None, // Example without a date
            rss_rank: Some(1),
            rss_image_link: Some("https://example.com/item_image.png".to_string()),
        },
    ]
}

fn example_total() -> i32 {
    5
}

fn example_page() -> i32 {
    1
}

fn example_has_next() -> bool {
    true
}
