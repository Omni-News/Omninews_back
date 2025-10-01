use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OmninewsSubscriptionResponseDto {
    pub is_active: bool,
    pub product_id: String,
    pub expires_date: NaiveDateTime,
}
