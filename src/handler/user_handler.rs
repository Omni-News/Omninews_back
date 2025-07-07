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
        user::request::LoginUserRequestDto,
    },
    service::user_service,
};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: verify_refresh_token, verify_access_token, login, logout]
}

/// # Verify refresh token
///
/// Returns a new access token if the refresh token is valid.
#[openapi(tag = "User")]
#[post("/refresh-token", data = "<refresh_token>")]
pub async fn verify_refresh_token(
    pool: &State<MySqlPool>,
    refresh_token: Json<VerifyRefreshTokenRequestDto>,
) -> Result<Json<AccessTokenResponseDto>, Status> {
    match user_service::validate_refresh_token(pool, refresh_token.into_inner()).await {
        Ok(token) => Ok(Json(token)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # Login
///
/// Returns a JWT token if the user credentials are valid or creates a new user and returns a JWT
/// token.
#[openapi(tag = "User")]
#[post("/login", data = "<user_data>")]
pub async fn login(
    pool: &State<MySqlPool>,
    user_data: Json<LoginUserRequestDto>,
) -> Result<Json<JwtTokenResponseDto>, Status> {
    match user_service::login_or_create_user(pool, user_data.into_inner()).await {
        Ok(token) => Ok(Json(token)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// #Logout
///
/// Removes the user's token from the database, effectively logging them out.
/// Returns `Status::Ok` if successful.
#[openapi(tag = "User")]
#[post("/logout")]
pub async fn logout(pool: &State<MySqlPool>, user: AuthenticatedUser) -> Result<Status, Status> {
    match user_service::delete_user_token(pool, user.user_email).await {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

// auth_middleware에서 access token은 검증됨.
/// # Verify access token
///
/// Returns `Status::Ok` if the access token is valid.
#[openapi(tag = "User")]
#[get("/access-token")]
pub async fn verify_access_token() -> Result<Status, Status> {
    Ok(Status::Ok)
}
