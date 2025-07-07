use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromFormField, JsonSchema)]
pub enum SearchType {
    Accuracy,
    Popularity,
    Latest,
}
