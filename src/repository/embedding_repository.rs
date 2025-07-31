use sqlx::{query, MySqlPool};

use crate::{db_util::get_db, model::embedding::NewEmbedding};

pub async fn insert_embedding(
    pool: &MySqlPool,
    embedding: NewEmbedding,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
    "INSERT INTO embedding (embedding_value, channel_id, rss_id, news_id, embedding_source_rank) VALUES (?, ?, ?, ?, ?);",
        embedding.embedding_value,
        embedding.channel_id,
        embedding.rss_id,
        embedding.news_id,
        embedding.embedding_source_rank,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => Err(e),
    }
}
