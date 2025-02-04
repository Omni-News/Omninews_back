use rocket::State;
use rss::Channel;
use scraper::{Html, Selector};
use sqlx::MySqlPool;

use crate::{
    model::rss::{NewRssChannel, Newticle, NewticleType, RssChannel, RssLink},
    repository::rss_channel_repository,
    service::morpheme_service,
};

use super::item_service;

pub async fn find_rss_channel_by_id(
    pool: &State<MySqlPool>,
    channel_id: i32,
) -> Result<RssChannel, sqlx::Error> {
    rss_channel_repository::select_rss_channel_by_id(pool, channel_id).await
}

pub async fn create_rss_and_morpheme(
    pool: &State<MySqlPool>,
    rss_link: RssLink,
) -> Result<i32, ()> {
    let rss_channel = parse_rss_link_to_channel(&rss_link.link).await?;

    let channel = make_rss_channel(rss_channel.clone());
    let channel_id = store_channel_and_morpheme(pool, channel).await?;

    item_service::crate_rss_item_and_morpheme(pool, rss_channel, channel_id).await;

    Ok(channel_id)
}

async fn parse_rss_link_to_channel(link: &str) -> Result<Channel, ()> {
    let response = reqwest::get(link).await.map_err(|_| ())?;
    let body = response.text().await.map_err(|_| ())?;
    Channel::read_from(body.as_bytes()).map_err(|_| ())
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
) -> Result<i32, ()> {
    let channel_id = store_rss_channel(pool, rss_channel.clone()).await?;

    morpheme_service::create_morpheme_by_newticle(
        pool,
        Newticle::NewRssChannel(rss_channel),
        NewticleType::Channel,
        channel_id,
    )
    .await;

    Ok(channel_id)
}

async fn store_rss_channel(pool: &State<MySqlPool>, channel: NewRssChannel) -> Result<i32, ()> {
    let channel_link = channel.channel_link.clone().unwrap_or_default();

    match rss_channel_repository::select_rss_channel_by_link(pool, channel_link).await {
        Ok(channel) => Ok(set_channel_rank(pool, channel).await?),
        Err(_) => Ok(rss_channel_repository::insert_rss_channel(pool, channel)
            .await
            .unwrap_or_default()),
    }
}

async fn set_channel_rank(pool: &State<MySqlPool>, mut channel: RssChannel) -> Result<i32, ()> {
    channel.channel_rank = channel.channel_rank.map(|e| e + 1);

    rss_channel_repository::update_rss_channel(pool, channel)
        .await
        .map_err(|_| ())
}

fn extract_html_to_passage_and_image_link(html: &str) -> (String, Option<String>) {
    let document = Html::parse_document(html);

    let passage_selector = Selector::parse("h3, p").unwrap();
    let image_selector = Selector::parse("img").unwrap();

    let extracted_text: Vec<String> = document
        .clone()
        .select(&passage_selector)
        .map(|e| e.text().collect::<Vec<_>>().join(" "))
        .collect();

    let image_link: Option<String> = document
        .select(&image_selector)
        .next()
        .map(|link| link.attr("src").unwrap().to_string());

    (extracted_text.join(" "), image_link)
}
