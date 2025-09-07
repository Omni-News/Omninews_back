use std::env;

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::Claims,
    dto::{
        auth::{
            request::VerifyRefreshTokenRequestDto,
            response::{AccessTokenResponseDto, JwtTokenResponseDto},
        },
        user::{
            request::{
                AppleLoginRequestDto, LoginUserRequestDto, UserNotificationRequestDto,
                UserThemeRequestDto,
            },
            response::UserThemeResponseDto,
        },
    },
    model::{
        auth::{AccessToken, JwtToken, TokenType},
        error::OmniNewsError,
        user::NewUser,
    },
    repository::user_repository,
    user_error, user_info, user_warn,
};

/// 1. access token O refresh token O -> processed in auto login.
/// 2. access token X refresh token O -> processed in auto login.
/// 3. access token X refresh token X -> Reissue refresh token and access token or Create user +
///    Update User Info
pub async fn login_or_create_user(
    pool: &MySqlPool,
    user: LoginUserRequestDto,
) -> Result<JwtTokenResponseDto, OmniNewsError> {
    let (is_access_available, is_refresh_available) = user_repository::validate_users_all_tokens(
        pool,
        user.user_email.clone().unwrap_or_default(),
    )
    .await
    .map_err(|e| {
        user_error!(
            "[Service] Failed to validate access and refresh token: {}",
            e
        );
        OmniNewsError::TokenValidationError
    })?;

    // 1 or 2 -> new login
    if is_access_available || is_refresh_available {
        user_warn!("[Service] Access or refresh token is already alived ailed to login");
    }

    // 3. access token X refresh token X -> reissue both token or create user
    if (user_repository::select_user_id_by_email(pool, user.user_email.clone().unwrap()).await)
        .is_ok()
    {
        // 3-1. reissue both
        user_info!(
            "[Service] 3-1. Success login: {}",
            user.user_email.clone().unwrap()
        );
        let jwt_token = issue_tokens(pool, user.user_email.clone().unwrap()).await?;
        // update user info
        match user_repository::update_user_info(
            pool,
            user.user_email.clone(),
            user.user_display_name,
            user.user_photo_url,
            user.user_social_login_provider,
            user.user_social_provider_id,
        )
        .await
        {
            Ok(_) => {
                user_info!(
                    "[Service] User info updated for email: {}",
                    user.user_email.clone().unwrap()
                );
            }
            Err(e) => {
                user_error!("[Service] Failed to update user info: {}", e);
                return Err(OmniNewsError::Database(e));
            }
        }
        user_info!(
            "[Service] User info updated for email: {}",
            user.user_email.clone().unwrap()
        );

        return Ok(JwtTokenResponseDto::from_model(jwt_token));
    }
    // 3-2.create user
    user_info!(
        "[Service] 3-2. Success login: {}",
        user.user_email.clone().unwrap()
    );

    let jwt_token = create_user(pool, user).await?;
    Ok(JwtTokenResponseDto::from_model(jwt_token))
}

async fn create_user(
    pool: &MySqlPool,
    user: LoginUserRequestDto,
) -> Result<JwtToken, OmniNewsError> {
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

    let new_user = NewUser::new(
        user,
        JwtToken::new(
            access_token.clone(),
            refresh_token.clone(),
            access_token_expires_at,
            refresh_token_expires_at,
        ),
        ktc_now,
    );
    match user_repository::insert_user(pool, new_user).await {
        Ok(_) => Ok(JwtToken::new(
            access_token,
            refresh_token,
            access_token_expires_at,
            refresh_token_expires_at,
        )),

        Err(e) => {
            user_error!("[Service] Failed to create user: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn vliadate_access_token(
    pool: &MySqlPool,
    token: String,
    email: String,
) -> Result<bool, OmniNewsError> {
    match user_repository::validate_access_token_by_user_email(pool, token, email).await {
        Ok(_) => Ok(true),
        Err(_) => {
            user_error!("[Service] Token validation failed");
            Err(OmniNewsError::TokenValidationError)
        }
    }
}

pub async fn validate_refresh_token(
    pool: &MySqlPool,
    refresh_token: VerifyRefreshTokenRequestDto,
) -> Result<AccessTokenResponseDto, OmniNewsError> {
    let user_email = refresh_token.email.clone().unwrap_or_default();
    let refresh_token = refresh_token.token.unwrap_or_default();

    match user_repository::validate_refresh_token_by_user_email(
        pool,
        refresh_token,
        user_email.clone(),
    )
    .await
    {
        Ok(_) => {
            user_info!("[Service] Reissue access token.");
            let acces_token = reissue_access_token(pool, user_email).await?;

            Ok(AccessTokenResponseDto::from_model(acces_token))
        }
        Err(_) => {
            user_error!("[Service] Refresh token validation failed");
            Err(OmniNewsError::TokenValidationError)
        }
    }
}

async fn reissue_access_token(
    pool: &MySqlPool,
    user_email: String,
) -> Result<AccessToken, OmniNewsError> {
    let (access_token, access_token_expires_at) =
        make_token(TokenType::Access, user_email.clone())?;

    let _ = user_repository::update_user_access_token(
        pool,
        user_email,
        access_token.clone(),
        access_token_expires_at,
    )
    .await?;

    Ok(AccessToken::new(access_token, access_token_expires_at))
}

async fn issue_tokens(pool: &MySqlPool, user_email: String) -> Result<JwtToken, OmniNewsError> {
    let (access_token, access_token_expires_at) =
        make_token(TokenType::Access, user_email.clone())?;

    let (refresh_token, refresh_token_expires_at) =
        make_token(TokenType::Refresh, user_email.clone())?;

    let tokens = JwtToken::new(
        access_token,
        refresh_token,
        access_token_expires_at,
        refresh_token_expires_at,
    );
    match user_repository::update_uesr_tokens(pool, user_email, tokens.clone()).await {
        Ok(_) => Ok(tokens),
        Err(e) => {
            user_error!("[Service] Failed to update user tokens: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

fn make_token(
    token_type: TokenType,
    sub: String,
) -> Result<(String, NaiveDateTime), OmniNewsError> {
    let key_string = env::var("JWT_SECRET_KEY").unwrap();
    let key = key_string.as_bytes();

    match token_type {
        TokenType::Access => {
            let access_token_exp = Utc::now().timestamp() as u64 + (60 * 60 * 24) + (9 * 3600); // 1 day and kst
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
                    user_error!("[Service] Failed to create JWT access token: {}", e);
                    return Err(OmniNewsError::TokenCreateError);
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
                Utc::now().timestamp() as u64 + (60 * 60 * 24 * 31) + (9 * 3600); // 1 month and
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
                    user_error!("[Service] Failed to create JWT refresh token: {}", e);
                    return Err(OmniNewsError::TokenCreateError);
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

pub async fn apple_login(
    pool: &MySqlPool,
    apple_login: AppleLoginRequestDto,
) -> Result<JwtTokenResponseDto, OmniNewsError> {
    let user_email = user_repository::select_user_email_by_social_provider_id(
        pool,
        apple_login
            .user_social_provider_id
            .clone()
            .unwrap_or_default(),
    )
    .await?;

    // Apple login is not a new user, so we can issue tokens directly
    login_or_create_user(
        pool,
        LoginUserRequestDto {
            user_email: Some(user_email),
            user_display_name: None,
            user_photo_url: None,
            user_social_login_provider: Some("apple".to_string()),
            user_social_provider_id: apple_login.user_social_provider_id,
        },
    )
    .await
}

pub async fn find_user_id_by_email(
    pool: &MySqlPool,
    user_email: String,
) -> Result<i32, OmniNewsError> {
    match user_repository::select_user_id_by_email(pool, user_email).await {
        Ok(user_id) => Ok(user_id),
        Err(e) => {
            user_error!("[Service] Failed to find user id by email: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn validate_premium_user(
    pool: &MySqlPool,
    user_email: &str,
) -> Result<bool, OmniNewsError> {
    match user_repository::validate_premium_user(pool, &user_email).await {
        Ok(res) => Ok(res),
        Err(e) => {
            user_error!("[Service] Failed to find user id by email: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn delete_user_token(pool: &MySqlPool, user_email: String) -> Result<(), OmniNewsError> {
    match user_repository::delete_user_token_by_email(pool, user_email).await {
        Ok(_) => Ok(()),
        Err(e) => {
            user_error!("[Service] Failed to remove user token: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
pub async fn update_user_notification_setting(
    pool: &MySqlPool,
    user_email: String,
    notification_data: UserNotificationRequestDto,
) -> Result<(), OmniNewsError> {
    match user_repository::update_user_notification_setting(
        pool,
        user_email,
        notification_data.user_notification_push.unwrap_or_default(),
        notification_data.user_fcm_token.unwrap_or_default(),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            user_error!(
                "[Service] Failed to update user notification setting: {}",
                e
            );
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn get_user_theme(
    pool: &MySqlPool,
    user_email: String,
) -> Result<UserThemeResponseDto, OmniNewsError> {
    match user_repository::get_user_theme(pool, user_email).await {
        Ok(theme) => Ok(UserThemeResponseDto::new(theme)),
        Err(e) => {
            user_error!("[Service] Failed to get user theme: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn update_user_theme(
    pool: &MySqlPool,
    user_email: String,
    theme: UserThemeRequestDto,
) -> Result<(), OmniNewsError> {
    match user_repository::update_user_theme(pool, user_email, theme.theme.unwrap_or_default())
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            user_error!("[Service] Failed to update user theme: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
