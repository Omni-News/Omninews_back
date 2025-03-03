use rocket::{http::Status, serde::json::Json, State};
use sqlx::MySqlPool;

use crate::{global::FETCH_FLAG, model::news::News, service::news_service};

#[get("/fetch_start")]
pub async fn fetch_start() -> &'static str {
    let mut fetch_flag = FETCH_FLAG.lock().await;
    *fetch_flag = true;
    "Fetching started!"
}

#[get("/fetch_stop")]
pub async fn fetch_stop() -> &'static str {
    let mut fetch_flag = FETCH_FLAG.lock().await;
    *fetch_flag = false;
    "Fetching stopped!"
}

#[get("/news?<category>")]
pub async fn get_news(pool: &State<MySqlPool>, category: &str) -> Result<Json<Vec<News>>, Status> {
    match news_service::get_news(pool, category.to_string()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
