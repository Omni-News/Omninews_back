use serde::{Deserialize, Serialize};

use super::rss::{RssChannel, RssItem};

#[derive(Debug, Clone, Deserialize)]
pub enum SearchType {
    Accuracy,
    Popularity,
    Latest,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    pub search_value: Option<String>,
    pub search_type: Option<SearchType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewticleList {
    pub channel_list: Option<Vec<RssChannel>>,
    pub rss_list: Option<Vec<RssItem>>,
}
