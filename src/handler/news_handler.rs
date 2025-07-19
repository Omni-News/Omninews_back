use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::AuthenticatedUser,
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

/// # 뉴스 카테고리별 조회 API
///
/// 제공된 카테고리에 해당하는 뉴스 목록을 반환합니다.
///
/// ### `category` : 뉴스 카테고리 (예: "정치", "경제", "사회", "생활/문화", "세계", "IT/과학")
///
#[openapi(tag = "뉴스 API")]
#[get("/news?<category>")]
pub async fn get_news(
    pool: &State<MySqlPool>,
    category: String,
    _auth: AuthenticatedUser,
) -> Result<Json<Vec<NewsResponseDto>>, Status> {
    match news_service::get_news(pool, category).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 외부 API 뉴스 조회 API
///
/// 제공된 파라미터에 따라 외부 API에서 뉴스 목록을 검색하여 반환합니다.
///
/// ### `query` : 검색어 (예: "AI", "경제")
///
/// ### `display` : 뉴스 개수 (예: 1 ~ 1000)
///
/// ### `sort` : 정렬 기준 (예: sim => 정확도순, date => 날짜순)
///
#[openapi(tag = "뉴스 API")]
#[get("/news/api?<params..>")]
pub async fn get_news_by_api(
    params: NewsRequestDto,
    _auth: AuthenticatedUser,
) -> Result<Json<Vec<NewsApiResponseDto>>, Status> {
    match news_service::get_news_by_api(params).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

// TODO fetch start, stop은 관리자 권한 따로 만들어서 관리하기
/// # 뉴스 수집 시작 API
///
/// 외부 API에서 뉴스를 자동으로 수집하는 프로세스를 시작합니다.
///
/// ### 기능 설명
///
/// fetch 시작 시 5분마다 뉴스 크롤링 및 가공 후 db 저장.
///
#[openapi(tag = "뉴스 관리 API")]
#[get("/news/fetch_start")]
pub async fn fetch_start(_auth: AuthenticatedUser) -> &'static str {
    let mut fetch_flag = FETCH_FLAG.lock().unwrap();
    *fetch_flag = true;
    "Fetching started!"
}

/// # 뉴스 수집 중지 API
///
/// 외부 API에서 뉴스를 자동으로 수집하는 프로세스를 중지합니다.
///
///
#[openapi(tag = "뉴스 관리 API")]
#[get("/news/fetch_stop")]
pub async fn fetch_stop(_auth: AuthenticatedUser) -> &'static str {
    let mut fetch_flag = FETCH_FLAG.lock().unwrap();
    *fetch_flag = false;
    "Fetching stopped!"
}
