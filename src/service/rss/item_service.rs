use crate::{
    dto::{
        rss::{request::UpdateRssRankRequestDto, response::RssItemResponseDto},
        search::request::SearchRequestDto,
    },
    model::{embedding::NewEmbedding, error::OmniNewsError, rss::NewRssItem, search::SearchType},
    repository::rss_item_repository,
    service::embedding_service,
    utils::{annoy_util::load_rss_annoy, embedding_util::EmbeddingService},
};
use chrono::{DateTime, NaiveDateTime};
use rocket::State;
use rss::{Channel, Item};
use scraper::{Html, Selector};
use sqlx::MySqlPool;

pub async fn crate_rss_item_and_embedding(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
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
        rss_item.set_description(extracted_description.clone());
        let item_image_link = use_channel_url_if_none(item_image_link, channel_image_url.clone());

        let item = make_rss_item(channel_id, rss_item, item_image_link);
        let item_id = store_rss_item(pool, item.clone()).await.unwrap();

        let sentence = format!(
            "{}\n{}",
            item.rss_title.clone().unwrap_or_default(),
            extracted_description
        );
        let embedding = NewEmbedding {
            embedding_value: None,
            channel_id: None,
            rss_id: Some(item_id),
            news_id: None,
            embedding_source_rank: Some(0),
        };

        let _ = embedding_service::create_embedding(pool, embedding_service, sentence, embedding)
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
    let rss_pub_date = parse_pub_date(item.pub_date());
    NewRssItem::new(channel_id, item, rss_pub_date, item_image_link)
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

async fn store_rss_item(
    pool: &State<MySqlPool>,
    mut rss_item: NewRssItem,
) -> Result<i32, OmniNewsError> {
    let item_link = rss_item.rss_link.clone().unwrap_or_default();

    if let Some(str) = rss_item.rss_description.as_mut() {
        *str = str.chars().take(200).collect()
    };

    match rss_item_repository::select_item_by_link(pool, item_link).await {
        Ok(item) => {
            warn!(
                "[Service] Item already exists with link: {}",
                item.rss_link.clone().unwrap_or_default()
            );
            Err(OmniNewsError::AlreadyExists)
        }

        Err(_) => rss_item_repository::insert_rss_item(pool, rss_item)
            .await
            .map_err(|e| {
                error!("[Service] Failed to select item by link : {}", e);
                OmniNewsError::Database(e)
            }),
    }
}

pub async fn get_rss_list(
    pool: &State<MySqlPool>,
    embedding_service: &State<EmbeddingService>,
    value: SearchRequestDto,
) -> Result<Vec<RssItemResponseDto>, OmniNewsError> {
    let load_annoy = load_rss_annoy(embedding_service, value.search_value.unwrap()).await?;

    let result = match value.search_type.clone().unwrap() {
        SearchType::Accuracy => {
            let mut res = Vec::new();
            for id in load_annoy.0.iter() {
                if let Ok(item) =
                    rss_item_repository::select_rss_item_by_embedding_id(pool, *id).await
                {
                    res.push(item);
                }
            }
            res
        }
        SearchType::Popularity => {
            let mut res = Vec::new();
            for id in load_annoy.0.iter() {
                if let Ok(item) =
                    rss_item_repository::select_rss_item_by_embedding_id(pool, *id).await
                {
                    res.push(item);
                }
            }
            res.sort_by(|a, b| {
                b.rss_rank
                    .unwrap_or_default()
                    .cmp(&a.rss_rank.unwrap_or_default())
            });

            res
        }
        SearchType::Latest => {
            let mut res = Vec::new();
            for id in load_annoy.0.iter() {
                if let Ok(item) =
                    rss_item_repository::select_rss_item_by_embedding_id(pool, *id).await
                {
                    res.push(item);
                }
            }
            res.sort_by(|a, b| {
                b.rss_pub_date
                    .unwrap_or_default()
                    .cmp(&a.rss_pub_date.unwrap_or_default())
            });

            res
        }
    };

    Ok(RssItemResponseDto::from_model_list(result))
}

// TODO 상위 100개 중 50개 랜덤 반환
pub async fn get_recommend_item(
    pool: &State<MySqlPool>,
) -> Result<Vec<RssItemResponseDto>, OmniNewsError> {
    match rss_item_repository::select_rss_items_order_by_rss_rank(pool).await {
        Ok(res) => {
            //            let mut rng = rng();
            //            res.shuffle(&mut rng);
            //            Ok(res.into_iter().take(50).collect())
            Ok(RssItemResponseDto::from_model_list(res))
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

pub async fn get_rss_item_by_channel_id(
    pool: &State<MySqlPool>,
    channel_id: i32,
) -> Result<Vec<RssItemResponseDto>, OmniNewsError> {
    match rss_item_repository::select_rss_items_by_channel_id(pool, channel_id).await {
        Ok(res) => Ok(RssItemResponseDto::from_model_list(res)),
        Err(e) => {
            error!("[Service] Failed to select items by channel id: {:?}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn update_rss_item_rank(
    pool: &State<MySqlPool>,
    update_rss_rank: UpdateRssRankRequestDto,
) -> Result<bool, OmniNewsError> {
    match rss_item_repository::update_rss_channel_rank_by_id(
        pool,
        update_rss_rank.rss_id,
        update_rss_rank.num,
    )
    .await
    {
        Ok(res) => Ok(res),
        Err(e) => Err(OmniNewsError::Database(e)),
    }
}
