use sqlx::MySqlPool;

use crate::{model::error::OmniNewsError, repository::token_repository};

pub async fn validate_token_user(
    pool: &MySqlPool,
    token: String,
    email: &str,
) -> Result<bool, OmniNewsError> {
    match token_repository::validate_token_user(pool, token).await {
        Ok(res) => {
            if res.eq(email) {
                return Ok(true);
            }
            Ok(false)
        }
        Err(_) => {
            error!("Token validation failed");
            Err(OmniNewsError::TokenValidationError)
        }
    }
}
