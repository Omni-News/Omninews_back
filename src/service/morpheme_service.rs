use std::collections::HashMap;

use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{
        morpheme::{Morpheme, MorphemeLinkMapping, NewMorpheme},
        rss::{Newticle, NewticleType},
    },
    morpheme::analyze::{analyze_morpheme, MorphemeError},
    repository::{morpheme_link_mapping_repository, morpheme_repository},
};

pub async fn get_morphemes_sources_by_search(
    pool: &State<MySqlPool>,
    search_morphemes: Vec<String>,
) -> Result<Vec<MorphemeLinkMapping>, ()> {
    let mut result = Vec::new();

    for prompt_morpheme in search_morphemes {
        let morphemes = morpheme_repository::select_morphemes_by_morpheme(pool, prompt_morpheme)
            .await
            .map_err(|_| ())?;

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
) -> Result<bool, ()> {
    let (passage, newticle_image_url) = match newticle {
        Newticle::NewRssChannel(channel) => (
            format!(
                "{} {}",
                channel.channel_title.unwrap(),
                channel.channel_description.unwrap()
            ),
            channel.channel_image_url.unwrap(),
        ),

        Newticle::NewRssItem(item) => (
            format!(
                "{} {}",
                item.rss_title.unwrap(),
                item.rss_description.unwrap()
            ),
            item.rss_image_link.unwrap(),
        ),
    };

    if let Ok(morphemes) = get_morphemes_and_rank(passage) {
        create_morpheme_and_source_link(pool, morphemes, newticle_type, id, newticle_image_url)
            .await
    } else {
        // TODO 에러처리
        Err(())
    }
}

fn get_morphemes_and_rank(passage: String) -> Result<Vec<NewMorpheme>, MorphemeError> {
    let mut count_map: HashMap<String, i32> = HashMap::new();

    let analyzed_morpheme = analyze_morpheme(passage)?;

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
) -> Result<bool, ()> {
    for morpheme in morphemes {
        let morpheme_word = morpheme.morpheme_word.clone().unwrap_or_default();

        let morpheme_id =
            match morpheme_repository::select_morpheme_by_word(pool, morpheme_word).await {
                Ok(morpheme) => set_morpheme_rank(pool, morpheme).await.unwrap_or_default(),

                Err(_) => morpheme_repository::insert_morpheme(pool, morpheme.clone())
                    .await
                    .unwrap_or_default(),
            };

        match store_morpheme_link_mapping(
            pool,
            newticle_type.clone(),
            newticle_id,
            morpheme_id,
            link.to_string(),
            morpheme.morpheme_rank,
        )
        .await
        {
            Ok(_) => (),
            Err(_) => {
                //TODO 에러처리
            }
        }
    }

    Ok(true)
}

async fn set_morpheme_rank(pool: &State<MySqlPool>, mut morpheme: Morpheme) -> Result<i32, ()> {
    morpheme.morpheme_rank = morpheme.morpheme_rank.map(|e| e + 1);

    morpheme_repository::update_morpheme_by_id(pool, morpheme)
        .await
        .map_err(|_| ())
}

async fn store_morpheme_link_mapping(
    pool: &State<MySqlPool>,
    newticle_type: NewticleType,
    newticle_id: i32,
    morpheme_id: i32,
    source_link: String,
    source_rank: Option<i32>,
) -> Result<bool, ()> {
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
        Err(_) => Err(()),
    }
}
