use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NewFeedback {
    pub feedback_email: Option<String>,
    pub feedback_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Feedback {
    pub feedback_id: Option<i32>,
    pub feedback_email: Option<String>,
    pub feedback_content: Option<String>,
}
