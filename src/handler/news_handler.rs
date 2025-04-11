use rocket::{http::Status, serde::json::Json, State};
use sqlx::MySqlPool;

use crate::{
    global::FETCH_FLAG,
    model::news::{News, NewsItem, NewsParams},
    service::news_service,
};

#[get("/news?<category>")]
pub async fn get_news(
    pool: &State<MySqlPool>,
    category: String,
) -> Result<Json<Vec<News>>, Status> {
    match news_service::get_news(pool, category).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/news/api?<params..>")]
pub async fn get_news_by_api(params: NewsParams) -> Result<Json<Vec<NewsItem>>, Status> {
    match news_service::get_news_by_api(params).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/fetch_start")]
pub async fn fetch_start() -> &'static str {
    let mut fetch_flag = FETCH_FLAG.lock().unwrap();
    *fetch_flag = true;
    "Fetching started!"
}

#[get("/fetch_stop")]
pub async fn fetch_stop() -> &'static str {
    let mut fetch_flag = FETCH_FLAG.lock().unwrap();
    *fetch_flag = false;
    "Fetching stopped!"
}
