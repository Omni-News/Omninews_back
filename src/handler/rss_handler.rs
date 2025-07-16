use okapi::openapi3::OpenApi;
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sqlx::MySqlPool;

use crate::dto::rss::request::{CreateRssRequestDto, UpdateRssRankRequestDto};
use crate::dto::rss::response::{RssChannelResponseDto, RssItemResponseDto};
use crate::service::rss::{channel_service, item_service};
use crate::EmbeddingService;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_channel_id_by_rss_link,
        get_rss_channel_by_id, get_rss_item_by_channel_id, get_recommend_channel,
        get_recommend_item, get_rss_preview, is_rss_exist, create_channel, create_rss_all,
        update_rss_item_rank]
}

/// # RSS 채널 생성 API
///
/// 유효한 RSS 링크를 통해 새 RSS 채널을 생성합니다.
///
/// ## 요청 형식
/// ```json
/// {
///   "rss_link": "https://example.com/feed.xml"
/// }
/// ```
///
/// ## 성공 응답
/// ```json
/// 12345  // 생성된 채널 ID (정수)
/// ```
///
/// ## 실패 응답
/// - 상태 코드 400: 잘못된 요청 (비어있는 RSS 링크)
/// - 상태 코드 500: 서버 오류
#[openapi(tag = "RSS API")]
#[post("/rss/channel", data = "<link>")]
pub async fn create_channel(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    link: Json<CreateRssRequestDto>,
) -> Result<Json<i32>, Status> {
    if link.rss_link.is_empty() {
        return Err(Status::BadRequest);
    }

    match channel_service::create_rss_and_embedding(pool, model, link.into_inner()).await {
        Ok(channel_id) => Ok(Json(channel_id)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # RSS 일괄 생성 API
///
/// 여러 RSS 링크를 한 번에 처리하여 채널을 생성합니다.
///
/// ## 요청 형식
/// ```json
/// [
///   {"rss_link": "https://example.com/feed1.xml"},
///   {"rss_link": "https://example.com/feed2.xml"},
///   {"rss_link": "https://example.com/feed3.xml"}
/// ]
/// ```
///
/// ## 성공 응답
/// ```json
/// true  // 모든 RSS 링크가 성공적으로 처리됨
/// ```
///
/// ## 실패 응답
/// - 상태 코드 400: 잘못된 요청 (빈 배열)
/// - 상태 코드 500: 서버 오류
#[openapi(tag = "RSS API")]
#[post("/rss/all", data = "<links>")]
pub async fn create_rss_all(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    links: Json<Vec<CreateRssRequestDto>>,
) -> Result<Json<bool>, Status> {
    if links.is_empty() {
        return Err(Status::BadRequest);
    }

    match channel_service::create_rss_all(pool, model, links.into_inner()).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # RSS 링크로 채널 ID 조회 API
///
/// 제공된 RSS 링크와 연결된 채널 ID를 반환합니다.
///
/// ## 요청 파라미터
/// - channel_rss_link: RSS 피드 URL (예: "https://example.com/feed.xml")
///
/// ## 성공 응답
/// ```
/// json {
///     12345  // 채널 ID (정수)
/// }
/// ```
///
/// ## 실패 응답
/// - 상태 코드 500: 서버 오류 (채널을 찾을 수 없음)
#[openapi(tag = "RSS API")]
#[get("/rss/id?<channel_rss_link>")]
pub async fn get_channel_id_by_rss_link(
    pool: &State<MySqlPool>,
    channel_rss_link: String,
) -> Result<Json<i32>, Status> {
    match channel_service::find_rss_channel_by_rss_link(pool, channel_rss_link).await {
        Ok(res) => Ok(Json(res.channel_id.unwrap())),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # RSS 채널 상세정보 조회 API
///
/// 채널 ID로 RSS 채널 상세 정보를 조회합니다.
///
/// ## 요청 파라미터
/// - channel_id: 조회할 채널 ID (예: 12345)
///
/// ## 성공 응답
/// ```json
/// {
///   "channel_id": 12345,
///   "channel_title": "샘플 블로그",
///   "channel_description": "최신 기술 뉴스를 제공합니다",
///   "channel_link": "https://example.com/blog",
///   "channel_rss_link": "https://example.com/feed.xml",
///   "channel_language": "ko",
///   "channel_copyright": "© 2025 Example Corp",
///   "channel_managing_editor": "editor@example.com",
///   "channel_web_master": "webmaster@example.com",
///   "channel_pub_date": "2025-07-09T06:02:15Z",
///   "channel_last_build_date": "2025-07-09T06:00:00Z",
///   "channel_category": "기술",
///   "channel_generator": "WordPress",
///   "channel_docs": "https://example.com/rss-spec",
///   "channel_ttl": 60,
///   "channel_image_url": "https://example.com/logo.png",
///   "channel_image_title": "샘플 블로그 로고",
///   "channel_image_link": "https://example.com",
///   "channel_text_input_title": null,
///   "channel_text_input_description": null,
///   "channel_text_input_name": null,
///   "channel_text_input_link": null,
///   "channel_skip_hours": null,
///   "channel_skip_days": null,
///   "channel_rank": 0,
///   "channel_subscription_count": 42
/// }
/// ```
#[openapi(tag = "RSS API")]
#[get("/rss/channel?<channel_id>")]
pub async fn get_rss_channel_by_id(
    pool: &State<MySqlPool>,
    channel_id: i32,
) -> Result<Json<RssChannelResponseDto>, Status> {
    match channel_service::find_rss_channel_by_id(pool, channel_id).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # RSS 채널 아이템 조회 API
///
/// 특정 채널에 속한 RSS 아이템 목록을 조회합니다.
///
/// ## 요청 파라미터
/// - channel_id: 조회할 채널 ID (예: 12345)
///
/// ## 성공 응답
/// ```json
/// [
///   {
///     "item_id": 54321,
///     "item_title": "최신 기술 트렌드 분석",
///     "item_description": "2025년 기술 트렌드에 대한 종합 분석입니다.",
///     "item_link": "https://example.com/blog/trends-2025",
///     "item_author": "홍길동",
///     "item_category": "기술 트렌드",
///     "item_comments": "https://example.com/blog/trends-2025#comments",
///     "item_enclosure": null,
///     "item_guid": "https://example.com/blog/trends-2025",
///     "item_pub_date": "2025-07-08T15:30:00Z",
///     "item_source": "Example Blog",
///     "item_content_encoded": "<p>자세한 내용은 여기에...</p>",
///     "item_rank": 0,
///     "channel_id": 12345
///   },
///   {
///     "item_id": 54322,
///     "item_title": "인공지능의 미래",
///     "item_description": "인공지능 기술의 발전 방향에 대한 예측",
///     "item_link": "https://example.com/blog/ai-future",
///     "item_author": "김철수",
///     "item_category": "인공지능",
///     "item_comments": "https://example.com/blog/ai-future#comments",
///     "item_enclosure": null,
///     "item_guid": "https://example.com/blog/ai-future",
///     "item_pub_date": "2025-07-07T09:45:00Z",
///     "item_source": "Example Blog",
///     "item_content_encoded": "<p>인공지능의 미래는...</p>",
///     "item_rank": 0,
///     "channel_id": 12345
///   }
/// ]
/// ```
#[openapi(tag = "RSS API")]
#[get("/rss/items?<channel_id>")]
pub async fn get_rss_item_by_channel_id(
    pool: &State<MySqlPool>,
    channel_id: i32,
) -> Result<Json<Vec<RssItemResponseDto>>, Status> {
    match item_service::get_rss_item_by_channel_id(pool, channel_id).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 추천 RSS 채널 조회 API
///
/// 사용자에게 추천할 만한 RSS 채널 목록을 반환합니다.
///
/// ## 성공 응답
/// ```json
/// [
///   {
///     "channel_id": 12345,
///     "channel_title": "인기 기술 블로그",
///     "channel_description": "최신 기술 트렌드 정보",
///     "channel_link": "https://example.com/tech",
///     "channel_rss_link": "https://example.com/tech/feed.xml",
///     "channel_image_url": "https://example.com/logo.png",
///     "channel_subscription_count": 1580
///   },
///   {
///     "channel_id": 12346,
///     "channel_title": "디자인 트렌드",
///     "channel_description": "UX/UI 디자인 최신 정보",
///     "channel_link": "https://example.com/design",
///     "channel_rss_link": "https://example.com/design/feed.xml",
///     "channel_image_url": "https://example.com/design-logo.png",
///     "channel_subscription_count": 842
///   }
/// ]
/// ```
#[openapi(tag = "RSS API")]
#[get("/rss/recommend/channel")]
pub async fn get_recommend_channel(
    pool: &State<MySqlPool>,
) -> Result<Json<Vec<RssChannelResponseDto>>, Status> {
    match channel_service::get_recommend_channel(pool).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 추천 RSS 아이템 조회 API
///
/// 사용자에게 추천할 만한 RSS 아이템 목록을 반환합니다.
///
/// ## 성공 응답
/// ```json
/// [
///   {
///     "item_id": 54321,
///     "item_title": "2025년 개발자가 알아야 할 기술",
///     "item_description": "미래 기술 트렌드 분석",
///     "item_link": "https://example.com/blog/dev-skills-2025",
///     "item_author": "박지성",
///     "item_category": "개발",
///     "item_pub_date": "2025-07-08T10:15:00Z",
///     "item_rank": 250,
///     "channel_id": 12345
///   },
///   {
///     "item_id": 54345,
///     "item_title": "최신 클라우드 컴퓨팅 동향",
///     "item_description": "클라우드 서비스 비교 분석",
///     "item_link": "https://example.com/blog/cloud-trends",
///     "item_author": "김영희",
///     "item_category": "클라우드",
///     "item_pub_date": "2025-07-07T14:30:00Z",
///     "item_rank": 180,
///     "channel_id": 12348
///   }
/// ]
/// ```
#[openapi(tag = "RSS API")]
#[get("/rss/recommend/item")]
pub async fn get_recommend_item(
    pool: &State<MySqlPool>,
) -> Result<Json<Vec<RssItemResponseDto>>, Status> {
    match item_service::get_recommend_item(pool).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # RSS 미리보기 API
///
/// 제공된 RSS 링크의 내용을 미리볼 수 있는 채널 정보를 반환합니다.
///
/// ## 요청 파라미터
/// - rss_link: 미리볼 RSS 피드 URL (예: "https://example.com/feed.xml")
///
/// ## 성공 응답
/// ```json
/// {
///   "channel_id": null,
///   "channel_title": "샘플 블로그",
///   "channel_description": "최신 기술 뉴스를 제공합니다",
///   "channel_link": "https://example.com/blog",
///   "channel_rss_link": "https://example.com/feed.xml",
///   "channel_language": "ko",
///   "channel_copyright": "© 2025 Example Corp",
///   "channel_image_url": "https://example.com/logo.png",
///   "channel_image_title": "샘플 블로그 로고",
///   "channel_image_link": "https://example.com"
/// }
/// ```
#[openapi(tag = "RSS API")]
#[get("/rss/preview?<rss_link>")]
pub async fn get_rss_preview(
    pool: &State<MySqlPool>,
    rss_link: String,
) -> Result<Json<RssChannelResponseDto>, Status> {
    match channel_service::get_rss_preview(pool, rss_link).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # RSS 존재 여부 확인 API
///
/// 제공된 RSS 링크가 이미 등록되어 있는지 확인합니다.
///
/// ## 요청 파라미터
/// - rss_link: 확인할 RSS 피드 URL (예: "https://example.com/feed.xml")
///
/// ## 성공 응답
/// ```json
/// true  // RSS 채널이 이미 존재함
/// ```
/// 또는
/// ```json
/// false  // RSS 채널이 존재하지 않음
/// ```
#[openapi(tag = "RSS API")]
#[get("/rss/exist?<rss_link>")]
pub async fn is_rss_exist(pool: &State<MySqlPool>, rss_link: String) -> Result<Json<bool>, Status> {
    match channel_service::is_channel_exist_by_link(pool, rss_link).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # RSS 아이템 평가 순위 업데이트 API
///
/// RSS 아이템의 인기도 순위를 업데이트합니다.
///
/// ## 요청 형식
/// ```json
/// {
///   "item_id": 54321,
///   "rank": 10
/// }
/// ```
///
/// ## 성공 응답
/// - 상태 코드 200: 성공적으로 업데이트됨
///
/// ## 실패 응답
/// - 상태 코드 500: 서버 오류
#[openapi(tag = "RSS API")]
#[put("/rss/item/rank", data = "<update_rss_rank>")]
pub async fn update_rss_item_rank(
    pool: &State<MySqlPool>,
    update_rss_rank: Json<UpdateRssRankRequestDto>,
) -> Result<Status, Status> {
    match item_service::update_rss_item_rank(pool, update_rss_rank.into_inner()).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}
