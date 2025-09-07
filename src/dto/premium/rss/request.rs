use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::premium::rss_generate::SiteType;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssGenerateRequestDto {
    pub channel_link: String,
    pub kind: SiteType,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssGenerateByCssReqeustDto {
    #[schemars(example = "channel_link_example")]
    pub channel_link: String,
    #[schemars(example = "channel_image_link_example")]
    pub channel_image_link: String,
    #[schemars(example = "channel_title_example")]
    pub channel_title: String,
    #[schemars(example = "channel_description_example")]
    pub channel_description: String,
    #[schemars(example = "channel_language_example")]
    pub channel_language: String,

    #[schemars(example = "item_title_css_example")]
    pub item_title_css: String,
    #[schemars(example = "item_description_css_example")]
    pub item_description_css: String,
    #[schemars(example = "item_link_css_example")]
    pub item_link_css: String,
    #[schemars(example = "item_author_css_example")]
    pub item_author_css: String,
    #[schemars(example = "item_pub_date_css_example")]
    pub item_pub_date_css: String,
    #[schemars(example = "item_image_css_example")]
    pub item_image_css: String,
}

fn channel_link_example() -> &'static str {
    "https://example.com/rss"
}
fn channel_image_link_example() -> &'static str {
    "https://example.com/image.png"
}
fn channel_title_example() -> &'static str {
    "Example Channel"
}
fn channel_description_example() -> &'static str {
    "This is an example channel description."
}
fn channel_language_example() -> &'static str {
    "ko-KR"
}
fn item_title_css_example() -> &'static str {
    ".item-title"
}
fn item_description_css_example() -> &'static str {
    ".item-description"
}
fn item_link_css_example() -> &'static str {
    ".item-link"
}
fn item_author_css_example() -> &'static str {
    ".item-author"
}
fn item_pub_date_css_example() -> &'static str {
    ".item-pub-date"
}
fn item_image_css_example() -> &'static str {
    ".item-image"
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssSummaryRequestDto {
    pub rss_link: String,
}
