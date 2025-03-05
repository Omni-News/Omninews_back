use crate::{
    model::{
        error::OmniNewsError,
        rss::{NewRssItem, Newticle, NewticleType, RssItem},
        search::{SearchRequest, SearchType},
    },
    morpheme::analyze::analyze_morpheme,
    repository::rss_item_repository,
    service::morpheme_service::{self, create_morpheme_by_newticle},
};
use chrono::{DateTime, NaiveDateTime};
use log::info;
use rand::{seq::SliceRandom, thread_rng};
use rocket::State;
use rss::{Channel, Item};
use scraper::{Html, Selector};
use sqlx::{pool, MySqlPool};

pub async fn crate_rss_item_and_morpheme(
    pool: &State<MySqlPool>,
    mut channel: Channel,
    channel_id: i32,
) -> Result<(), OmniNewsError> {
    let channel_image_url = channel
        .image()
        .map_or(String::new(), |image| image.url().to_string()); // TODO rss cateogory 미구현 상태.
                                                                 //
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

    let image_link = document
        .select(&image_selector)
        .next()
        .and_then(|link| link.attr("src").map(|s| s.to_string()))
        .filter(|link| link.len() <= 1000);

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
        rss_title: Some(
            item.title()
                .filter(|title| title.len() <= 200)
                .unwrap_or_default()
                .to_string(),
        ),
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
) -> Result<i32, OmniNewsError> {
    let item_link = rss_item.rss_link.clone().unwrap_or_default();

    if let Some(str) = rss_item.rss_description.as_mut() {
        *str = str.chars().take(200).collect()
    };

    match rss_item_repository::select_item_by_link(pool, item_link).await {
        Ok(item) => Ok(set_item_rank(pool, item).await?),
        Err(_) => rss_item_repository::insert_rss_item(pool, rss_item)
            .await
            .map_err(|e| {
                error!("[Service] Failed to select item by link : {}", e);
                OmniNewsError::Database(e)
            }),
    }
}

async fn set_item_rank(pool: &State<MySqlPool>, mut item: RssItem) -> Result<i32, OmniNewsError> {
    item.rss_rank = item.rss_rank.map(|r| r + 1);

    rss_item_repository::update_rss_item(pool, item)
        .await
        .map_err(|e| {
            error!("[Service] Failed to set item rank: {}", e);
            OmniNewsError::Database(e)
        })
}

pub async fn get_rss_list(
    pool: &State<MySqlPool>,
    search_value: SearchRequest,
) -> Result<Vec<RssItem>, OmniNewsError> {
    let search_morphemes =
        analyze_morpheme(search_value.search_value.unwrap()).map_err(OmniNewsError::Morpheme)?;

    let morphemes_sources =
        morpheme_service::get_morphemes_sources_by_search_value(pool, search_morphemes).await?;

    let mut result: Vec<RssItem> = Vec::new();
    for morpheme_source in morphemes_sources {
        if let Some(morpheme_id) = morpheme_source.morpheme_id {
            result = match search_value.search_type.clone().unwrap() {
                SearchType::Accuracy => {
                    rss_item_repository::select_rss_items_by_morpheme_id_order_by_source_rank(
                        pool,
                        morpheme_id,
                    )
                    .await
                    .map_err(|e| {
                        error!(
                            "[Service] Faild to select items by morpheme id order by source rank: {}",
                            e
                        );
                        OmniNewsError::Database(e)
                    })?
                }
                SearchType::Popularity => {
                    rss_item_repository::select_rss_items_by_morpheme_id_order_by_rss_rank(
                        pool,
                        morpheme_id,
                    )
                    .await
                    .map_err(|e| {
                        error!(
                            "[Service] Failed to select items by morpheme id order by rss rank: {}",
                            e
                        );
                        OmniNewsError::Database(e)
                    })?
                }
                SearchType::Latest => {
                    rss_item_repository::select_rss_items_by_morpheme_id_order_by_pub_date(
                        pool,
                        morpheme_id,
                    )
                    .await
                    .map_err(|e| {
                        error!(
                            "[Service] Failed to select items by morpheme id order by pub date: {}",
                            e
                        );
                        OmniNewsError::Database(e)
                    })?
                }
            };
        }
    }

    Ok(result)
}

// 상위 100개 중 50개 랜덤 반환
pub async fn get_recommend_item(pool: &State<MySqlPool>) -> Result<Vec<RssItem>, OmniNewsError> {
    match rss_item_repository::select_rss_items_order_by_rss_rank(pool).await {
        Ok(mut res) => {
            let mut rng = thread_rng();
            res.shuffle(&mut rng);
            Ok(res.into_iter().take(50).collect())
        }
        Err(e) => {
            error!(
                "[Service] Failed to select items order by rss rank: {:?}",
                e
            );
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn get_rss_item_by_channel_title(
    pool: &State<MySqlPool>,
    channel_title: String,
) -> Result<Vec<RssItem>, OmniNewsError> {
    rss_item_repository::select_rss_items_by_channel_title(pool, channel_title)
        .await
        .map_err(|e| {
            error!("[Service] Failed to select items by channel title: {:?}", e);
            OmniNewsError::Database(e)
        })
}
