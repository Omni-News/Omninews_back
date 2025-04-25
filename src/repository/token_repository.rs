use sqlx::{query, MySqlPool};

use crate::db_util::get_db_scheduler;

pub async fn validate_token_user(pool: &MySqlPool, token: String) -> Result<String, sqlx::Error> {
    let mut conn = get_db_scheduler(pool).await?;

    let result = query!(
        "SELECT user_email FROM user
        WHERE user_access_token = ?",
        token
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.user_email),
        Err(_) => Err(sqlx::Error::RowNotFound),
    }
}
