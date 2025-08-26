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
    "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUz..."
}

fn example_email() -> &'static str {
    "hong11@gil.com"
}
