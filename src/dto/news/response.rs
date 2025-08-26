use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::news::News;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsResponseDto {
    #[schemars(example = "example_news_id")]
    pub news_id: Option<i32>,
    #[schemars(example = "example_news_title")]
    pub news_title: Option<String>,
    #[schemars(example = "example_news_description")]
    pub news_description: Option<String>,
    #[schemars(example = "example_news_summary")]
    pub news_summary: Option<String>,
    #[schemars(example = "example_news_link")]
    pub news_link: Option<String>,
    #[schemars(example = "example_news_source")]
    pub news_source: Option<String>,
    #[schemars(example = "example_news_pub_date")]
    pub news_pub_date: Option<NaiveDateTime>,
    #[schemars(example = "example_news_image_link")]
    pub news_image_link: Option<String>,
    #[schemars(example = "example_news_category")]
    pub news_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsApiResponseDto {
    #[schemars(example = "example_news_title")]
    pub news_title: Option<String>,
    #[schemars(example = "example_news_original_link")]
    pub news_original_link: Option<String>,
    #[schemars(example = "example_news_link")]
    pub news_link: Option<String>,
    #[schemars(example = "example_news_description")]
    pub news_description: Option<String>,
    #[schemars(example = "example_news_pub_date")]
    pub news_pub_date: Option<NaiveDateTime>,
}

impl NewsResponseDto {
    pub fn from_model(news: News) -> Self {
        NewsResponseDto {
            news_id: news.news_id,
            news_title: news.news_title,
            news_description: news.news_description,
            news_summary: news.news_summary,
            news_link: news.news_link,
            news_source: news.news_source,
            news_pub_date: news.news_pub_date,
            news_image_link: news.news_image_link,
            news_category: news.news_category,
        }
    }

    pub fn from_model_list(news_list: Vec<News>) -> Vec<Self> {
        news_list.into_iter().map(Self::from_model).collect()
    }
}

impl NewsApiResponseDto {
    pub fn new(
        news_title: String,
        news_original_link: String,
        news_link: String,
        news_description: String,
        news_pub_date: NaiveDateTime,
    ) -> Self {
        NewsApiResponseDto {
            news_title: Some(news_title),
            news_original_link: Some(news_original_link),
            news_link: Some(news_link),
            news_description: Some(news_description),
            news_pub_date: Some(news_pub_date),
        }
    }
}

/*
*
    pub news_id: Option<i32>,
    pub news_title: Option<String>,
    pub news_description: Option<String>,
    pub news_summary: Option<String>,
    pub news_link: Option<String>,
    pub news_source: Option<String>,
    pub news_pub_date: Option<NaiveDateTime>,
    pub news_image_link: Option<String>,
    pub news_category: Option<String>,
*/
fn example_news_id() -> i32 {
    1
}
fn example_news_title() -> &'static str {
    "Example News Title"
}
fn example_news_description() -> &'static str {
    "This is an example news description."
}
fn example_news_summary() -> &'static str {
    "This is an example news summary."
}
fn example_news_link() -> &'static str {
    "https://example.com/news/1"
}
fn example_news_source() -> &'static str {
    "Example News Source"
}
fn example_news_pub_date() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("2023-10-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
}
fn example_news_image_link() -> &'static str {
    "https://example.com/news/1/image.jpg"
}
fn example_news_category() -> &'static str {
    "Example Category"
}
fn example_news_original_link() -> &'static str {
    "https://original.example.com/news/1"
}
