use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
pub struct OmninewsReceiptRequestDto {
    pub transaction_id: Option<String>,
    pub platform: Option<String>,
}
