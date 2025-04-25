use log::{info, warn};
use rand::{rng, seq::SliceRandom};
use rocket::State;
use rss::Channel;
use sqlx::MySqlPool;

use crate::{
    model::{
        embedding::NewEmbedding,
        error::OmniNewsError,
        rss::{NewRssChannel, RssChannel, RssLink},
        search::{SearchRequest, SearchType},
    },
    repository::rss_channel_repository,
    service::embedding_service,
    utils::{annoy_util::load_channel_annoy, embedding_util::EmbeddingService},
};

use super::item_service;

pub async fn create_rss_all(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    links: Vec<RssLink>,
) -> Result<bool, OmniNewsError> {
    for link in links {
        info!("[Service] Add : {}", link.link);
        create_rss_and_embedding(pool, model, link).await?;
    }
    Ok(true)
}

pub async fn create_rss_and_embedding(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
    rss_link: RssLink,
) -> Result<i32, OmniNewsError> {
    let rss_channel = parse_rss_link_to_channel(&rss_link.link).await?;

    let channel = make_rss_channel(rss_channel.clone(), rss_link.link);
    let channel_id = store_channel_and_embedding(pool, embedding_service, channel).await?;

    let _ = item_service::crate_rss_item_and_embedding(
        pool,
        embedding_service,
        rss_channel,
        channel_id,
    )
    .await
    .map_err(|e| {
        error!("[Service] Failed to create rss item and embedding: {:?}", e);
        e
    });

    Ok(channel_id)
}

async fn parse_rss_link_to_channel(link: &str) -> Result<Channel, OmniNewsError> {
    let response = reqwest::get(link).await.map_err(|e| {
        error!("[Service] Not found url : {}", link);
        OmniNewsError::Request(e)
    })?;
    let body = response.text().await.map_err(OmniNewsError::Request)?;
    Channel::read_from(body.as_bytes()).map_err(|e| {
        error!("[Service] Failed to read from rss body: {:?}", e);
        OmniNewsError::Parse
    })
}

fn make_rss_channel(channel: Channel, rss_link: String) -> NewRssChannel {
    NewRssChannel {
        channel_title: Some(channel.title().to_string()),
        channel_link: Some(channel.link().to_string()),
        channel_description: Some(channel.description().to_string()),
        channel_image_url: channel.image().map(|e| e.url().to_string()),
        channel_language: Some(channel.language().unwrap_or("None").to_string()),
        rss_generator: Some(channel.generator().unwrap_or("None").to_string()),
        channel_rank: Some(0),
        channel_rss_link: Some(rss_link),
    }
}

async fn store_channel_and_embedding(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
    rss_channel: NewRssChannel,
) -> Result<i32, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_link(
        pool,
        rss_channel.channel_link.clone().unwrap_or_default(),
    )
    .await
    {
        Ok(_) => {
            warn!(
                "[Service] Already exist channel: {}",
                rss_channel.channel_link.clone().unwrap()
            );
            Err(OmniNewsError::AlreadyExists)
        }
        Err(_) => {
            let channel_id = store_rss_channel(pool, rss_channel.clone()).await?;

            let sentence = format!(
                "{}\n{}",
                rss_channel.channel_title.unwrap_or_default(),
                rss_channel.channel_description.unwrap_or_default()
            );
            let embedding = NewEmbedding {
                embedding_value: None,
                channel_id: Some(channel_id),
                rss_id: None,
                news_id: None,
                embedding_source_rank: Some(0),
            };
            embedding_service::create_embedding(pool, embedding_service, sentence, embedding)
                .await?;
            Ok(channel_id)
        }
    }
}

async fn store_rss_channel(
    pool: &State<MySqlPool>,
    channel: NewRssChannel,
) -> Result<i32, OmniNewsError> {
    let channel_link = channel.channel_link.clone().unwrap_or_default();

    match rss_channel_repository::select_rss_channel_by_link(pool, channel_link).await {
        Ok(channel) => Ok(channel.channel_id.unwrap()),
        Err(_) => Ok(rss_channel_repository::insert_rss_channel(pool, channel)
            .await
            .map_err(|e| {
                error!("[Service] Failed to insert rss channel: {:?}", e);
                OmniNewsError::Database(e)
            })?),
    }
}

pub async fn get_rss_channel_by_id(
    pool: &State<MySqlPool>,
    channel_id: i32,
) -> Result<RssChannel, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_id(pool, channel_id).await {
        Ok(res) => Ok(res),
        Err(e) => {
            error!("[Service] Failed to select rss channel by id: {:?}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn get_channel_list(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
    value: SearchRequest,
) -> Result<Vec<RssChannel>, OmniNewsError> {
    let load_annoy = load_channel_annoy(embedding_service, value.search_value.unwrap()).await?;

    let result = match value.search_type.clone().unwrap() {
        SearchType::Accuracy => {
            let mut res = Vec::new();
            for id in load_annoy.0.iter() {
                if let Ok(channel) =
                    rss_channel_repository::select_rss_channel_by_embedding_id(pool, *id).await
                {
                    res.push(channel);
                }
            }

            res
        }
        SearchType::Popularity => {
            let mut res = Vec::new();
            for id in load_annoy.0.iter() {
                if let Ok(channel) =
                    rss_channel_repository::select_rss_channel_by_embedding_id(pool, *id).await
                {
                    res.push(channel);
                }
            }

            res.sort_by(|a, b| {
                a.channel_rank
                    .unwrap_or_default()
                    .cmp(&b.channel_rank.unwrap_or_default())
            });

            res
        }

        // 스키마에 날짜 컬럼 없어 정확순으로 대체
        SearchType::Latest => {
            let mut res = Vec::new();
            for id in load_annoy.0.iter() {
                if let Ok(channel) =
                    rss_channel_repository::select_rss_channel_by_embedding_id(pool, *id).await
                {
                    res.push(channel);
                }
            }

            res
        }
    };

    Ok(result)
}

// 랭크 50순위 채널에서 20개 랜덤 반환
pub async fn get_recommend_channel(
    pool: &State<MySqlPool>,
) -> Result<Vec<RssChannel>, OmniNewsError> {
    match rss_channel_repository::select_rss_channels_order_by_channel_rank(pool).await {
        Ok(mut res) => {
            let mut rng = rng();
            res.shuffle(&mut rng);
            Ok(res.into_iter().take(20).collect())
        }
        Err(e) => {
            error!(
                "[Service] Failed to select channels order by channel rank: {:?}",
                e
            );
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn get_rss_preview(rss_link: String) -> Result<RssChannel, OmniNewsError> {
    let rss_channel = parse_rss_link_to_channel(&rss_link).await?;
    let new_channel = make_rss_channel(rss_channel, rss_link.clone());
    let channel = RssChannel {
        channel_id: Some(0),
        channel_title: Some(new_channel.channel_title.unwrap_or_default()),
        channel_image_url: Some(new_channel.channel_image_url.unwrap_or_default()),
        channel_description: Some(new_channel.channel_description.unwrap_or_default()),
        channel_link: Some(new_channel.channel_link.unwrap_or_default()),
        channel_language: Some(new_channel.channel_language.unwrap_or_default()),
        channel_rank: Some(0),
        rss_generator: Some(new_channel.rss_generator.unwrap_or_default()),
        channel_rss_link: Some(rss_link),
    };

    Ok(channel)
}

pub async fn is_channel_exist_by_link(
    pool: &State<MySqlPool>,
    channel_link: String,
) -> Result<bool, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_channel_rss_link(pool, channel_link).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub async fn is_channel_exist_by_id(
    pool: &State<MySqlPool>,
    channel_id: i32,
) -> Result<bool, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_id(pool, channel_id).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub async fn update_rss_channel_rank(
    pool: &State<MySqlPool>,
    channel_id: i32,
    num: i32,
) -> Result<bool, OmniNewsError> {
    match rss_channel_repository::update_rss_channel_rank_by_id(pool, channel_id, num).await {
        Ok(res) => Ok(res),
        Err(e) => {
            error!("[Service] Failed to update rss channel rank: {:?}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
