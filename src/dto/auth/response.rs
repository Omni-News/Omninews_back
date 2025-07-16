use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::auth::{AccessToken, JwtToken};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JwtTokenResponseDto {
    #[schemars(example = "example_access_token")]
    pub access_token: Option<String>,
    #[schemars(example = "example_refresh_token")]
    pub refresh_token: Option<String>,
    #[schemars(example = "example_access_token_expires")]
    pub access_token_expires_at: Option<NaiveDateTime>,
    #[schemars(example = "example_refresh_token_expires")]
    pub refresh_token_expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccessTokenResponseDto {
    #[schemars(example = "example_access_token")]
    pub access_token: Option<String>,
    #[schemars(example = "example_access_token_expires")]
    pub access_token_expires_at: Option<NaiveDateTime>,
}

impl JwtTokenResponseDto {
    pub fn from_model(jwt_token: JwtToken) -> Self {
        JwtTokenResponseDto {
            access_token: jwt_token.access_token,
            refresh_token: jwt_token.refresh_token,
            access_token_expires_at: jwt_token.access_token_expires_at,
            refresh_token_expires_at: jwt_token.refresh_token_expires_at,
        }
    }
}

impl AccessTokenResponseDto {
    pub fn from_model(access_token: AccessToken) -> Self {
        AccessTokenResponseDto {
            access_token: access_token.access_token,
            access_token_expires_at: access_token.access_token_expires_at,
        }
    }
}

fn example_access_token() -> &'static str {
    "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUz...    // 서버에서 생성한 access token"
}

fn example_access_token_expires() -> &'static str {
    "2016-07-08 09:10:11    // 서버에서 생성한 access token의 만료 시간"
}

fn example_refresh_token() -> &'static str {
    "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUz...   // 서버에서 생성한 refresh token"
}

fn example_refresh_token_expires() -> &'static str {
    "2016-07-09 12:30:00    // 서버에서 생성한 refresh token의 만료 시간"
}
