use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
pub struct OmninewsReceiptRequestDto {
    pub receipt_data: Option<String>,
    pub platform: Option<String>,
    pub is_test: Option<bool>,
}
