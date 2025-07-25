use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserThemeResponseDto {
    #[schemars(example = "example_theme")]
    theme: Option<String>,
}

impl UserThemeResponseDto {
    pub fn new(theme: String) -> Self {
        UserThemeResponseDto { theme: Some(theme) }
    }
}

fn example_theme() -> &'static str {
    "paper"
}
