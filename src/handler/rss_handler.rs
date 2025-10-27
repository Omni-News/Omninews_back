use okapi::openapi3::OpenApi;
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use sqlx::MySqlPool;

use crate::auth_middleware::AuthenticatedUser;
use crate::dto::rss::request::{CreateRssRequestDto, RssItemRequestDto, UpdateRssRankRequestDto};
use crate::dto::rss::response::{RssChannelResponseDto, RssItemResponseDto};
use crate::service::{channel_service, item_service};
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
/// ### `rss_link` : RSS 피드 URL (예: "https://example.com/feed.xml")
///
#[openapi(tag = "RSS API")]
#[post("/rss/channel", data = "<link>")]
pub async fn create_channel(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    link: Json<CreateRssRequestDto>,
    _auth: AuthenticatedUser,
) -> Result<Json<i32>, Status> {
    if link.rss_link.is_empty() {
        return Err(Status::BadRequest);
    }

    match channel_service::create_rss_and_embedding(pool, model, link.into_inner().rss_link).await {
        Ok(channel_id) => Ok(Json(channel_id)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # RSS 일괄 생성 API
///
/// 여러 RSS 링크를 한 번에 처리하여 채널을 생성합니다.
///
/// ### `[ { rss_link` : RSS 피드 URL (예: "https://example.com/feed.xml") }, { rss_link: ... } ]
///
#[openapi(tag = "RSS API")]
#[post("/rss/all", data = "<links>")]
pub async fn create_rss_all(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    links: Json<Vec<CreateRssRequestDto>>,
    _auth: AuthenticatedUser,
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
/// ### `channel_rss_link` : RSS 피드 URL (예: "https://example.com/feed.xml")
///
#[openapi(tag = "RSS API")]
#[get("/rss/id?<channel_rss_link>")]
pub async fn get_channel_id_by_rss_link(
    pool: &State<MySqlPool>,
    channel_rss_link: String,
    _auth: AuthenticatedUser,
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
/// ### `channel_id` : 조회할 채널 ID (예: 3)
///
#[openapi(tag = "RSS API")]
#[get("/rss/channel?<channel_id>")]
pub async fn get_rss_channel_by_id(
    pool: &State<MySqlPool>,
    channel_id: i32,
    _auth: AuthenticatedUser,
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
/// ### `channel_id` : 조회할 채널 ID (예: 3)
/// ### `page` : 조회할 page 번호 (예: 1)
///
#[openapi(tag = "RSS API")]
#[get("/rss/items?<data..>")]
pub async fn get_rss_item_by_channel_id(
    pool: &State<MySqlPool>,
    data: RssItemRequestDto,
    _auth: AuthenticatedUser,
) -> Result<Json<Vec<RssItemResponseDto>>, Status> {
    match item_service::get_rss_item_by_channel_id_pagenation(pool, data).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

// TODO 추천 채널 기능 변경 후 여기 내용 추가하기
/// # 추천 RSS 채널 조회 API
///
/// 사용자에게 추천하는 RSS 채널 목록을 반환합니다.
///
/// ## 기능 설명
/// 랭크 50순위 채널에서 20개 랜덤 반환
///
#[openapi(tag = "RSS API")]
#[get("/rss/recommend/channel")]
pub async fn get_recommend_channel(
    pool: &State<MySqlPool>,
    _auth: AuthenticatedUser,
) -> Result<Json<Vec<RssChannelResponseDto>>, Status> {
    match channel_service::get_recommend_channel(pool).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

// TODO 추천 rss 기능 변경 후 여기 내용 추가하기
/// # 추천 RSS 아이템 조회 API
///
/// 사용자에게 추천하는 RSS 아이템 목록을 반환합니다.
///
/// ## 기능 설명
/// 상위 100개 중 50개 랜덤 반환
///
#[openapi(tag = "RSS API")]
#[get("/rss/recommend/item")]
pub async fn get_recommend_item(
    pool: &State<MySqlPool>,
    _auth: AuthenticatedUser,
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
/// ### `rss_link` : 미리볼 RSS 피드 URL (예: "https://example.com/feed.xml")
///
#[openapi(tag = "RSS API")]
#[get("/rss/preview?<rss_link>")]
pub async fn get_rss_preview(
    pool: &State<MySqlPool>,
    rss_link: String,
    _auth: AuthenticatedUser,
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
/// ### `rss_link` : 확인할 RSS 피드 URL (예: "https://example.com/feed.xml")
///
#[openapi(tag = "RSS API")]
#[get("/rss/exist?<rss_link>")]
pub async fn is_rss_exist(
    pool: &State<MySqlPool>,
    rss_link: String,
    _auth: AuthenticatedUser,
) -> Result<Json<bool>, Status> {
    match channel_service::is_channel_exist_by_link(pool, rss_link).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # RSS 아이템 평가 순위 업데이트 API
///
/// RSS 아이템의 인기도 순위를 업데이트합니다.
///
/// ### `rss_id` : 업데이트할 RSS 아이템 ID
///
/// ### `num` : 업데이트할 값 (예: 1, -1)
///
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
