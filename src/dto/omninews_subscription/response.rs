use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::user::User;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OmninewsSubscriptionResponseDto {
    pub is_active: bool,
    pub product_id: String,
    pub expires_date: NaiveDateTime,
}

impl OmninewsSubscriptionResponseDto {
    pub fn from_model(user: User) -> Self {
        Self {
            is_active: user.user_subscription_plan.is_some()
                && user.user_subscription_plan.unwrap() > 0,
            product_id: user.user_subscription_product_id.unwrap_or_default(),
            expires_date: user.user_subscription_end_date.unwrap_or_default(),
        }
    }
}
