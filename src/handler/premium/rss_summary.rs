use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::AuthenticatedUser,
    dto::premium::rss::{request::RssSummaryRequestDto, response::RssSummaryResponseDto},
    service::{premium::rss::summary_rss, user_service},
};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: rss_summary]
}

#[openapi(tag = "Premium RSS")]
#[post("/premium/rss/summary", data = "<data>")]
/// # Rss Summary API
///
/// 링크가 주어지면, 해당 링크의 본문을 요약해서 반환합니다.
///
/// ### `rss_link`: Rss 링크
///
pub async fn rss_summary(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    data: Json<RssSummaryRequestDto>,
) -> Result<Json<RssSummaryResponseDto>, Status> {
    // premium 유저 검증
    if let Ok(res) = user_service::validate_premium_user(pool, &user.user_email).await {
        if !res {
            return Err(Status::InternalServerError);
        }
    }

    match summary_rss::summary(pool, data.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}
