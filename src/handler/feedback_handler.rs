use rocket::serde::json::Json;
use rocket::{http::Status, State};
use sqlx::MySqlPool;

use crate::model::feedback::{Feedback, NewFeedback};
use crate::service::feedback_service;

#[get("/feedback")]
pub async fn get_feedbacks(pool: &State<MySqlPool>) -> Result<Json<Vec<Feedback>>, Status> {
    match feedback_service::find_feedbacks(pool).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/feedback", data = "<feedback>")]
pub async fn create_feedback(
    pool: &State<MySqlPool>,
    feedback: Json<NewFeedback>,
) -> Result<&str, Status> {
    match feedback_service::insert_feedback(pool, feedback.into_inner()).await {
        Ok(_) => Ok("Success"),
        Err(_) => Err(Status::InternalServerError),
    }
}
