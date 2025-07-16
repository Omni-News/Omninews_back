use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifyRefreshTokenRequestDto {
    #[schemars(example = "example_token")]
    pub token: Option<String>,
    #[schemars(example = "example_email")]
    pub email: Option<String>,
}

fn example_token() -> &'static str {
    "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUz...    // 클라이언트가 가지고 있는 refresh token"
}

fn example_email() -> &'static str {
    "hong11@gil.com    // 클라이언트의 email 주소"
}
