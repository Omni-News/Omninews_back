use chrono::NaiveDateTime;
use rocket::form::FromForm;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct News {
    pub news_id: Option<i32>,
    pub news_title: Option<String>,
    pub news_description: Option<String>,
    pub news_link: Option<String>,
    pub news_source: Option<String>,
    pub news_pub_date: Option<NaiveDateTime>,
    pub news_image_link: Option<String>,
    pub news_category: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NewNews {
    pub news_title: Option<String>,
    pub news_description: Option<String>,
    pub news_link: Option<String>,
    pub news_source: Option<String>,
    pub news_pub_date: Option<NaiveDateTime>,
    pub news_image_link: Option<String>,
    pub news_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct NewsParams {
    pub query: Option<String>,
    pub display: Option<i32>,
    pub sort: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct NewsItem {
    pub news_title: Option<String>,
    pub news_original_link: Option<String>,
    pub news_link: Option<String>,
    pub news_description: Option<String>,
    pub news_pub_date: Option<NaiveDateTime>,
}
