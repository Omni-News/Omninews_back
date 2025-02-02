use chrono::{DateTime, NaiveDateTime};
use rocket::State;
use rss::{Channel, Item};
use scraper::{Html, Selector};
use sqlx::MySqlPool;

use crate::{
    model::{NewRssChannel, NewRssItem, RssChannel, RssItem, RssLink, SearchType},
    repository::{
        rss_channel_repository::{self},
        rss_item_repository,
    },
};

use super::morpheme_service::{create_morpheme_for_channel, validate_and_create_morpheme_for_rss};

pub async fn find_rss_channel_by_id(
    pool: &State<MySqlPool>,
    channel_id: u64,
) -> Result<RssChannel, sqlx::Error> {
    rss_channel_repository::select_rss_channel_by_id(pool, channel_id).await
}

pub async fn find_rss_item_by_id(
    pool: &State<MySqlPool>,
    rss_id: u64,
) -> Result<RssItem, sqlx::Error> {
    rss_item_repository::select_rss_item_by_id(pool, rss_id).await
}

pub async fn find_rss_channel_by_morpheme_id(
    pool: &State<MySqlPool>,
    morpheme_id: u64,
    order_by: SearchType,
) -> Result<Vec<RssChannel>, sqlx::Error> {
    match order_by {
        SearchType::Accuracy => Ok(
            rss_channel_repository::select_rss_channels_by_morpheme_id_order_by_source_rank(
                pool,
                morpheme_id,
            )
            .await
            .unwrap(),
        ),
        SearchType::Popularity => Ok(
            rss_channel_repository::select_rss_channels_by_morpheme_id_order_by_channel_rank(
                pool,
                morpheme_id,
            )
            .await
            .unwrap(),
        ),
        SearchType::Latest => Ok(Vec::<RssChannel>::new()),
    }
}
pub async fn find_rss_item_by_morpheme_id(
    pool: &State<MySqlPool>,
    morpheme_id: u64,
    order_by: SearchType,
) -> Result<Vec<RssItem>, sqlx::Error> {
    match order_by {
        SearchType::Accuracy => {
            rss_item_repository::select_rss_items_by_morpheme_id_order_by_source_rank(
                pool,
                morpheme_id,
            )
            .await
        }
        SearchType::Popularity => {
            rss_item_repository::select_rss_items_by_morpheme_id_order_by_rss_rank(
                pool,
                morpheme_id,
            )
            .await
        }
        SearchType::Latest => {
            rss_item_repository::select_rss_items_by_morpheme_id_order_by_pub_date(
                pool,
                morpheme_id,
            )
            .await
        }
    }
}

pub async fn create_rss_and_morpheme(
    pool: &State<MySqlPool>,
    rss_link: RssLink,
) -> Result<u64, ()> {
    let link = rss_link.link;

    let response = reqwest::get(&link).await.unwrap();
    let body = response.text().await.unwrap();
    let mut channel = Channel::read_from(body.as_bytes()).unwrap();

    let rss_channel = make_rss_channel(channel.clone());
    let rss_channel_id = validate_and_create_rss_channel(pool, rss_channel.clone())
        .await
        .unwrap();
    create_morpheme_for_channel(pool, rss_channel.clone(), rss_channel_id).await;

    for item in channel.items_mut() {
        let description = item.description().unwrap_or("None");
        let (extracted_html, image_link) = extract_html_to_passage_and_image_link(description);
        item.set_description(extracted_html.clone());

        let rss_item = make_rss_item(
            rss_channel_id,
            item,
            image_link.map_or(
                rss_channel.clone().channel_image_url.unwrap_or_default(),
                |link| link,
            ),
        );
        let rss_id = validate_and_create_rss_item(pool, rss_item.clone())
            .await
            .unwrap();
        validate_and_create_morpheme_for_rss(pool, rss_item, rss_id).await;
    }
    Ok(rss_channel_id)
}

// TODO 사용자가 rss를 눌렀을 때도 rank +1 처리.
async fn validate_and_create_rss_item(
    pool: &State<MySqlPool>,
    mut rss_item: NewRssItem,
) -> Result<u64, sqlx::Error> {
    let rss_link = rss_item.clone().rss_link.unwrap();
    if let Ok(rss_item) = rss_item_repository::select_rss_item_by_link(pool, rss_link).await {
        let mut update_rss_item = rss_item.clone();
        update_rss_item.rss_rank = rss_item.rss_rank.map(|e| e + 1);

        return rss_item_repository::update_rss_item_by_id(pool, update_rss_item).await;
    }

    if let Some(str) = rss_item.rss_description.as_mut() {
        *str = str.chars().take(200).collect()
    }

    rss_item_repository::insert_rss_item(pool, rss_item).await
}

async fn validate_and_create_rss_channel(
    pool: &State<MySqlPool>,
    rss_channel: NewRssChannel,
) -> Result<u64, sqlx::Error> {
    let channel_link = rss_channel.clone().channel_link.unwrap();
    if let Ok(rss_channel) =
        rss_channel_repository::select_rss_channel_by_link(pool, channel_link).await
    {
        let mut update_rss_channel = rss_channel.clone();
        update_rss_channel.channel_rank = rss_channel.channel_rank.map(|e| e + 1);

        return rss_channel_repository::update_rss_channel_by_id(pool, update_rss_channel).await;
    }
    rss_channel_repository::insert_rss_channel(pool, rss_channel).await
}

fn parse_pub_date(pub_date_str: Option<&str>) -> Option<NaiveDateTime> {
    pub_date_str
        .and_then(|date_str| DateTime::parse_from_rfc2822(date_str).ok())
        .map(|dt| dt.naive_utc())
        .or_else(|| {
            Some(
                NaiveDateTime::parse_from_str("1970-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S")
                    .ok()
                    .unwrap(),
            )
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
fn make_rss_item(rss_channel_id: u64, item: &Item, rss_image_link: String) -> NewRssItem {
    NewRssItem {
        channel_id: Some(rss_channel_id as i32),
        rss_title: Some(item.title().unwrap_or("None").to_string()),
        rss_description: Some(item.description().unwrap_or("None").to_string()),
        rss_link: Some(item.link().unwrap_or("None").to_string()),
        rss_author: Some(item.author().unwrap_or("None").to_string()),
        rss_pub_date: parse_pub_date(item.pub_date()),
        rss_rank: Some(1),
        rss_image_link: Some(rss_image_link),
    }
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
