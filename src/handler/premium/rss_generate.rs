use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::AuthenticatedUser,
    config::webdriver::DriverPool,
    dto::premium::rss::{
        request::{RssGenerateByCssReqeustDto, RssGenerateRequestDto},
        response::RssGenerateResponseDto,
    },
    service::{premium::rss::generate_rss, user_service},
    utils::embedding_util::EmbeddingService,
};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: rss_generate, rss_generate_by_css]
}

#[openapi(tag = "Premium RSS Generation API")]
#[post("/premium/rss/generate", data = "<data>")]
/// # RSS Generation API
///
/// 사이트 종류와 링크를 입력받아 RSS 피드를 생성합니다.
///
/// ### `channel_link`: Rss 피드를 생성할 사이트의 링크나 유저 명
/// ### `kind`: 사이트 종류 (예: "Instagram", "Medium", "Naver" 등)
pub async fn rss_generate(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
    driver_pool: &State<DriverPool>,
    data: Json<RssGenerateRequestDto>,
    user: AuthenticatedUser,
) -> Result<Json<RssGenerateResponseDto>, Status> {
    // premium 유저 검증
    if let Ok(res) = user_service::validate_premium_user(pool, &user.user_email).await {
        if !res {
            return Err(Status::InternalServerError);
        }
    }

    match generate_rss::generate(pool, embedding_service, driver_pool, data.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[openapi(tag = "Premium RSS Generation API")]
#[post("/premium/rss/generate_by_css", data = "<data>")]
/// # RSS Generation by CSS API
///
/// 사이틀의 CSS 선택자를 제공받아 RSS 피드를 생성합니다.
/// 채널의 정보는 사용자에게 직접 입력받고, item의 정보는 CSS 선택자를 통해 추출합니다.
///
/// ### `channel_link`: Rss 피드를 생성할 사이트의 링크
/// ### `channel_image_link`: 채널 이미지 링크
/// ### `channel_title`: 채널 제목
/// ### `channel_description`: 채널 설명
/// ### `channel_language`: 채널 언어
///
/// ### `item_title_css`: 아이템 제목의 CSS 선택자
/// ### `item_description_css`: 아이템 설명의 CSS 선택자
/// ### `item_link_css`: 아이템 링크의 CSS 선택자
/// ### `item_author_css`: 아이템 작성자의 CSS 선택자
/// ### `item_pub_date_css`: 아이템 게시 날짜의 CSS 선택자
/// ### `item_image_css`: 아이템 이미지의 CSS 선택자
pub async fn rss_generate_by_css(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
    driver_pool: &State<DriverPool>,
    data: Json<RssGenerateByCssReqeustDto>,
    user: AuthenticatedUser,
) -> Result<Json<RssGenerateResponseDto>, Status> {
    // premium 유저 검증
    if let Ok(res) = user_service::validate_premium_user(pool, &user.user_email).await {
        if !res {
            return Err(Status::InternalServerError);
        }
    }

    match generate_rss::rss_by_css(pool, embedding_service, driver_pool, data.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
