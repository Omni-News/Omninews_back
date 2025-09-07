use chrono::NaiveDateTime;
use sqlx::{query, MySqlPool};

use crate::{
    db_util::get_db,
    model::{auth::JwtToken, user::NewUser},
};

pub async fn insert_user(pool: &MySqlPool, user: NewUser) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "INSERT INTO user 
            (user_email, user_display_name, user_photo_url, user_social_login_provider,
            user_social_provider_id, user_access_token, user_refresh_token, user_access_token_expires_at,
            user_refresh_token_expires_at, user_last_active_at,
            user_created_at, user_updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )",
        user.user_email,
        user.user_display_name,
        user.user_photo_url,
        user.user_social_login_provider,
        user.user_social_provider_id,
        user.user_access_token,
        user.user_refresh_token,
        user.user_access_token_expires_at,
        user.user_refresh_token_expires_at,
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
pub async fn select_user_id_by_email(
    pool: &MySqlPool,
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

pub async fn validate_premium_user(
    pool: &MySqlPool,
    user_email: &str,
) -> Result<bool, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "SELECT user_email FROM user WHERE user_email = ? AND user_subscription_plan <> 0 ",
        user_email
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(_) => Ok(true),
        Err(sqlx::Error::RowNotFound) => Ok(false),
        Err(e) => Err(e),
    }
}

pub async fn select_user_email_by_social_provider_id(
    pool: &MySqlPool,
    user_social_provider_id: String,
) -> Result<String, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "SELECT user_email FROM user WHERE user_social_provider_id = ?",
        user_social_provider_id
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.user_email),
        Err(e) => Err(e),
    }
}

pub async fn delete_user_token_by_email(
    pool: &MySqlPool,
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

pub async fn validate_users_all_tokens(
    pool: &MySqlPool,
    user_email: String,
) -> Result<(bool, bool), sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let access_result = query!(
        "SELECT user_email FROM user
        WHERE user_email = ? AND user_access_token IS NOT NULL AND user_access_token_expires_at > NOW()",
        user_email
    )
    .fetch_one(&mut *conn)
    .await;

    let refresh_result = query!(
        "SELECT user_email FROM user
        WHERE user_email = ? AND user_refresh_token IS NOT NULL AND user_refresh_token_expires_at > NOW()",
        user_email
    )
    .fetch_one(&mut *conn)
    .await;

    match (access_result, refresh_result) {
        (Ok(_), Ok(_)) => Ok((true, true)),
        (Ok(_), Err(_)) => Ok((true, false)),
        (Err(_), Ok(_)) => Ok((false, true)),
        (Err(_), Err(_)) => Ok((false, false)),
    }
}

pub async fn validate_access_token_by_user_email(
    pool: &MySqlPool,
    token: String,
    email: String,
) -> Result<String, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "SELECT user_email FROM user
        WHERE user_access_token = ? AND user_email = ?",
        token,
        email
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.user_email),
        Err(_) => Err(sqlx::Error::RowNotFound),
    }
}

pub async fn validate_refresh_token_by_user_email(
    pool: &MySqlPool,
    token: String,
    email: String,
) -> Result<JwtToken, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    info!("Validating refresh token: {}, for email: {}", token, email);

    let result = query!(
        "SELECT * FROM user
        WHERE user_refresh_token = ? AND user_email = ? AND user_refresh_token_expires_at > NOW();",
        token,
        email
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(JwtToken {
            access_token: res.user_access_token,
            access_token_expires_at: res.user_access_token_expires_at,
            refresh_token: res.user_refresh_token,
            refresh_token_expires_at: res.user_refresh_token_expires_at,
        }),
        Err(_) => Err(sqlx::Error::RowNotFound),
    }
}

pub async fn update_user_access_token(
    pool: &MySqlPool,
    user_email: String,
    access_token: String,
    access_token_expires_at: NaiveDateTime,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "UPDATE user
            SET user_access_token = ?, user_access_token_expires_at = ?
        WHERE user_email = ?",
        access_token,
        access_token_expires_at,
        user_email
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.rows_affected() as i32),
        Err(e) => Err(e),
    }
}

pub async fn update_uesr_tokens(
    pool: &MySqlPool,
    user_email: String,
    tokens: JwtToken,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "UPDATE user
            SET user_access_token = ?, user_refresh_token = ?,
                user_access_token_expires_at = ?, user_refresh_token_expires_at = ?
        WHERE user_email = ?",
        tokens.access_token.unwrap(),
        tokens.refresh_token.unwrap(),
        tokens.access_token_expires_at.unwrap(),
        tokens.refresh_token_expires_at.unwrap(),
        user_email
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.rows_affected() as i32),
        Err(e) => Err(e),
    }
}

pub async fn update_user_notification_setting(
    pool: &MySqlPool,
    user_email: String,
    notification_push: bool,
    user_fcm_token: String,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "UPDATE user
            SET user_notification_push = ?, user_fcm_token = ?
        WHERE user_email = ?",
        notification_push,
        user_fcm_token,
        user_email
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.rows_affected() as i32),
        Err(e) => Err(e),
    }
}

pub async fn get_user_theme(pool: &MySqlPool, user_email: String) -> Result<String, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "SELECT user_theme FROM user WHERE user_email = ?",
        user_email
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.user_theme.unwrap_or_else(|| "default".to_string())),
        Err(e) => Err(e),
    }
}

pub async fn update_user_theme(
    pool: &MySqlPool,
    user_email: String,
    theme: String,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    info!("theme: {}, user_email: {}", theme, user_email);

    let result = query!(
        "UPDATE user
            SET user_theme = ?
        WHERE user_email = ?",
        theme,
        user_email
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.rows_affected() as i32),
        Err(e) => Err(e),
    }
}

pub async fn update_user_info(
    pool: &MySqlPool,
    user_email: Option<String>,
    user_display_name: Option<String>,
    user_photo_url: Option<String>,
    user_social_login_provider: Option<String>,
    user_social_provider_id: Option<String>,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "UPDATE user
            SET user_display_name = ?, user_photo_url = ?,
                user_social_login_provider = ?, user_social_provider_id = ?
        WHERE user_email = ?",
        user_display_name,
        user_photo_url,
        user_social_login_provider,
        user_social_provider_id,
        user_email
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.rows_affected() as i32),
        Err(e) => Err(e),
    }
}
