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

    pub fn theme(&self) -> &Option<String> {
        &self.theme
    }
}

fn example_theme() -> &'static str {
    "paper"
}

