use std::{collections::HashSet, path::PathBuf};

use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{embedding::Embedding, error::OmniNewsError, rss::NewticleType},
    service::embedding_service,
    utils::embedding_util::decode_embedding,
};

use super::embedding_util::{embedding_sentence, EmbeddingService};

pub async fn save_annoy(pool: &MySqlPool) -> Result<(), OmniNewsError> {
    let embeddings_channel =
        embedding_service::find_all_embeddings_by(pool, NewticleType::Channel).await?;
    let embeddings_rss = embedding_service::find_all_embeddings_by(pool, NewticleType::Rss).await?;
    let embeddings_news =
        embedding_service::find_all_embeddings_by(pool, NewticleType::News).await?;
    save_channel_annoy(embeddings_channel).await?;
    save_rss_annoy(embeddings_rss).await?;
    save_news_annoy(embeddings_news).await?;

    Ok(())
}

async fn save_channel_annoy(embeddings: Vec<Embedding>) -> Result<(), OmniNewsError> {
    if embeddings.is_empty() {
        info!("[Service] No embeddings found for channel.");
        return Ok(());
    }
    let embedding_dim = embeddings[0].embedding_value.as_ref().unwrap().len();
    info!("[Service] Embedding dimension: {}", embedding_dim);

    let annoy = rannoy::Rannoy::new(384);
    annoy.set_seed(123);

    for embedding in embeddings.iter() {
        let decoded_embedding = decode_embedding(embedding.embedding_value.as_ref().unwrap());
        annoy.add_item(embedding.embedding_id.unwrap(), decoded_embedding.as_ref());
    }
    annoy.build(40);
    annoy.save(PathBuf::from("channel_embeddings.ann"));

    Ok(())
}

async fn save_rss_annoy(embeddings: Vec<Embedding>) -> Result<(), OmniNewsError> {
    if embeddings.is_empty() {
        info!("[Service] No embeddings found for rss.");
        return Ok(());
    }
    let embedding_dim = embeddings[0].embedding_value.as_ref().unwrap().len();
    info!("[Service] Embedding dimension: {}", embedding_dim);

    let annoy = rannoy::Rannoy::new(384);
    annoy.set_seed(123);

    for embedding in embeddings.iter() {
        let decoded_embedding = decode_embedding(embedding.embedding_value.as_ref().unwrap());
        annoy.add_item(embedding.embedding_id.unwrap(), decoded_embedding.as_ref());
    }
    annoy.build(40);
    annoy.save(PathBuf::from("rss_embeddings.ann"));

    Ok(())
}

async fn save_news_annoy(embeddings: Vec<Embedding>) -> Result<(), OmniNewsError> {
    if embeddings.is_empty() {
        info!("[Service] No embeddings found for news.");
        return Ok(());
    }
    let embedding_dim = embeddings[0].embedding_value.as_ref().unwrap().len();
    info!("[Service] Embedding dimension: {}", embedding_dim);

    let annoy = rannoy::Rannoy::new(384);
    annoy.set_seed(123);

    for embedding in embeddings.iter() {
        let decoded_embedding = decode_embedding(embedding.embedding_value.as_ref().unwrap());
        annoy.add_item(embedding.embedding_id.unwrap(), decoded_embedding.as_ref());
    }
    annoy.build(40);
    annoy.save(PathBuf::from("news_embeddings.ann"));

    Ok(())
}

// TODO 지금은 10개 조회지만, 상황에 맞춰 더많이 추가 가능하도록 수정
pub async fn load_channel_annoy(
    service: &EmbeddingService,
    search_value: String,
) -> Result<(Vec<i32>, Vec<f32>), OmniNewsError> {
    let annoy = rannoy::Rannoy::new(384);
    annoy.load(PathBuf::from("channel_embeddings.ann"));

    let embedding_search_text = embedding_sentence(service, search_value).await?;

    let (result_ids, distances) = annoy.get_nns_by_vector(embedding_search_text, 200, -1);
    // remove duplicate ids
    let unique_ids = result_ids
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    Ok((unique_ids, distances))
}

pub async fn load_rss_annoy(
    service: &EmbeddingService,
    search_value: String,
) -> Result<(Vec<i32>, Vec<f32>), OmniNewsError> {
    let annoy = rannoy::Rannoy::new(384);
    annoy.load(PathBuf::from("rss_embeddings.ann"));

    let embedding_search_text = embedding_sentence(service, search_value).await?;

    let (result_ids, distances) = annoy.get_nns_by_vector(embedding_search_text, 200, -1);
    Ok((result_ids, distances))
}

#[allow(dead_code)]
pub async fn load_news_annoy(
    service: &State<EmbeddingService>,
    search_value: String,
) -> Result<(Vec<i32>, Vec<f32>), OmniNewsError> {
    let annoy = rannoy::Rannoy::new(384);
    annoy.load(PathBuf::from("news_embeddings.ann"));

    let embedding_search_text = embedding_sentence(service, search_value).await?;

    let (result_ids, distances) = annoy.get_nns_by_vector(embedding_search_text, 10, -1);
    Ok((result_ids, distances))
}
