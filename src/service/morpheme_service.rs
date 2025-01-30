use std::collections::HashMap;

use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{MorphemeToSourceLink, NewMorpheme, NewRssChannel, NewRssItem},
    morpheme::{self, analyze::MorphemeError},
    repository::{morpheme_repository, morpheme_to_source_link_repository},
};

pub async fn validate_and_create_morpheme_for_rss(
    pool: &State<MySqlPool>,
    rss_item: NewRssItem,
    rss_id: u64,
) {
    let mut passage = String::new();
    passage.push_str(rss_item.rss_title.unwrap().as_str());
    passage.push_str(rss_item.rss_description.unwrap().as_str());

    if let Ok(morphemes) = get_morphemes(passage) {
        create_morpheme_and_source_link(
            pool,
            morphemes,
            "rss",
            rss_id,
            rss_item.rss_link.unwrap().as_str(),
        )
        .await
    }
}

pub async fn create_morpheme_for_channel(
    pool: &State<MySqlPool>,
    rss_channel: NewRssChannel,
    channel_id: u64,
) {
    let mut passage = String::new();
    passage.push_str(rss_channel.channel_title.unwrap().as_str());
    passage.push_str(rss_channel.channel_description.unwrap().as_str());

    if let Ok(morphemes) = get_morphemes(passage) {
        create_morpheme_and_source_link(
            pool,
            morphemes,
            "channel",
            channel_id,
            rss_channel.channel_link.unwrap().as_str(),
        )
        .await
    }
}

pub fn get_morphemes(passage: String) -> Result<Vec<NewMorpheme>, MorphemeError> {
    let mut count_map: HashMap<String, i32> = HashMap::new();

    let analyzed_morpheme = morpheme::analyze::analyze_morpheme(passage)?;
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

pub async fn create_morpheme_and_source_link(
    pool: &State<MySqlPool>,
    morphemes: Vec<NewMorpheme>,
    newticle_type: &str,
    newticle_id: u64,
    link: &str,
) {
    for morpheme in morphemes {
        let morpheme_word = morpheme.clone().morpheme_word.unwrap();
        let morpheme_id =
            match morpheme_repository::select_morpheme_by_word(pool, morpheme_word).await {
                Ok(existing_morpheme) => {
                    let mut update_morpheme = existing_morpheme.clone();
                    update_morpheme.morpheme_rank = update_morpheme.morpheme_rank.map(|e| e + 1);

                    morpheme_repository::update_morpheme_by_id(pool, update_morpheme)
                        .await
                        .unwrap()
                }
                Err(_) => morpheme_repository::insert_morpheme(pool, morpheme)
                    .await
                    .unwrap(),
            };

        let morpheme_to_source_link =
            MorphemeToSourceLink::new(newticle_type, newticle_id, morpheme_id, link);

        morpheme_to_source_link_repository::insert_morpheme_to_source_link(
            pool,
            morpheme_to_source_link,
        )
        .await
        .unwrap();
    }
}
