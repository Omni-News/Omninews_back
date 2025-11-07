use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::AuthenticatedUser,
    dto::{
        auth::{
            request::VerifyRefreshTokenRequestDto,
            response::{AccessTokenResponseDto, JwtTokenResponseDto},
        },
        user::{
            request::{
                AppleLoginRequestDto, DemoLoginRequestDto, LoginUserRequestDto,
                UserNotificationRequestDto, UserThemeRequestDto,
            },
            response::UserThemeResponseDto,
        },
    },
    service::user_service,
};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: demo_login, verify_refresh_token, verify_access_token, login, apple_login, logout, notification_setting, get_user_theme, user_theme_setting, remove_user]
}

/// # 리프레시 토큰 검증 API
///
/// 유효한 리프레시 토큰으로 새 액세스 토큰을 발급합니다.
///
/// 클라이언트에서 로그인 실패 시 refresh token 재발급 요청
///
/// ### `token` : 클라이언트가 가지고 있는 refresh token
///
/// ### `email` : 클라이언트의 email 주소
///
#[openapi(tag = "인증 API")]
#[post("/user/refresh-token", data = "<refresh_token>")]
pub async fn verify_refresh_token(
    pool: &State<MySqlPool>,
    refresh_token: Json<VerifyRefreshTokenRequestDto>,
) -> Result<Json<AccessTokenResponseDto>, Status> {
    match user_service::validate_refresh_token(pool, refresh_token.into_inner()).await {
        Ok(token) => Ok(Json(token)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 데모 로그인 API
/// 리뷰어어를 위한 데모 로그인 API입니다.
///
/// ### `user_email` : 사용자의 이메일 주소
/// ### `user_password`: 사용자의 비밀번호
/// ### `expired`: 구독 만료 여부
#[openapi(tag = "인증 API")]
#[post("/user/demo_login", data = "<data>")]
pub async fn demo_login(
    pool: &State<MySqlPool>,
    data: Json<DemoLoginRequestDto>,
) -> Result<Json<JwtTokenResponseDto>, Status> {
    match user_service::demo_login(pool, data.into_inner()).await {
        Ok(token) => Ok(Json(token)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 로그인/회원가입 API
///
/// 사용자 소셜 로그인 정보로 로그인 또는 신규 가입을 처리합니다.
///
/// ### `user_email` : 사용자의 이메일 주소
///
/// ### `user_display_name` : 사용자의 표시 이름
///
/// ### `user_photo_url` : 사용자의 프로필 사진 URL ( 소셜 로그인 제공자에서 제공 )
///
/// ### `user_social_login_provider` : 소셜 로그인 제공자 (예: google, kakao 등)
///
/// ### `user_social_provider_id` : 소셜 로그인 제공자의 고유 ID ( 소셜 로그인 제공자에서 제공 )
///
/// ### `user_platform` : 사용자의 플랫폼 (예: ios, android 등)
///
#[openapi(tag = "인증 API")]
#[post("/user/login", data = "<user_data>")]
pub async fn login(
    pool: &State<MySqlPool>,
    user_data: Json<LoginUserRequestDto>,
) -> Result<Json<JwtTokenResponseDto>, Status> {
    match user_service::login_or_create_user(pool, user_data.into_inner()).await {
        Ok(token) => Ok(Json(token)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 애플 로그인 API
///
/// 애플의 최초 회원가입이 아닐 시 사용하는 API입니다.
///
/// ### `user_social_provider_id` : 애플에서 발급한 고유 ID (로그인에 사용됩니다.)
///
#[openapi(tag = "인증 API")]
#[post("/user/apple/login", data = "<user_data>")]
pub async fn apple_login(
    pool: &State<MySqlPool>,
    user_data: Json<AppleLoginRequestDto>,
) -> Result<Json<JwtTokenResponseDto>, Status> {
    match user_service::apple_login(pool, user_data.into_inner()).await {
        Ok(token) => Ok(Json(token)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 로그아웃 API
///
/// 현재 로그인된 사용자의 액세스 토큰을 삭제합니다.
///
#[openapi(tag = "인증 API")]
#[post("/user/logout")]
pub async fn logout(pool: &State<MySqlPool>, user: AuthenticatedUser) -> Result<Status, Status> {
    match user_service::delete_user_token(pool, user.user_email).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 액세스 토큰 검증 API
///
/// 현재 액세스 토큰의 유효성을 확인합니다.
///
#[openapi(tag = "인증 API")]
#[get("/user/access-token")]
pub async fn verify_access_token(_auth: AuthenticatedUser) -> Result<Status, Status> {
    Ok(Status::Ok)
}

/// # 사용자 회원 탈퇴 API
///
/// 회원 탈퇴 API입니다.
///
#[openapi(tag = "인증 API")]
#[delete("/user/delete")]
pub async fn remove_user(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Status, Status> {
    match user_service::remove_user(pool, user.user_email).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}
/// # 사용자 알림 설정 API
///
/// 사용자가 알림을 설정했을 때 사용되는 API입니다.
///
/// ### `user_notification_push` : 사용자 알림 푸시 설정 (true/false)
///
/// ### `user_fcm_token` : 사용자 FCM 토큰 (Firebase Cloud Messaging Token)
///
#[openapi(tag = "알림 API")]
#[post("/user/notification", data = "<notification_data>")]
pub async fn notification_setting(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
    notification_data: Json<UserNotificationRequestDto>,
) -> Result<Status, Status> {
    match user_service::update_user_notification_setting(
        pool,
        user.user_email,
        notification_data.into_inner(),
    )
    .await
    {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 사용자 테마 확인 API
///
/// 사용자의 현재 테마를 확인  API입니다.
///
#[openapi(tag = " 테마 API")]
#[get("/user/theme")]
pub async fn get_user_theme(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<UserThemeResponseDto>, Status> {
    match user_service::get_user_theme(pool, user.user_email).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 사용자 테마 설정 API
///
/// 사용자의 테마를 변경하는  API입니다.
///
/// ### `theme` : 변경할 테마 이름 (예: paper)
///
#[openapi(tag = " 테마 API")]
#[post("/user/theme", data = "<theme_data>")]
pub async fn user_theme_setting(
    pool: &State<MySqlPool>,
    theme_data: Json<UserThemeRequestDto>,
    user: AuthenticatedUser,
) -> Result<Status, Status> {
    match user_service::update_user_theme(pool, user.user_email, theme_data.into_inner()).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}
