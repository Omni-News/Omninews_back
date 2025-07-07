use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoginUserRequestDto {
    pub user_email: Option<String>,
    pub user_display_name: Option<String>,
    pub user_photo_url: Option<String>,
    pub user_social_login_provider: Option<String>,
    pub user_social_provider_id: Option<String>,
    pub user_notification_push: Option<bool>,
}
