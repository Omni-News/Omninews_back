use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sqlx::MySqlPool;

use crate::{
    dto::news::{
        request::NewsRequestDto,
        response::{NewsApiResponseDto, NewsResponseDto},
    },
    global::FETCH_FLAG,
    service::news_service,
};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_news, get_news_by_api, fetch_start, fetch_stop]
}

/// # get_news
///
/// Return a news list based on the provided category.
#[openapi(tag = "News")]
#[get("/?<category>")]
pub async fn get_news(
    pool: &State<MySqlPool>,
    category: String,
) -> Result<Json<Vec<NewsResponseDto>>, Status> {
    match news_service::get_news(pool, category).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # get_news_by_api
///
/// Retrun a news list based on the provided parameters.
#[openapi(tag = "News")]
#[get("/api?<params..>")]
pub async fn get_news_by_api(
    params: NewsRequestDto,
) -> Result<Json<Vec<NewsApiResponseDto>>, Status> {
    match news_service::get_news_by_api(params).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # fetch_start
///
/// Resume fetching news from the API.
#[openapi(tag = "News")]
#[get("/fetch_start")]
pub async fn fetch_start() -> &'static str {
    let mut fetch_flag = FETCH_FLAG.lock().unwrap();
    *fetch_flag = true;
    "Fetching started!"
}

/// # fetch_stop
///
/// Stop fetching news from the API.
#[openapi(tag = "News")]
#[get("/fetch_stop")]
pub async fn fetch_stop() -> &'static str {
    let mut fetch_flag = FETCH_FLAG.lock().unwrap();
    *fetch_flag = false;
    "Fetching stopped!"
}
