use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::premium::rss_generate::SiteType;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssGenerateRequestDto {
    pub channel_link: String,
    pub kind: SiteType,
}
