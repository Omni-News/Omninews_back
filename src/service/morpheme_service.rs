use std::collections::HashMap;

use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{
        error::{MorphemeError, OmniNewsError},
        morpheme::{Morpheme, MorphemeLinkMapping, NewMorpheme},
        rss::{Newticle, NewticleType},
    },
    morpheme::analyze::analyze_morpheme,
    repository::{morpheme_link_mapping_repository, morpheme_repository},
};

pub async fn get_morphemes_sources_by_search_value(
    pool: &State<MySqlPool>,
    search_morphemes: Vec<String>,
) -> Result<Vec<MorphemeLinkMapping>, OmniNewsError> {
    let mut result = Vec::new();

    for prompt_morpheme in search_morphemes {
        let morphemes = morpheme_repository::select_morphemes_by_morpheme(pool, prompt_morpheme)
            .await
            .map_err(|e| {
                error!("[Service] Failed to select morphemes by morpheme: {}", e);
                OmniNewsError::Database(e)
            })?;

        for morpheme in morphemes {
            if let Some(morpheme_id) = morpheme.morpheme_id {
                let mut links =
                    morpheme_link_mapping_repository::select_morphemes_link_by_morpheme_id(
                        pool,
                        morpheme_id,
                    )
                    .await
                    .unwrap_or_else(|_| vec![]); // 에러 발생 시 빈 벡터 반환

                result.append(&mut links);
            }
        }
    }

    Ok(result)
}

pub async fn create_morpheme_by_newticle(
    pool: &State<MySqlPool>,
    newticle: Newticle,
    newticle_type: NewticleType,
    id: i32,
) -> Result<bool, OmniNewsError> {
    let (passage, newticle_link) = match newticle {
        Newticle::NewRssChannel(channel) => (
            format!(
                "{} {}",
                channel.channel_title.unwrap_or_default(),
                channel.channel_description.unwrap_or_default()
            ),
            channel.channel_image_url.unwrap_or_default(),
        ),

        Newticle::NewRssItem(item) => (
            format!(
                "{} {}",
                item.rss_title.unwrap_or_default(),
                item.rss_description.unwrap_or_default()
            ),
            item.rss_link.unwrap_or_default(),
        ),
    };

    match get_morphemes_and_rank(passage) {
        Ok(morphemes) => {
            create_morpheme_and_source_link(pool, morphemes, newticle_type, id, newticle_link).await
        }
        Err(e) => Err(OmniNewsError::Morpheme(e)),
    }
}

fn get_morphemes_and_rank(passage: String) -> Result<Vec<NewMorpheme>, MorphemeError> {
    let mut count_map: HashMap<String, i32> = HashMap::new();

    let analyzed_morpheme = analyze_morpheme(passage).unwrap_or_default();

    // Remove deplicate morphemes and increase rank by 1.
    for morpheme in analyzed_morpheme {
        *count_map.entry(morpheme).or_insert(0) += 1;
    }

    Ok(count_map
        .into_iter()
        .map(|(morpheme, count)| NewMorpheme {
            morpheme_word: Some(morpheme),
            morpheme_rank: Some(count),
        })
        .collect())
}

async fn create_morpheme_and_source_link(
    pool: &State<MySqlPool>,
    morphemes: Vec<NewMorpheme>,
    newticle_type: NewticleType,
    newticle_id: i32,
    link: String,
) -> Result<bool, OmniNewsError> {
    for morpheme in morphemes {
        let morpheme_word = morpheme.morpheme_word.clone().unwrap_or_default();

        let morpheme_id =
            match morpheme_repository::select_morpheme_by_word(pool, morpheme_word).await {
                Ok(morpheme) => set_morpheme_rank(pool, morpheme).await.unwrap_or_default(),

                Err(_) => morpheme_repository::insert_morpheme(pool, morpheme.clone())
                    .await
                    .map_err(|e| {
                        error!("[Service] Failed to insert morpheme: {}", e);
                        OmniNewsError::Database(e)
                    })?,
            };

        let _ = store_morpheme_link_mapping(
            pool,
            newticle_type.clone(),
            newticle_id,
            morpheme_id,
            link.to_string(),
            morpheme.morpheme_rank,
        )
        .await?;
    }

    Ok(true)
}

async fn set_morpheme_rank(
    pool: &State<MySqlPool>,
    mut morpheme: Morpheme,
) -> Result<i32, OmniNewsError> {
    morpheme.morpheme_rank = morpheme.morpheme_rank.map(|e| e + 1);

    morpheme_repository::update_morpheme_by_id(pool, morpheme)
        .await
        .map_err(|e| {
            error!("[Service] Failed to set morpheme rank: {}", e);
            OmniNewsError::Database(e)
        })
}

async fn store_morpheme_link_mapping(
    pool: &State<MySqlPool>,
    newticle_type: NewticleType,
    newticle_id: i32,
    morpheme_id: i32,
    source_link: String,
    source_rank: Option<i32>,
) -> Result<bool, OmniNewsError> {
    let morpheme_link_mapping = MorphemeLinkMapping::new(
        newticle_type,
        Some(newticle_id),
        Some(morpheme_id),
        Some(source_link),
        source_rank,
    );

    match morpheme_link_mapping_repository::insert_morpheme_link_mapping(
        pool,
        morpheme_link_mapping,
    )
    .await
    {
        Ok(_) => Ok(true),
        Err(e) => {
            error!("[Service] Failed to insert morpheme link mapping: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
