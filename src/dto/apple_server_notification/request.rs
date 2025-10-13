use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AppleServerNotificationRequestDto {
    pub signed_payload: Option<String>,
}
