use sqlx::MySqlPool;

use crate::{
    embedding_error,
    model::{embedding::NewEmbedding, error::OmniNewsError},
    repository::embedding_repository,
    utils::embedding_util::{embedding_sentence, encode_embedding, EmbeddingService},
};

pub async fn create_embedding(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    sentence: String,
    mut embedding: NewEmbedding,
) -> Result<i32, OmniNewsError> {
    let embedding_value = embedding_sentence(embedding_service, sentence).await?;
    let encoded_embedding_value = encode_embedding(&embedding_value);

    embedding.embedding_value = Some(encoded_embedding_value);

    match embedding_repository::insert_embedding(pool, embedding).await {
        Ok(res) => Ok(res),
        Err(e) => {
            embedding_error!("[Service] Failed to insert embedding: {}", e);
            Err(OmniNewsError::EmbeddingError)
        }
    }
}
