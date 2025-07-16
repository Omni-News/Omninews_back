use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    #[schemars(example = "example_user_notification_push")]
    pub user_notification_push: Option<bool>,
}

fn example_user_email() -> &'static str {
    "hong11@gil.com     // user의 email 주소"
}

fn example_user_display_name() -> &'static str {
    "홍길동    // 화면에 표시될 user의 이름"
}

fn example_user_photo_url() -> &'static str {
    "https://example.com/photo.jpg    // user의 프로필 사진 url, Oauth로그인 시 각 플랫폼에서 제공됨."
}

fn example_user_social_login_provider() -> &'static str {
    "google    // user의 소셜 로그인 제공자, ex) google, apple, kakao"
}

fn example_user_social_provider_id() -> &'static str {
    "1234567890    // 소셜 로그인 제공자에서 발급한 user의 고유 ID, Oauth 로그인 시 플랫폼에서 제공됨."
}

fn example_user_notification_push() -> bool {
    true // user의 푸시 알림 수신 여부, 기본값은 true
}
