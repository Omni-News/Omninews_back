use rocket::State;
use sqlx::{query, MySqlPool};

use crate::{db_util::get_db, model::user::NewUser};

pub async fn select_user_by_email(
    pool: &State<MySqlPool>,
    user_email: String,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!("SELECT user_id FROM user WHERE user_email = ?", user_email)
        .fetch_one(&mut *conn)
        .await;

    match result {
        Ok(res) => Ok(res.user_id),
        Err(e) => Err(e),
    }
}

pub async fn insert_user(pool: &State<MySqlPool>, user: NewUser) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "INSERT INTO user 
            (user_email, user_display_name, user_photo_url, user_social_login_provider,
            user_social_provider_id, user_access_token, user_refresh_token, user_access_token_expires_at,
            user_refresh_token_expires_at, user_notification_push, user_last_active_at,
            user_created_at, user_updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        user.user_email,
        user.user_display_name,
        user.user_photo_url,
        user.user_social_login_provider,
        user.user_social_provider_id,
        user.user_access_token,
        user.user_refresh_token,
        user.user_access_token_expires_at,
        user.user_refresh_token_expires_at,
        user.user_notification_push,
        user.user_last_active_at,
        user.user_created_at,
        user.user_updated_at,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => Err(e),
    }
}

pub async fn delete_user_token_by_email(
    pool: &State<MySqlPool>,
    user_email: String,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "UPDATE user
            SET user_access_token = NULL, user_refresh_token = NULL,
                user_access_token_expires_at = NULL, user_refresh_token_expires_at = NULL
        WHERE user_email = ?",
        user_email
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.rows_affected() as i32),
        Err(e) => Err(e),
    }
}
