use rocket::{http::Status, serde::json::Json, State};
use sqlx::MySqlPool;

use crate::{model::news::News, service::news_service};

#[get("/fetch_news")]
pub async fn create_news(pool: &State<MySqlPool>) -> Result<(), Status> {
    match news_service::crawl_news_and_store_every_5_minutes(pool).await {
        Ok(_) => Ok(()),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/news?<category>")]
pub async fn get_news(pool: &State<MySqlPool>, category: &str) -> Result<Json<Vec<News>>, Status> {
    match news_service::get_news(pool, category.to_string()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
