use crate::{
    dto::{
        rss::{request::UpdateRssRankRequestDto, response::RssItemResponseDto},
        search::{request::SearchRequestDto, response::SearchResponseDto},
    },
    model::{
        embedding::NewEmbedding,
        error::OmniNewsError,
        rss::{NewRssItem, RssItem},
        search::SearchType,
    },
    repository::rss_item_repository,
    rss_error, rss_warn,
    service::embedding_service,
    utils::{annoy_util::load_rss_annoy, embedding_util::EmbeddingService},
};
use chrono::{DateTime, NaiveDateTime};
use rss::{Channel, Item};
use scraper::{Html, Selector};
use sqlx::MySqlPool;

pub async fn create_rss_items_and_embedding(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    mut channel: Channel,
    item_image_links: Option<Vec<String>>,
    channel_id: i32,
) -> Result<(), OmniNewsError> {
    let channel_image_url = channel
        .image()
        .map_or(String::new(), |image| image.url().to_string()); // TODO rss cateogory 미구현 상태.

    for (i, rss_item) in channel.items_mut().iter_mut().enumerate() {
        create_rss_item_and_embedding(
            pool,
            embedding_service,
            channel_id,
            channel_image_url.clone(),
            if let Some(links) = &item_image_links {
                links.get(i).cloned()
            } else {
                None
            },
            rss_item,
        )
        .await?;
    }
    Ok(())
}
pub async fn create_rss_item_and_embedding(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    channel_id: i32,
    channel_image_url: String,
    item_image_link: Option<String>,
    rss_item: &mut Item,
) -> Result<bool, OmniNewsError> {
    let description = rss_item.description().unwrap_or("None");
    let extracted_description = extract_html_passage(description);

    let item_image_link = if let Some(link) = item_image_link {
        Some(link)
    } else {
        extracted_description.1
    };

    rss_item.set_description(extracted_description.0.clone());

    let item_image_link = use_channel_url_if_none(item_image_link, channel_image_url.clone());

    let item = match make_rss_item(channel_id, rss_item, item_image_link) {
        Ok(item) => item,
        Err(e) => {
            rss_error!("[Service] Failed to make rss item: {}", e);
            return Err(e);
        }
    };

    let item_id = store_rss_item(pool, item.clone()).await.unwrap();

    let sentence = format!(
        "{}\n{}\n{}",
        item.rss_title.unwrap_or_default(),
        extracted_description.0,
        item.rss_author.unwrap_or_default()
    );
    let embedding = NewEmbedding {
        embedding_value: None,
        channel_id: None,
        rss_id: Some(item_id),
        news_id: None,
        embedding_source_rank: Some(0),
    };

    let _ =
        embedding_service::create_embedding(pool, embedding_service, sentence, embedding).await?;
    Ok(true)
}

fn extract_html_passage(html: &str) -> (String, Option<String>) {
    let document = Html::parse_document(html);

    let passage_selector = Selector::parse("h3, p").unwrap();
    let image_selector = Selector::parse("img").unwrap();

    let extracted_texts: Vec<String> = document
        .clone()
        .select(&passage_selector)
        .map(|e| e.text().collect::<Vec<_>>().join(" "))
        .collect();

    let extracted_text = if extracted_texts.is_empty() {
        html.to_string()
    } else {
        extracted_texts.join(" ")
    };

    let extracted_text = extracted_text
        .chars()
        .take(200)
        .collect::<String>()
        .trim()
        .to_string();

    let image_link = document
        .select(&image_selector)
        .next()
        .and_then(|link| link.attr("src").map(|s| s.to_string()))
        .filter(|link| link.len() <= 1000);

    (extracted_text, image_link)
}

fn use_channel_url_if_none(link: Option<String>, channel_image_url: String) -> String {
    match link {
        Some(link) => link,
        None => channel_image_url,
    }
}

pub fn make_rss_item(
    channel_id: i32,
    item: &Item,
    item_image_link: String,
) -> Result<NewRssItem, OmniNewsError> {
    let rss_pub_date = parse_pub_date(item.pub_date());
    if item.title.is_none() || item.description.is_none() {
        return Err(OmniNewsError::NotFound(
            "RSS item must have a title and description".to_string(),
        ));
    }
    Ok(NewRssItem::new(
        channel_id,
        item,
        rss_pub_date,
        item_image_link,
    ))
}

fn parse_pub_date(pub_date_str: Option<&str>) -> Option<NaiveDateTime> {
    pub_date_str
        .and_then(|date_str| {
            // 1. RFC2822 형식 파싱 시도
            if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
                // 타임존이 KST/+09:00 인지 확인 (초 단위로 환산해서 비교)
                if dt.offset().local_minus_utc() == 9 * 3600 {
                    // 한국 시간이면 타임존 제거만 하고 값 유지
                    return Some(dt.naive_local());
                } else {
                    // 다른 타임존이면 기존처럼 UTC 변환
                    return Some(dt.naive_utc());
                }
            }

            None
        })
        .or_else(|| {
            // 파싱 실패하면 디폴트 시간
            Some(
                NaiveDateTime::parse_from_str("1970-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S")
                    .ok()
                    .unwrap(),
            )
        })
}

async fn store_rss_item(pool: &MySqlPool, mut rss_item: NewRssItem) -> Result<i32, OmniNewsError> {
    let item_link = rss_item.rss_link.clone().unwrap_or_default();

    if let Some(str) = rss_item.rss_description.as_mut() {
        *str = str.chars().take(200).collect()
    };

    match rss_item_repository::select_item_by_link(pool, item_link).await {
        Ok(item) => {
            rss_warn!(
                "[Service] Item already exists with link: {}",
                item.rss_link.clone().unwrap_or_default()
            );
            rss_item_repository::insert_rss_item(pool, rss_item)
                .await
                .map_err(|e| {
                    rss_error!("[Service] Failed to select item by link : {}", e);
                    OmniNewsError::Database(e)
                })
        }

        Err(_) => rss_item_repository::insert_rss_item(pool, rss_item)
            .await
            .map_err(|e| {
                rss_error!("[Service] Failed to select item by link : {}", e);
                OmniNewsError::Database(e)
            }),
    }
}

pub async fn get_rss_list(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    value: SearchRequestDto,
) -> Result<SearchResponseDto, OmniNewsError> {
    let search_value = value.search_value.unwrap_or_default();
    let load_annoy = load_rss_annoy(embedding_service, search_value.clone()).await?;
    let page = value.search_page_size.unwrap_or_default();

    let mut _item_list: Vec<RssItem> = vec![];
    let _total = load_annoy.0.len() as i32;

    let mut searched_rss_items =
        rss_item_repository::select_rss_item_by_embedding_id(pool, search_value.as_str(), 0)
            .await?;
    let total = searched_rss_items.len() as i32;
    // Provide 20 rss item each select request
    let offset = (page - 1) * 20;
    let has_next = total > offset + 20;

    // too long page size
    if offset > total {
        return Ok(SearchResponseDto::new(vec![], vec![], total, page, false));
    }

    match value.search_type.clone().unwrap() {
        SearchType::Accuracy => {
            //            push_rss_item(
            //                pool,
            //                &load_annoy,
            //                &mut item_list,
            //                total,
            //                offset,
            //                search_value.as_str(),
            //            )
            //            .await;
        }
        SearchType::Popularity => {
            //            push_rss_item(
            //                pool,
            //                &load_annoy,
            //                &mut item_list,
            //                total,
            //                offset,
            //                search_value.as_str(),
            //            )
            //            .await;

            searched_rss_items.sort_by(|a, b| {
                b.rss_rank
                    .unwrap_or_default()
                    .cmp(&a.rss_rank.unwrap_or_default())
            });
        }
        SearchType::Latest => {
            //            push_rss_item(
            //                pool,
            //                &load_annoy,
            //                &mut item_list,
            //                total,
            //                offset,
            //                search_value.as_str(),
            //            )
            //            .await;

            searched_rss_items.sort_by(|a, b| {
                b.rss_pub_date
                    .unwrap_or_default()
                    .cmp(&a.rss_pub_date.unwrap_or_default())
            });
        }
    };
    Ok(SearchResponseDto::new(
        vec![],
        RssItemResponseDto::from_model_list(searched_rss_items),
        total,
        page,
        has_next,
    ))
}

#[allow(unused)]
async fn push_rss_item(
    pool: &MySqlPool,
    load_annoy: &(Vec<i32>, Vec<f32>),
    item_list: &mut [RssItem],
    total: i32,
    offset: i32,
    search_value: &str,
) {
    for i in 0..20 {
        if offset + i == total {
            break;
        }

        if let Ok(item) = rss_item_repository::select_rss_item_by_embedding_id(
            pool,
            search_value,
            load_annoy.0[(i + offset) as usize],
        )
        .await
        {
            //item_list.push(item);
        }
    }
}

// TODO 상위 100개 중 50개 랜덤 반환
pub async fn get_recommend_item(
    pool: &MySqlPool,
) -> Result<Vec<RssItemResponseDto>, OmniNewsError> {
    match rss_item_repository::select_rss_items_order_by_rss_rank(pool).await {
        Ok(res) => {
            //            let mut rng = rng();
            //            res.shuffle(&mut rng);
            //            Ok(res.into_iter().take(50).collect())
            Ok(RssItemResponseDto::from_model_list(res))
        }
        Err(e) => {
            rss_error!(
                "[Service] Failed to select items order by rss rank: {:?}",
                e
            );
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn get_rss_item_by_channel_id(
    pool: &MySqlPool,
    channel_id: i32,
) -> Result<Vec<RssItemResponseDto>, OmniNewsError> {
    match rss_item_repository::select_rss_items_by_channel_id(pool, channel_id).await {
        Ok(res) => Ok(RssItemResponseDto::from_model_list(res)),
        Err(e) => {
            rss_error!("[Service] Failed to select items by channel id: {:?}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn update_rss_item_rank(
    pool: &MySqlPool,
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
