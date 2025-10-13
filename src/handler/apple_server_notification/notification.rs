use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};

use crate::dto::apple_server_notification::request::AppleServerNotificationRequestDto;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings:
        apple_server_notification,
    ]
}

#[openapi(tag = "Apple")]
#[post("/apple/notification/v2", data = "<data>")]
/// # Apple Server Notification API
/// 구독 등 변경사항 있을 시, Apple Server에서 관련 내용을 보내주는 API입니다.
///
pub async fn apple_server_notification(
    data: Json<AppleServerNotificationRequestDto>,
) -> Result<Status, Status> {
    // TODO: Apple에서 signedPayload를 주면, 이를 디코딩해서 사용하면 됨.
    info!("data: {:?}", data.into_inner());
    Ok(Status::Ok)
}
