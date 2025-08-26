use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RssFolder {
    pub folder_id: Option<i32>,
    pub folder_name: Option<String>,
    pub user_id: Option<i32>,
}
