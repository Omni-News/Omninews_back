use rocket::{http::Status, serde::json::Json, State};
use sqlx::MySqlPool;

use crate::{
    model::{
        token::JwtToken,
        user::{ParamUser, UserEmail},
    },
    service::user_service,
};

#[post("/user", data = "<user_data>")]
pub async fn create_user(
    pool: &State<MySqlPool>,
    user_data: Json<ParamUser>,
) -> Result<Json<JwtToken>, Status> {
    match user_service::create_user(pool, user_data.into_inner()).await {
        Ok(token) => Ok(Json(token)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/user/logout", data = "<user_email>")]
pub async fn logout(
    pool: &State<MySqlPool>,
    user_email: Json<UserEmail>,
) -> Result<Status, Status> {
    match user_service::delete_user_token(
        pool,
        user_email.into_inner().user_email.unwrap_or_default(),
    )
    .await
    {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}
