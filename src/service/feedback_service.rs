use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{
        error::OmniNewsError,
        feedback::{Feedback, NewFeedback},
    },
    repository::feedback_repository,
};

pub async fn insert_feedback(
    pool: &State<MySqlPool>,
    feedback: NewFeedback,
) -> Result<i32, OmniNewsError> {
    feedback_repository::insert_feedback(pool, feedback)
        .await
        .map_err(|e| {
            error!("Error inserting feedback: {:?}", e);
            OmniNewsError::Database(e)
        })
}

pub async fn find_feedbacks(pool: &State<MySqlPool>) -> Result<Vec<Feedback>, OmniNewsError> {
    feedback_repository::select_feedbacks(pool)
        .await
        .map_err(|e| {
            error!("Error finding feedbacks: {:?}", e);
            OmniNewsError::Database(e)
        })
}
