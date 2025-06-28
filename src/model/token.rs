use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub enum TokenType {
    Access,
    Refresh,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtToken {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub access_token_expires_at: Option<NaiveDateTime>,
    pub refresh_token_expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub access_token: Option<String>,
    pub access_token_expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenAndUserEmail {
    pub token: Option<String>,
    pub email: Option<String>,
}
