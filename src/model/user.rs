use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::dto::user::request::LoginUserRequestDto;

use super::auth::JwtToken;

#[derive(Debug, Clone)]
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
    pub user_notification_push: Option<i8>,
    pub user_fcm_token: Option<String>,
    pub user_articles_read: Option<i32>,
    pub user_last_active_at: Option<NaiveDateTime>,
    pub user_subscription_product_id: Option<String>,
    pub user_subscription_transaction_id: Option<String>,
    pub user_subscription_receipt_data: Option<String>,
    pub user_subscription_platform: Option<String>,
    pub user_subscription_is_test: Option<i8>,
    pub user_subscription_start_date: Option<NaiveDateTime>,
    pub user_subscription_end_date: Option<NaiveDateTime>,
    pub user_subscription_auto_renew: Option<i8>,
    pub user_created_at: Option<NaiveDateTime>,
    pub user_updated_at: Option<NaiveDateTime>,
}

impl NewUser {
    pub fn new(user_dto: LoginUserRequestDto, token: JwtToken, now: NaiveDateTime) -> Self {
        Self {
            user_email: user_dto.user_email,
            user_display_name: user_dto.user_display_name,
            user_photo_url: user_dto.user_photo_url,
            user_social_login_provider: user_dto.user_social_login_provider,
            user_social_provider_id: user_dto.user_social_provider_id,
            user_access_token: token.access_token,
            user_refresh_token: token.refresh_token,
            user_access_token_expires_at: token.access_token_expires_at,
            user_refresh_token_expires_at: token.refresh_token_expires_at,
            user_last_active_at: Some(now),
            user_created_at: Some(now),
            user_updated_at: Some(now),
        }
    }
}
