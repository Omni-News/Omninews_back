use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{dto::rss::response::RssChannelResponseDto, model::rss::RssChannel};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RssFolderResponseDto {
    pub folder_id: Option<i32>,
    pub folder_name: Option<String>,
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
