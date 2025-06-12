use serde::{Deserialize, Serialize};

use super::rss::RssChannel;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewRssFolder {
    pub folder_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RssFolder {
    pub folder_id: Option<i32>,
    pub folder_name: Option<String>,
    pub user_id: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelFolderId {
    pub folder_id: Option<i32>,
    pub channel_id: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FolderId {
    pub folder_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateFolder {
    pub folder_id: Option<i32>,
    pub folder_name: Option<String>,
}

// TODO reponse와 model 접근 객체 구분 필요할 듯. news, rss 등 dto로 분리
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RssFolderResponse {
    pub folder_id: Option<i32>,
    pub folder_name: Option<String>,
    pub folder_channels: Option<Vec<RssChannel>>,
}
