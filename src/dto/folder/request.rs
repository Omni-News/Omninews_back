use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateFolderRequestDto {
    #[schemars(example = "example_folder_name")]
    pub folder_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChannelFolderRequestDto {
    #[schemars(example = "example_folder_id")]
    pub folder_id: Option<i32>,
    #[schemars(example = "example_channel_id")]
    pub channel_id: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateFolderRequestDto {
    #[schemars(example = "example_folder_id")]
    pub folder_id: Option<i32>,
    #[schemars(example = "example_folder_name")]
    pub folder_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteFolderRequestDto {
    #[schemars(example = "example_folder_id")]
    pub folder_id: Option<i32>,
}

fn example_folder_name() -> &'static str {
    "Development"
}

fn example_channel_id() -> i32 {
    3
}
fn example_folder_id() -> i32 {
    2
}
