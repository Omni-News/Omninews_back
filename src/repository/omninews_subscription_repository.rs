use chrono::NaiveDateTime;
use sqlx::{query, MySqlPool};

use crate::{db_util::get_db, model::omninews_subscription::NewOmniNewsSubscription};

pub async fn select_subscription_transaction_id(
    pool: &sqlx::MySqlPool,
    user_id: i32,
) -> Result<String, sqlx::Error> {
    let result = query!(
        "SELECT omninews_subscription_transaction_id FROM omninews_subscription WHERE user_id = ?",
        user_id,
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(res) => Ok(res.omninews_subscription_transaction_id.unwrap_or_default()),
        Err(e) => Err(e),
    }
}

pub async fn register_subscription(
    pool: &MySqlPool,
    subscription: NewOmniNewsSubscription,
) -> Result<bool, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "INSERT INTO 
            omninews_subscription (user_id, omninews_subscription_transaction_id, omninews_subscription_status, omninews_subscription_product_id, omninews_subscription_auto_renew, omninews_subscription_platform, omninews_subscription_start_date, omninews_subscription_renew_date, omninews_subscription_end_date, omninews_subscription_is_sandbox)
        VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        )",
        subscription.user_id,
        subscription.omninews_subscription_transaction_id,
        subscription.omninews_subscription_status,
        subscription.omninews_subscription_product_id,
        subscription.omninews_subscription_auto_renew,
        subscription.omninews_subscription_platform,
        subscription.omninews_subscription_start_date,
        subscription.omninews_subscription_renew_date,
        subscription.omninews_subscription_end_date,
        subscription.omninews_subscription_is_sandbox
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                Ok(true)
            } else {
                Err(sqlx::Error::RowNotFound)
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn expired_subscription(pool: &MySqlPool, user_id: i32) -> Result<bool, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "UPDATE omninews_subscription
     SET omninews_subscription_status = false, omninews_subscription_auto_renew = false
     WHERE user_id = ?",
        user_id
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

pub async fn update_verify_subscription_info(
    pool: &MySqlPool,
    user_id: i32,
    auto_renew: bool,
    renew_date: NaiveDateTime,
    expire_date: NaiveDateTime,
) -> Result<bool, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "UPDATE omninews_subscription
     SET omninews_subscription_auto_renew = ?, omninews_subscription_renew_date = ?, omninews_subscription_end_date = ?
     WHERE user_id = ?",
        auto_renew,
        renew_date,
        expire_date,
        user_id
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

pub async fn select_subscription_status_by_user_email(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<bool, sqlx::Error> {
    let result = query!(
        "SELECT omninews_subscription_status FROM omninews_subscription WHERE user_id = ?",
        user_id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(res) => Ok(res.omninews_subscription_status.unwrap_or_default() == 1),
        Err(e) => Err(e),
    }
}
