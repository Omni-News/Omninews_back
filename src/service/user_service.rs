use std::env;

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rocket::State;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::{
    model::{
        error::OmniNewsError,
        token::JwtToken,
        user::{NewUser, ParamUser},
    },
    repository::user_repository,
};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String,
    sub: String,
    company: String,
    exp: u64,
}

pub async fn create_user(
    pool: &State<MySqlPool>,
    user: ParamUser,
) -> Result<JwtToken, OmniNewsError> {
    // Validate user already exists
    if let Ok(res) =
        user_repository::select_user_by_email(pool, user.user_email.clone().unwrap_or_default())
            .await
    {
        error!("[Service] User already exists, user id: {}", res);
        return Err(OmniNewsError::AlreadyExists);
    };

    let (access_token, access_token_expires_at) = make_token(
        TokenType::Access,
        user.user_email.clone().unwrap_or_default(),
    )?;
    let (refresh_token, refresh_token_expires_at) = make_token(
        TokenType::Refresh,
        user.user_email.clone().unwrap_or_default(),
    )?;

    let ktc_now = Utc::now()
        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
        .naive_utc();

    let new_user = NewUser {
        user_email: user.user_email,
        user_display_name: user.user_display_name,
        user_photo_url: user.user_photo_url,
        user_social_login_provider: user.user_social_login_provider,
        user_social_provider_id: user.user_social_provider_id,
        user_access_token: Some(access_token.clone()),
        user_refresh_token: Some(refresh_token.clone()),
        user_access_token_expires_at: Some(access_token_expires_at),
        user_refresh_token_expires_at: Some(refresh_token_expires_at),
        user_notification_push: user.user_notification_push,
        user_last_active_at: Some(ktc_now),
        user_created_at: Some(ktc_now),
        user_updated_at: Some(ktc_now),
    };
    match user_repository::insert_user(pool, new_user).await {
        Ok(_) => Ok(JwtToken {
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            access_token_expires_at: Some(access_token_expires_at),
            refresh_token_expires_at: Some(refresh_token_expires_at),
        }),

        Err(e) => {
            error!("[Service] Failed to create user: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

enum TokenType {
    Access,
    Refresh,
}
fn make_token(
    token_type: TokenType,
    sub: String,
) -> Result<(String, NaiveDateTime), OmniNewsError> {
    let key_string = env::var("JWT_SECRET_KEY").unwrap();
    let key = key_string.as_bytes();

    match token_type {
        TokenType::Access => {
            let access_token_exp = Utc::now().timestamp() as u64 + (30 * 60) + (9 * 3600); // 30 min and kst
            let access_token_claim = Claims {
                aud: "omninews".to_owned(),
                sub: sub.clone(),
                company: "kdh".to_owned(),
                exp: access_token_exp,
            };

            let access_token = match encode(
                &Header::default(),
                &access_token_claim,
                &EncodingKey::from_secret(key),
            ) {
                Ok(token) => Some(token),
                Err(e) => {
                    error!("[Service] Failed to create JWT access token: {}", e);
                    return Err(OmniNewsError::TokenError);
                }
            };

            let access_token_expires_at = DateTime::from_timestamp(access_token_exp as i64, 0);

            Ok((
                access_token.unwrap_or_default(),
                access_token_expires_at.unwrap_or_default().naive_utc(),
            ))
        }
        TokenType::Refresh => {
            let refresh_token_exp =
                Utc::now().timestamp() as u64 + (60 * 60 * 24 * 14) + (9 * 3600); // 14 days and
                                                                                  // kst

            let refresh_token_claim = Claims {
                aud: "omninews".to_owned(),
                sub,
                company: "kdh".to_owned(),
                exp: refresh_token_exp,
            };

            let refresh_token = match encode(
                &Header::default(),
                &refresh_token_claim,
                &EncodingKey::from_secret(key),
            ) {
                Ok(token) => Some(token),
                Err(e) => {
                    error!("[Service] Failed to create JWT refresh token: {}", e);
                    return Err(OmniNewsError::TokenError);
                }
            };

            let refresh_token_expires_at = DateTime::from_timestamp(refresh_token_exp as i64, 0);

            Ok((
                refresh_token.unwrap_or_default(),
                refresh_token_expires_at.unwrap_or_default().naive_utc(),
            ))
        }
    }
}

pub async fn delete_user_token(
    pool: &State<MySqlPool>,
    user_email: String,
) -> Result<(), OmniNewsError> {
    match user_repository::delete_user_token_by_email(pool, user_email).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("[Service] Failed to remove user token: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
