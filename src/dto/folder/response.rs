use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{dto::rss::response::RssChannelResponseDto, model::rss::RssChannel};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RssFolderResponseDto {
    #[schemars(example = "example_folder_id")]
    pub folder_id: Option<i32>,
    #[schemars(example = "example_folder_name")]
    pub folder_name: Option<String>,
    #[schemars(example = "example_channel_id")]
    pub folder_channels: Option<Vec<RssChannelResponseDto>>,
}

impl RssFolderResponseDto {
    pub fn new(
        folder_id: Option<i32>,
        folder_name: Option<String>,
        channels: Vec<RssChannel>,
    ) -> Self {
        Self {
            folder_id,
            folder_name,
            folder_channels: Some(RssChannelResponseDto::from_model_list(channels)),
        }
    }
}

fn example_folder_id() -> i32 {
    1
}
fn example_folder_name() -> &'static str {
    "Tech News"
}
fn example_channel_id() -> i32 {
    3
}
