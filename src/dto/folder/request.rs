use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateFolderRequestDto {
    pub folder_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChannelFolderRequestDto {
    pub folder_id: Option<i32>,
    pub channel_id: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateFolderRequestDto {
    pub folder_id: Option<i32>,
    pub folder_name: Option<String>,
}
