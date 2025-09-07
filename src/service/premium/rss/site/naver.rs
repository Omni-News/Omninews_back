use sqlx::MySqlPool;

use crate::{
    model::error::OmniNewsError, service::channel_service, utils::embedding_util::EmbeddingService,
};
// channel : https://blog.naver.com/editor_style
// rss : https://blog.rss.naver.com/editor_style
pub async fn generate_rss(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    link: &str,
) -> Result<i32, OmniNewsError> {
    let user = extract_user_by_link(link)?;
    let naver_rss_link = format!("https://blog.rss.naver.com/{user}");

    match channel_service::create_rss_and_embedding(pool, embedding_service, naver_rss_link).await {
        Ok(channel_id) => Ok(channel_id),
        Err(e) => {
            error!(
                "[Service] Failed to create Naver Rss channel through rss generator. {}",
                e
            );
            Err(e)
        }
    }
}

fn extract_user_by_link(link: &str) -> Result<String, OmniNewsError> {
    let strs = link.split('/').collect::<Vec<&str>>();
    let last = strs.last().map(|s| s.to_string()).unwrap_or_default();

    if !last.is_empty() {
        return Ok(last);
    }

    error!("[Service] Unable to extract Naver user in Rss link.");
    Err(OmniNewsError::ExtractLinkError)
}
