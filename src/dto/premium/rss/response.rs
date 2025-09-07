use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::dto::rss::response::RssChannelResponseDto;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssGenerateResponseDto {
    pub is_exist: bool,
    pub channel: RssChannelResponseDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssSummaryResponseDto {
    pub text: String,
}
