use crate::{
    model::rss::{NewRssItem, Newticle, NewticleType, RssItem},
    repository::rss_item_repository,
    service::morpheme_service::create_morpheme_by_newticle,
};
use chrono::{DateTime, NaiveDateTime};
use rocket::State;
use rss::{Channel, Item};
use scraper::{Html, Selector};
use sqlx::MySqlPool;

pub async fn find_rss_item_by_id(
    pool: &State<MySqlPool>,
    rss_id: i32,
) -> Result<RssItem, sqlx::Error> {
    rss_item_repository::select_rss_item_by_id(pool, rss_id).await
}

pub async fn crate_rss_item_and_morpheme(
    pool: &State<MySqlPool>,
    mut channel: Channel,
    channel_id: i32,
) -> Result<(), ()> {
    let channel_image_url = channel
        .image()
        .map_or(String::new(), |image| image.url().to_string());

    // TODO rss cateogory 미구현 상태.
    for rss_item in channel.items_mut() {
        let description = rss_item.description().unwrap_or("None");
        let (extracted_description, item_image_link) =
            extract_html_to_passage_and_image_link(description);
        rss_item.set_description(extracted_description);
        let item_image_link = use_channel_url_if_none(item_image_link, channel_image_url.clone());

        let item = make_rss_item(channel_id, rss_item, item_image_link);

        let item_id = store_rss_item_and_morpheme(pool, item.clone())
            .await
            .unwrap();

        create_morpheme_by_newticle(pool, Newticle::NewRssItem(item), NewticleType::Rss, item_id)
            .await?;
    }
    Ok(())
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

fn use_channel_url_if_none(link: Option<String>, channel_image_url: String) -> String {
    match link {
        Some(link) => link,
        None => channel_image_url,
    }
}

fn make_rss_item(channel_id: i32, item: &Item, item_image_link: String) -> NewRssItem {
    NewRssItem {
        channel_id: Some(channel_id),
        rss_title: Some(item.title().unwrap_or("None").to_string()),
        rss_description: Some(item.description().unwrap_or("None").to_string()),
        rss_link: Some(item.link().unwrap_or("None").to_string()),
        rss_author: Some(item.author().unwrap_or("None").to_string()),
        rss_pub_date: parse_pub_date(item.pub_date()),
        rss_rank: Some(1),
        rss_image_link: Some(item_image_link),
    }
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

// TODO 사용자가 rss를 눌렀을 때도 rank +1 처리.
async fn store_rss_item_and_morpheme(
    pool: &State<MySqlPool>,
    mut rss_item: NewRssItem,
) -> Result<i32, ()> {
    let item_link = rss_item.rss_link.clone().unwrap_or_default();

    if let Some(str) = rss_item.rss_description.as_mut() {
        *str = str.chars().take(200).collect()
    };

    match rss_item_repository::select_item_by_link(pool, item_link).await {
        Ok(item) => Ok(set_item_rank(pool, item).await?),
        Err(_) => rss_item_repository::insert_rss_item(pool, rss_item)
            .await
            .map_err(|_| ()),
    }
}

async fn set_item_rank(pool: &State<MySqlPool>, mut item: RssItem) -> Result<i32, ()> {
    item.rss_rank = item.rss_rank.map(|r| r + 1);

    rss_item_repository::update_rss_item(pool, item)
        .await
        .map_err(|_| ())
}
