use crate::model::{NewUser, User};
use crate::service::user_service;
use r2d2::Pool;
use r2d2_mysql::MySqlConnectionManager;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

#[post("/user", data = "<user>")]
pub async fn create_user(
    pool: &State<Pool<MySqlConnectionManager>>,
    user: Json<NewUser>,
) -> Result<Status, Status> {
    match user_service::create_user(user.into_inner(), pool).await {
        Ok(_) => Ok(Status::Created),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/user")]
pub async fn get_users(
    pool: &State<Pool<MySqlConnectionManager>>,
) -> Result<Json<Vec<User>>, Status> {
    match user_service::get_users(pool).await {
        Ok(users) => Ok(Json(users)),
        Err(_) => Err(Status::InternalServerError),
    }
}
