use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::AuthenticatedUser,
    dto::omninews_subscription::{
        request::OmninewsReceiptRequestDto, response::OmninewsSubscriptionResponseDto,
    },
    service::omninews_subscription_service,
};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: verify_subscription, register_subscription]
}

#[openapi(tag = "OmniNews Subscription API")]
#[get("/subscription/verify?<is_sandbox>")]
///
/// # 사용자 구독 검증 API
///
/// 로그인한 사용자의 OmniNews 구독 상태를 확인합니다.
///
async fn verify_subscription(
    pool: &State<MySqlPool>,
    auth: AuthenticatedUser,
    is_sandbox: Option<bool>,
) -> Result<Json<OmninewsSubscriptionResponseDto>, Status> {
    match omninews_subscription_service::verify_subscription(
        pool,
        &auth.user_email,
        is_sandbox.unwrap_or_default(),
    )
    .await
    {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[openapi(tag = "OmniNews Subscription API")]
#[post("/subscription/register", data = "<subscription>")]
///
/// # 사용자 구독 등록 API
///
/// 사용자의 구독 정보를 등록합니다.
///
/// ### `receipt_data` : Base64로 인코딩된 영수증 데이터 (ex. "base64_encoded_receipt_data")
/// ### `platform` : 구독 플랫폼 (ex. "ios", "android")
/// ### `is_test` : 테스트 환경 여부 (ex. true, false)
///
async fn register_subscription(
    pool: &State<MySqlPool>,
    subscription: Json<OmninewsReceiptRequestDto>,
    auth: AuthenticatedUser,
) -> Result<Json<bool>, Status> {
    match omninews_subscription_service::register_subscription(
        pool,
        &auth.user_email,
        subscription.into_inner(),
    )
    .await
    {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(Status::InternalServerError),
    }
}
