use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DemoLoginRequestDto {
    #[schemars(example = "example_user_email")]
    pub user_email: Option<String>,
    #[schemars(example = "example_user_display_name")]
    pub user_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoginUserRequestDto {
    #[schemars(example = "example_user_email")]
    pub user_email: Option<String>,
    #[schemars(example = "example_user_display_name")]
    pub user_display_name: Option<String>,
    #[schemars(example = "example_user_photo_url")]
    pub user_photo_url: Option<String>,
    #[schemars(example = "example_user_social_login_provider")]
    pub user_social_login_provider: Option<String>,
    #[schemars(example = "example_user_social_provider_id")]
    pub user_social_provider_id: Option<String>,
    #[schemars(example = "example_user_platform")]
    pub user_platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AppleLoginRequestDto {
    #[schemars(example = "example_user_social_provider_id")]
    pub user_social_provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserNotificationRequestDto {
    #[schemars(example = "example_user_notification_push")]
    pub user_notification_push: Option<bool>,
    #[schemars(example = "example_user_fcm_token")]
    pub user_fcm_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserThemeRequestDto {
    #[schemars(example = "example_theme")]
    pub theme: Option<String>,
}

fn example_user_email() -> &'static str {
    "hong11@gil.com"
}

fn example_user_display_name() -> &'static str {
    "홍길동"
}

fn example_user_photo_url() -> &'static str {
    "https://example.com/photo.jpg"
}

fn example_user_social_login_provider() -> &'static str {
    "google"
}

fn example_user_social_provider_id() -> &'static str {
    "1234567890"
}

fn example_user_notification_push() -> bool {
    true
}

fn example_user_fcm_token() -> &'static str {
    "fcm_token_example_1234567890"
}

fn example_theme() -> &'static str {
    "paper"
}

fn example_user_platform() -> &'static str {
    "ios"
}
