use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
pub struct ApiNewsRequestDto {
    pub query: Option<String>,
    pub display: Option<i32>,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
pub struct NewsRequestDto {
    pub category: Option<String>,
    pub page: Option<i32>,
}
