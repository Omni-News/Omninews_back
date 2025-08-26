use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
pub struct NewsRequestDto {
    pub query: Option<String>,
    pub display: Option<i32>,
    pub sort: Option<String>,
}
