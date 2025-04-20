use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEmail {
    pub user_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ParamUser {
    pub user_email: Option<String>,
    pub user_display_name: Option<String>,
    pub user_photo_url: Option<String>,
    pub user_social_login_provider: Option<String>,
    pub user_social_provider_id: Option<String>,
    pub user_notification_push: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NewUser {
    pub user_email: Option<String>,
    pub user_display_name: Option<String>,
    pub user_photo_url: Option<String>,
    pub user_social_login_provider: Option<String>,
    pub user_social_provider_id: Option<String>,
    pub user_access_token: Option<String>,
    pub user_refresh_token: Option<String>,
    pub user_access_token_expires_at: Option<NaiveDateTime>,
    pub user_refresh_token_expires_at: Option<NaiveDateTime>,
    pub user_notification_push: Option<bool>,
    pub user_last_active_at: Option<NaiveDateTime>,
    pub user_created_at: Option<NaiveDateTime>,
    pub user_updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: Option<i32>,
    pub user_email: Option<String>,
    pub user_display_name: Option<String>,
    pub user_photo_url: Option<String>,
    pub user_social_login_provider: Option<String>,
    pub user_social_provider_id: Option<String>,
    pub user_access_token: Option<String>,
    pub user_refresh_token: Option<String>,
    pub user_access_token_expires_at: Option<NaiveDateTime>,
    pub user_refresh_token_expires_at: Option<NaiveDateTime>,
    pub user_status: Option<String>,
    pub user_role: Option<String>,
    pub user_theme: Option<String>,
    pub user_notification_push: Option<bool>,
    pub user_articles_read: Option<i32>,
    pub user_last_active_at: Option<String>,
    pub user_subscription_plan: Option<String>,
    pub user_subscription_start_date: Option<String>,
    pub user_subscription_end_date: Option<String>,
    pub user_subscription_last_date: Option<String>,
    pub user_subscription_auto_renew: Option<bool>,
}
