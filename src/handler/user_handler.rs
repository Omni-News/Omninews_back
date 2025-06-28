use rocket::{http::Status, serde::json::Json, State};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::AuthenticatedUser,
    model::{
        token::{AccessToken, JwtToken, RefreshTokenAndUserEmail},
        user::ParamUser,
    },
    service::user_service,
};

// auth middleware에서 access token을 검증하고, 유효한 경우 Status::Ok를 반환합니다.
#[get("/user/access-token")]
pub async fn verify_access_token() -> Result<Status, Status> {
    Ok(Status::Ok)
}

#[post("/user/refresh-token", data = "<refresh_token>")]
pub async fn verify_refresh_token(
    pool: &State<MySqlPool>,
    refresh_token: Json<RefreshTokenAndUserEmail>,
) -> Result<Json<AccessToken>, Status> {
    match user_service::validate_refresh_token(pool, refresh_token.into_inner()).await {
        Ok(token) => Ok(Json(token)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/user/login", data = "<user_data>")]
pub async fn login(
    pool: &State<MySqlPool>,
    user_data: Json<ParamUser>,
) -> Result<Json<JwtToken>, Status> {
    match user_service::login_or_create_user(pool, user_data.into_inner()).await {
        Ok(token) => Ok(Json(token)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/user/logout")]
pub async fn logout(pool: &State<MySqlPool>, user: AuthenticatedUser) -> Result<Status, Status> {
    match user_service::delete_user_token(pool, user.user_email).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}
