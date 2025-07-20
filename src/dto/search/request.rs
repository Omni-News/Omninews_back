use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::search::SearchType;

#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
pub struct SearchRequestDto {
    pub search_value: Option<String>,
    pub search_type: Option<SearchType>,
    pub search_page_size: Option<i32>,
}
