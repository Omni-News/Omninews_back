use sqlx::MySqlPool;

use crate::{
    model::error::OmniNewsError, service::channel_service, utils::embedding_util::EmbeddingService,
};

// channel : https://kang1027.tistory.com/
// rss : https://kang1027.tistory.com/rss
pub async fn generate_rss(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    link: &str,
) -> Result<i32, OmniNewsError> {
    let user = extract_user_by_link(link)?;
    let tistory_rss_link = format!("https://{user}.tistory.com/rss");

    match channel_service::create_rss_and_embedding(pool, embedding_service, tistory_rss_link).await
    {
        Ok(channel_id) => Ok(channel_id),
        Err(e) => {
            error!(
                "[Service] Failed to create Tistory Rss channel through rss generator. {}",
                e
            );
            Err(e)
        }
    }
}

fn extract_user_by_link(link: &str) -> Result<String, OmniNewsError> {
    let strs = link.split('/').collect::<Vec<&str>>();

    if strs.len() == 1 {
        return Ok(strs.first().map(|s| s.to_string()).unwrap_or_default());
    }

    let last = strs.get(2).map(|s| s.to_string()).unwrap_or_default();

    if !last.is_empty() {
        return Ok(last
            .split(".")
            .collect::<Vec<&str>>()
            .first()
            .map(|s| s.to_string())
            .unwrap_or_default());
    }

    error!("[Service] Unable to extract Tistory user in Rss link.");
    Err(OmniNewsError::ExtractLinkError)
}
