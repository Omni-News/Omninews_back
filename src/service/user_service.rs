use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{
        rss::{RssChannel, RssItem},
        search::SearchRequest,
    },
    morpheme::analyze::analyze_morpheme,
};

use super::{
    morpheme_service::{self},
    rss::search_service,
};

pub async fn get_rss_list(
    pool: &State<MySqlPool>,
    prompt: SearchRequest,
) -> Result<Vec<RssItem>, ()> {
    let prompt_morphemes = analyze_morpheme(prompt.search_value.unwrap()).unwrap();
    let morphemes_sources =
        morpheme_service::get_morphemes_sources_by_search(pool, prompt_morphemes)
            .await
            .unwrap();

    let mut result: Vec<RssItem> = Vec::new();
    for morpheme_source in morphemes_sources {
        if let Some(morpheme_id) = morpheme_source.morpheme_id {
            result = search_service::find_rss_item_by_morpheme_id(
                pool,
                morpheme_id,
                prompt.search_type.clone().unwrap(),
            )
            .await
            .unwrap();
        }
    }

    Ok(result)
}

pub async fn get_channel_list(
    pool: &State<MySqlPool>,
    value: SearchRequest,
) -> Result<Vec<RssChannel>, ()> {
    let search_morphemes = analyze_morpheme(value.search_value.unwrap()).unwrap();
    let morphemes_sources =
        morpheme_service::get_morphemes_sources_by_search(pool, search_morphemes)
            .await
            .unwrap();

    let mut result: Vec<RssChannel> = Vec::new();
    for morpheme_source in morphemes_sources {
        if let Some(morpheme_id) = morpheme_source.morpheme_id {
            result = search_service::find_rss_channel_by_morpheme_id(
                pool,
                morpheme_id,
                value.search_type.clone().unwrap(),
            )
            .await
            .unwrap();
        }
    }

    Ok(result)
}
