use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum SiteType {
    Instagram,
    Medium,
    Naver,
    Tistory,
    Default,
}
