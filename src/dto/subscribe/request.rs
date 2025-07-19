use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
pub struct SubscribeRequestDto {
    #[schemars(example = "example_channel_id")]
    pub channel_id: Option<i32>,
}

fn example_channel_id() -> i32 {
    1
}
