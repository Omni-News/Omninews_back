use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::{
    db::get_db,
    model::feedback::{Feedback, NewFeedback},
};

pub async fn insert_feedback(
    pool: &State<MySqlPool>,
    feedback: NewFeedback,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "INSERT INTO feedback (feedback_email, feedback_content) VALUES (?, ?);",
        feedback.feedback_email,
        feedback.feedback_content,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => Err(e),
    }
}

pub async fn select_feedbacks(pool: &State<MySqlPool>) -> Result<Vec<Feedback>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(Feedback, "SELECT * FROM feedback")
        .fetch_all(&mut *conn)
        .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}
