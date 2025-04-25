use rocket::{http::Status, serde::json::Json, State};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::AuthenticatedUser,
    model::{token::JwtToken, user::ParamUser},
    service::user_service,
};

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
