use std::collections::HashMap;

use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{Morpheme, MorphemeToSourceLink, RssChannel, RssItem},
    morpheme::{self, analyze::MorphemeError},
    repository::{morpheme_repository, morpheme_to_source_link_repository},
};

pub async fn create_morpheme_for_rss(pool: &State<MySqlPool>, rss_item: RssItem, rss_id: u64) {
    let mut passage = String::new();
    passage.push_str(rss_item.rss_title.as_str());
    passage.push_str(rss_item.rss_description.as_str());

    if let Ok(morphemes) = get_morphemes(passage) {
        create_morpheme_and_source_link(pool, morphemes, "rss", rss_id, rss_item.rss_link.as_str())
            .await
    }
}

pub async fn create_morpheme_for_channel(
    pool: &State<MySqlPool>,
    rss_channel: RssChannel,
    channel_id: u64,
) {
    let mut passage = String::new();
    passage.push_str(rss_channel.channel_title.as_str());
    passage.push_str(rss_channel.channel_description.as_str());

    if let Ok(morphemes) = get_morphemes(passage) {
        create_morpheme_and_source_link(
            pool,
            morphemes,
            "channel",
            channel_id,
            rss_channel.channel_link.as_str(),
        )
        .await
    }
}

pub fn get_morphemes(passage: String) -> Result<Vec<Morpheme>, MorphemeError> {
    let mut count_map: HashMap<String, u64> = HashMap::new();

    let analyzed_morpheme = morpheme::analyze::analyze_morpheme(passage)?;
    for morpheme in analyzed_morpheme {
        *count_map.entry(morpheme).or_insert(0) += 1;
    }

    Ok(count_map
        .into_iter()
        .map(|(morpheme, count)| Morpheme {
            morpheme_word: morpheme,
            morpheme_rank: count,
        })
        .collect())
}

pub async fn create_morpheme_and_source_link(
    pool: &State<MySqlPool>,
    morphemes: Vec<Morpheme>,
    newticle_type: &str,
    newticle_id: u64,
    link: &str,
) {
    for morpheme in morphemes {
        let morpheme_id = morpheme_repository::insert_morpheme(pool, morpheme)
            .await
            .unwrap();
        // TODO rss, rss_channel, news만들 때마다 형태소 추가, ㅡmorpheme_to_source_link도 추가
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
