use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifyRefreshTokenRequestDto {
    pub token: Option<String>,
    pub email: Option<String>,
}
