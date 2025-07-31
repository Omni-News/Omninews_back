use std::{collections::HashSet, path::PathBuf};

use rocket::State;

use crate::model::error::OmniNewsError;

use super::embedding_util::{embedding_sentence, EmbeddingService};

// 거리 임계값 상수 추가
const DISTANCE_THRESHOLD: f32 = 0.6;

// TODO 지금은 10개 조회지만, 상황에 맞춰 더많이 추가 가능하도록 수정
pub async fn load_channel_annoy(
    service: &EmbeddingService,
    search_value: String,
) -> Result<(Vec<i32>, Vec<f32>), OmniNewsError> {
    let annoy = rannoy::Rannoy::new(384);
    annoy.load(PathBuf::from("channel_embeddings.ann"));

    // 검색어 형식화
    let search_query = format!("제목: {}. 내용: {}", search_value, search_value);
    let embedding_search_text = embedding_sentence(service, search_query).await?;

    // search_k 값 추가 (10000)
    let (result_ids, distances) = annoy.get_nns_by_vector(embedding_search_text, 200, 10000);

    // 거리 기반 필터링 적용
    let filtered_results: Vec<(i32, f32)> = result_ids
        .into_iter()
        .zip(distances.into_iter())
        .filter(|&(_, distance)| distance < DISTANCE_THRESHOLD)
        .collect();

    // 필터링된 결과 사용
    let filtered_ids: Vec<i32> = filtered_results.iter().map(|(id, _)| *id).collect();
    let filtered_distances: Vec<f32> = filtered_results.iter().map(|(_, dist)| *dist).collect();

    // 중복 제거
    let unique_ids = filtered_ids
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    Ok((unique_ids, filtered_distances))
}

pub async fn load_rss_annoy(
    service: &EmbeddingService,
    search_value: String,
) -> Result<(Vec<i32>, Vec<f32>), OmniNewsError> {
    let annoy = rannoy::Rannoy::new(384);
    annoy.load(PathBuf::from("rss_embeddings.ann"));

    // 검색어 형식화
    let search_query = format!("제목: {}. 내용: {}", search_value, search_value);
    let embedding_search_text = embedding_sentence(service, search_query).await?;

    // search_k 값 추가
    let (result_ids, distances) = annoy.get_nns_by_vector(embedding_search_text, 200, 10000);

    // 거리 기반 필터링 적용
    let filtered_results: Vec<(i32, f32)> = result_ids
        .into_iter()
        .zip(distances.into_iter())
        .filter(|&(_, distance)| distance < DISTANCE_THRESHOLD)
        .collect();

    // 필터링된 결과 사용
    let filtered_ids: Vec<i32> = filtered_results.iter().map(|(id, _)| *id).collect();
    let filtered_distances: Vec<f32> = filtered_results.iter().map(|(_, dist)| *dist).collect();

    Ok((filtered_ids, filtered_distances))
}

#[allow(dead_code)]
pub async fn load_news_annoy(
    service: &State<EmbeddingService>,
    search_value: String,
) -> Result<(Vec<i32>, Vec<f32>), OmniNewsError> {
    let annoy = rannoy::Rannoy::new(384);
    annoy.load(PathBuf::from("news_embeddings.ann"));

    // 검색어 형식화
    let search_query = format!("제목: {}. 내용: {}", search_value, search_value);
    let embedding_search_text = embedding_sentence(service, search_query).await?;

    // search_k 값 추가
    let (result_ids, distances) = annoy.get_nns_by_vector(embedding_search_text, 10, 10000);

    // 거리 기반 필터링 적용
    let filtered_results: Vec<(i32, f32)> = result_ids
        .into_iter()
        .zip(distances.into_iter())
        .filter(|&(_, distance)| distance < DISTANCE_THRESHOLD)
        .collect();

    // 필터링된 결과 사용
    let filtered_ids: Vec<i32> = filtered_results.iter().map(|(id, _)| *id).collect();
    let filtered_distances: Vec<f32> = filtered_results.iter().map(|(_, dist)| *dist).collect();

    Ok((filtered_ids, filtered_distances))
}
