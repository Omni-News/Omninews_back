use log::info;
use rocket::State;
use rss::Channel;
use sqlx::MySqlPool;

use crate::{
    model::{
        error::OmniNewsError,
        rss::{NewRssChannel, Newticle, NewticleType, RssChannel, RssLink},
        search::{SearchRequest, SearchType},
    },
    morpheme::analyze::analyze_morpheme,
    repository::rss_channel_repository,
    service::morpheme_service,
};

use super::item_service;

pub async fn create_rss_all(
    pool: &State<MySqlPool>,
    links: Vec<RssLink>,
) -> Result<bool, OmniNewsError> {
    for link in links {
        info!("Add : {}", link.link);
        create_rss_and_morpheme(pool, link).await?;
    }
    Ok(true)
}

pub async fn create_rss_and_morpheme(
    pool: &State<MySqlPool>,
    rss_link: RssLink,
) -> Result<i32, OmniNewsError> {
    let rss_channel = parse_rss_link_to_channel(&rss_link.link).await?;

    let channel = make_rss_channel(rss_channel.clone());
    let channel_id = store_channel_and_morpheme(pool, channel).await?;

    item_service::crate_rss_item_and_morpheme(pool, rss_channel, channel_id).await?;

    Ok(channel_id)
}

async fn parse_rss_link_to_channel(link: &str) -> Result<Channel, OmniNewsError> {
    let response = reqwest::get(link).await.map_err(|e| {
        error!("Not found url : {}", link);
        OmniNewsError::Request(e)
    })?;
    let body = response.text().await.map_err(OmniNewsError::Request)?;
    Channel::read_from(body.as_bytes()).map_err(|e| {
        error!("Failed to read from rss body: {:?}", e);
        OmniNewsError::Parse
    })
}

fn make_rss_channel(channel: Channel) -> NewRssChannel {
    NewRssChannel {
        channel_title: Some(channel.title().to_string()),
        channel_link: Some(channel.link().to_string()),
        channel_description: Some(channel.description().to_string()),
        channel_image_url: channel.image().map(|e| e.url().to_string()),
        channel_language: Some(channel.language().unwrap_or("None").to_string()),
        rss_generator: Some(channel.generator().unwrap_or("None").to_string()),
        channel_rank: Some(1),
    }
}

async fn store_channel_and_morpheme(
    pool: &State<MySqlPool>,
    rss_channel: NewRssChannel,
) -> Result<i32, OmniNewsError> {
    let channel_id = store_rss_channel(pool, rss_channel.clone()).await?;

    let _ = morpheme_service::create_morpheme_by_newticle(
        pool,
        Newticle::NewRssChannel(rss_channel),
        NewticleType::Channel,
        channel_id,
    )
    .await?;

    Ok(channel_id)
}

async fn store_rss_channel(
    pool: &State<MySqlPool>,
    channel: NewRssChannel,
) -> Result<i32, OmniNewsError> {
    let channel_link = channel.channel_link.clone().unwrap_or_default();

    match rss_channel_repository::select_rss_channel_by_link(pool, channel_link).await {
        Ok(channel) => Ok(set_channel_rank(pool, channel).await?),
        Err(_) => Ok(rss_channel_repository::insert_rss_channel(pool, channel)
            .await
            .map_err(|e| {
                error!("Failed to insert rss channel: {:?}", e);
                OmniNewsError::Database(e)
            })?),
    }
}

async fn set_channel_rank(
    pool: &State<MySqlPool>,
    mut channel: RssChannel,
) -> Result<i32, OmniNewsError> {
    channel.channel_rank = channel.channel_rank.map(|e| e + 1);

    rss_channel_repository::update_rss_channel(pool, channel)
        .await
        .map_err(|e| {
            error!("Failed to set channel rank: {:?}", e);
            OmniNewsError::Database(e)
        })
}

pub async fn get_channel_list(
    pool: &State<MySqlPool>,
    value: SearchRequest,
) -> Result<Vec<RssChannel>, OmniNewsError> {
    let search_morphemes =
        analyze_morpheme(value.search_value.unwrap()).map_err(OmniNewsError::Morpheme)?;

    let morphemes_sources =
        morpheme_service::get_morphemes_sources_by_search_value(pool, search_morphemes).await?;

    let mut result: Vec<RssChannel> = Vec::new();
    for morpheme_source in morphemes_sources {
        if let Some(morpheme_id) = morpheme_source.morpheme_id {
            result = match value.search_type.clone().unwrap() {
                SearchType::Accuracy => {
                    rss_channel_repository::select_rss_channels_by_morpheme_id_order_by_source_rank(
                        pool,
                        morpheme_id,
                    )
                    .await
                    .map_err(|e| {
                            error!("Faild to select channels by morpheme id order by source rank: {}", e);
                            OmniNewsError::Database(e)
                        })?
                }
                SearchType::Popularity => {
                    rss_channel_repository::select_rss_channels_by_morpheme_id_order_by_channel_rank(
                        pool,
                        morpheme_id,
                    )
                    .await
                    .map_err(|e| {
                            error!("Failed to select channels by morpheme id order by channel rank: {}", e);
                            OmniNewsError::Database(e)
                        })?
                }
                SearchType::Latest => Vec::new(), // Channel doesn't have pub_date()
            };
        }
    }

    Ok(result)
}
