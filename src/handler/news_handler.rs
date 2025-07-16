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

/// # 뉴스 카테고리별 조회 API
///
/// 제공된 카테고리에 해당하는 뉴스 목록을 반환합니다.
///
/// ## 요청 파라미터
/// - `category`: 조회할 뉴스 카테고리 (예: "business", "technology", "sports")
///
/// ## 성공 응답
/// ```
/// [
///   {
///     "news_id": 123,
///     "title": "최신 기술 트렌드 분석",
///     "description": "올해 주목해야 할 기술 트렌드에 대한 분석...",
///     "url": "https://example.com/news/123",
///     "image_url": "https://example.com/images/news123.jpg",
///     "published_at": "2025-07-09T05:30:00Z",
///     "source": "Tech Daily",
///     "category": "technology"
///   }
/// ]
/// ```
///
/// ## 실패 응답
/// - 상태 코드 500: 서버 오류
#[openapi(tag = "뉴스 API")]
#[get("/news?<category>")]
pub async fn get_news(
    pool: &State<MySqlPool>,
    category: String,
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
/// ## 요청 파라미터
/// - `q`: 검색어 (예: "climate change")
/// - `language`: 뉴스 언어 (예: "ko", "en")
/// - `country`: 뉴스 국가 코드 (예: "kr", "us")
/// - `category`: 뉴스 카테고리 (예: "business", "technology")
/// - `page_size`: 페이지당 결과 수 (예: 10)
/// - `page`: 페이지 번호 (예: 1)
///
/// ## 성공 응답
/// [
///   {
///     "title": "기후 변화에 관한 최신 연구",
///     "description": "세계 과학자들이 발표한 기후 변화 연구 결과...",
///     "url": "https://example.com/news/climate",
///     "image_url": "https://example.com/images/climate.jpg",
///     "published_at": "2025-07-08T14:25:00Z",
///     "source": "Science Daily",
///     "category": "environment"
///   }
/// ]
///
/// ## 실패 응답
/// - 상태 코드 500: API 호출 실패 또는 서버 오류
#[openapi(tag = "뉴스 API")]
#[get("/news/api?<params..>")]
pub async fn get_news_by_api(
    params: NewsRequestDto,
) -> Result<Json<Vec<NewsApiResponseDto>>, Status> {
    match news_service::get_news_by_api(params).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 뉴스 수집 시작 API
///
/// 외부 API에서 뉴스를 자동으로 수집하는 프로세스를 시작합니다.
///
/// ## 성공 응답
/// - 텍스트 메시지: "Fetching started!"
///
/// ## 사용 시나리오
/// - 뉴스 수집이 중단된 후 다시 시작하고자 할 때 사용
/// - 관리자 권한이 필요한 API입니다
#[openapi(tag = "뉴스 관리 API")]
#[get("/news/fetch_start")]
pub async fn fetch_start() -> &'static str {
    let mut fetch_flag = FETCH_FLAG.lock().unwrap();
    *fetch_flag = true;
    "Fetching started!"
}

/// # 뉴스 수집 중지 API
///
/// 외부 API에서 뉴스를 자동으로 수집하는 프로세스를 중지합니다.
///
/// ## 성공 응답
/// - 텍스트 메시지: "Fetching stopped!"
///
/// ## 사용 시나리오
/// - 시스템 유지보수나 리소스 관리를 위해 일시적으로 뉴스 수집을 중단할 때 사용
/// - 관리자 권한이 필요한 API입니다
#[openapi(tag = "뉴스 관리 API")]
#[get("/news/fetch_stop")]
pub async fn fetch_stop() -> &'static str {
    let mut fetch_flag = FETCH_FLAG.lock().unwrap();
    *fetch_flag = false;
    "Fetching stopped!"
}
