use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::auth::{AccessToken, JwtToken};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JwtTokenResponseDto {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub access_token_expires_at: Option<NaiveDateTime>,
    pub refresh_token_expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccessTokenResponseDto {
    pub access_token: Option<String>,
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
