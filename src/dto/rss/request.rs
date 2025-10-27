use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, FromForm)]
pub struct RssItemRequestDto {
    #[schemars(example = "example_channel_id")]
    pub channel_id: Option<i32>,
    #[schemars(example = "example_page")]
    pub page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateRssRequestDto {
    #[schemars(example = "example_rss_link")]
    pub rss_link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRssRankRequestDto {
    #[schemars(example = "example_rss_id")]
    pub rss_id: i32,
    #[schemars(example = "example_num")]
    pub num: i32,
}

fn example_rss_link() -> &'static str {
    "https://example.com/rss"
}

fn example_rss_id() -> i32 {
    3
}

fn example_num() -> i32 {
    1
}

fn example_channel_id() -> i32 {
    5
}

fn example_page() -> i32 {
    1
}
