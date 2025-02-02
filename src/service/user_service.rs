use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{
        MorphemeToSourceLink, News, NewticleList, RssChannel, RssItem, SearchType, UserPrompt,
    },
    morpheme::analyze::analyze_morpheme,
};

use super::{
    morpheme_service::{self},
    rss_service,
};

pub async fn get_newticle_list(
    pool: &State<MySqlPool>,
    prompt: UserPrompt,
) -> Result<NewticleList, ()> {
    let prompt_morphemes = analyze_morpheme(prompt.user_prompt.unwrap()).unwrap();
    let morphemes_sources =
        morpheme_service::get_morphemes_sources_from_prompt(pool, prompt_morphemes)
            .await
            .unwrap();

    if let Some(search_type) = prompt.search_type {
        match search_type {
            SearchType::Accuracy => {
                Ok(search_newticles(pool, morphemes_sources.clone(), search_type).await)
            }
            SearchType::Popularity => {
                Ok(search_newticles(pool, morphemes_sources.clone(), search_type).await)
            }
            SearchType::Latest => {
                Ok(search_newticles(pool, morphemes_sources.clone(), search_type).await)
            }
        }
    } else {
        Err(())
    }
}

pub async fn get_rss_list(pool: &State<MySqlPool>, prompt: UserPrompt) -> Result<Vec<RssItem>, ()> {
    let prompt_morphemes = analyze_morpheme(prompt.user_prompt.unwrap()).unwrap();
    let morphemes_sources =
        morpheme_service::get_morphemes_sources_from_prompt(pool, prompt_morphemes)
            .await
            .unwrap();

    let mut result: Vec<RssItem> = Vec::new();
    for morpheme_source in morphemes_sources {
        if let Some(morpheme_id) = morpheme_source.morpheme_id {
            result = rss_service::find_rss_item_by_morpheme_id(
                pool,
                morpheme_id as u64,
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
    prompt: UserPrompt,
) -> Result<Vec<RssChannel>, ()> {
    let prompt_morphemes = analyze_morpheme(prompt.user_prompt.unwrap()).unwrap();
    let morphemes_sources =
        morpheme_service::get_morphemes_sources_from_prompt(pool, prompt_morphemes)
            .await
            .unwrap();

    let mut result: Vec<RssChannel> = Vec::new();
    for morpheme_source in morphemes_sources {
        if let Some(morpheme_id) = morpheme_source.morpheme_id {
            result = rss_service::find_rss_channel_by_morpheme_id(
                pool,
                morpheme_id as u64,
                prompt.search_type.clone().unwrap(),
            )
            .await
            .unwrap();
        }
    }

    Ok(result)
}
pub async fn search_newticles(
    pool: &State<MySqlPool>,
    morphemes_sources: Vec<MorphemeToSourceLink>,
    search_type: SearchType,
) -> NewticleList {
    let mut rss_list: Vec<RssItem> = Vec::new();
    let mut channel_list: Vec<RssChannel> = Vec::new();
    let news_list: Vec<News> = Vec::new();

    for morpheme_source in morphemes_sources {
        let rss_id = morpheme_source.rss_id.unwrap();
        let channel_id = morpheme_source.channel_id.unwrap();
        let news_id = morpheme_source.news_id.unwrap();

        if rss_id != 0 {
            rss_list.push(
                rss_service::find_rss_item_by_id(pool, rss_id as u64)
                    .await
                    .unwrap(),
            );
        } else if channel_id != 0 {
            channel_list.push(
                rss_service::find_rss_channel_by_id(pool, channel_id as u64)
                    .await
                    .unwrap(),
            );
        } else if news_id != 0 {
            // TODO 나중에 개발, Google news도 rss로 등록하면 됨.
        }
    }

    NewticleList {
        rss_list: Some(rss_list),
        channel_list: Some(channel_list),
        news_list: Some(news_list),
    }
}
