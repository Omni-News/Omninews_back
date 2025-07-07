use log::{info, warn};
use rocket::State;
use rss::Channel;
use sqlx::MySqlPool;

use crate::{
    dto::{
        rss::{request::CreateRssRequestDto, response::RssChannelResponseDto},
        search::request::SearchRequestDto,
    },
    model::{
        embedding::NewEmbedding,
        error::OmniNewsError,
        rss::{NewRssChannel, RssChannel},
        search::SearchType,
    },
    repository::rss_channel_repository,
    service::embedding_service,
    utils::{annoy_util::load_channel_annoy, embedding_util::EmbeddingService},
};

use super::item_service;

pub async fn create_rss_all(
    pool: &State<MySqlPool>,
    model: &State<EmbeddingService>,
    links: Vec<CreateRssRequestDto>,
) -> Result<bool, OmniNewsError> {
    for link in links {
        info!("[Service] Add : {}", link.rss_link);
        create_rss_and_embedding(pool, model, link).await?;
    }
    Ok(true)
}

pub async fn create_rss_and_embedding(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
    link: CreateRssRequestDto,
) -> Result<i32, OmniNewsError> {
    let rss_channel = parse_rss_link_to_channel(&link.rss_link).await?;

    let channel = make_rss_channel(rss_channel.clone(), link.rss_link);
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
    NewRssChannel::new(
        channel.title().to_string(),
        channel.link().to_string(),
        channel.description().to_string(),
        channel.image().map(|e| e.url().to_string()),
        channel.language().unwrap_or("None").to_string(),
        channel.generator().unwrap_or("None").to_string(),
        0,
        rss_link,
    )
}

async fn store_channel_and_embedding(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
    rss_channel: NewRssChannel,
) -> Result<i32, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_rss_link(
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

    match rss_channel_repository::select_rss_channel_by_rss_link(pool, channel_link).await {
        Ok(channel) => Ok(channel.channel_id.unwrap()),
        Err(_) => Ok(rss_channel_repository::insert_rss_channel(pool, channel)
            .await
            .map_err(|e| {
                error!("[Service] Failed to insert rss channel: {:?}", e);
                OmniNewsError::Database(e)
            })?),
    }
}

pub async fn find_rss_channel_by_id(
    pool: &State<MySqlPool>,
    channel_id: i32,
) -> Result<RssChannelResponseDto, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_id(pool, channel_id).await {
        Ok(res) => Ok(RssChannelResponseDto::from_model(res)),
        Err(e) => {
            error!("[Service] Failed to select rss channel by id: {:?}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn find_rss_channel_by_rss_link(
    pool: &State<MySqlPool>,
    channel_rss_link: String,
) -> Result<RssChannel, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_rss_link(pool, channel_rss_link).await {
        Ok(res) => Ok(res),
        Err(e) => {
            warn!("[Service] Failed to select rss channel by link: {:?}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn get_channel_list(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
    value: SearchRequestDto,
) -> Result<Vec<RssChannelResponseDto>, OmniNewsError> {
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
                b.channel_rank
                    .unwrap_or_default()
                    .cmp(&a.channel_rank.unwrap_or_default())
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

    Ok(RssChannelResponseDto::from_model_list(result))
}

// TODO 랭크 50순위 채널에서 20개 랜덤 반환
pub async fn get_recommend_channel(
    pool: &State<MySqlPool>,
) -> Result<Vec<RssChannelResponseDto>, OmniNewsError> {
    match rss_channel_repository::select_rss_channels_order_by_channel_rank(pool).await {
        Ok(res) => {
            //            let mut rng = rng();
            //            res.shuffle(&mut rng);
            //            Ok(res.into_iter().take(20).collect())
            Ok(RssChannelResponseDto::from_model_list(res))
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

pub async fn get_rss_preview(
    pool: &State<MySqlPool>,
    rss_link: String,
) -> Result<RssChannelResponseDto, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_channel_rss_link(pool, rss_link.clone())
        .await
    {
        Ok(res) => Ok(RssChannelResponseDto::from_model(res)),
        Err(_) => {
            let rss_channel = parse_rss_link_to_channel(&rss_link).await?;
            let new_channel = make_rss_channel(rss_channel, rss_link.clone());
            let channel = RssChannel::new(new_channel, rss_link);
            Ok(RssChannelResponseDto::from_model(channel))
        }
    }
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
