use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::news::News;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsResponseDto {
    pub news_id: Option<i32>,
    pub news_title: Option<String>,
    pub news_description: Option<String>,
    pub news_summary: Option<String>,
    pub news_link: Option<String>,
    pub news_source: Option<String>,
    pub news_pub_date: Option<NaiveDateTime>,
    pub news_image_link: Option<String>,
    pub news_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NewsApiResponseDto {
    pub news_title: Option<String>,
    pub news_original_link: Option<String>,
    pub news_link: Option<String>,
    pub news_description: Option<String>,
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
