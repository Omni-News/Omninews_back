use chrono::NaiveDateTime;

pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone)]
pub struct JwtToken {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub access_token_expires_at: Option<NaiveDateTime>,
    pub refresh_token_expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct AccessToken {
    pub access_token: Option<String>,
    pub access_token_expires_at: Option<NaiveDateTime>,
}

impl JwtToken {
    pub fn new(
        access_token: String,
        refresh_token: String,
        access_token_expires_at: NaiveDateTime,
        refresh_token_expires_at: NaiveDateTime,
    ) -> Self {
        Self {
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            access_token_expires_at: Some(access_token_expires_at),
            refresh_token_expires_at: Some(refresh_token_expires_at),
        }
    }
}

impl AccessToken {
    pub fn new(access_token: String, access_token_expires_at: NaiveDateTime) -> Self {
        Self {
            access_token: Some(access_token),
            access_token_expires_at: Some(access_token_expires_at),
        }
    }
}
