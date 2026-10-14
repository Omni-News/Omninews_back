use chrono::{FixedOffset, Utc};
use feed_rs::parser::parse;
use reqwest::Url;
use rss::{Channel, ChannelBuilder, Image, Item, ItemBuilder};
use serde_json::Value;
use sqlx::MySqlPool;
use thirtyfour::WebDriver;

use crate::{
    dto::{
        rss::{request::CreateRssRequestDto, response::RssChannelResponseDto},
        search::{request::SearchRequestDto, response::SearchResponseDto},
    },
    model::{
        error::OmniNewsError,
        rss::{NewRssChannel, RssChannel},
        search::SearchType,
    },
    repository::rss_channel_repository,
    rss_error, rss_info, rss_warn,
};

use super::item_service;

pub async fn create_rss_all(
    pool: &MySqlPool,
    links: Vec<CreateRssRequestDto>,
) -> Result<bool, OmniNewsError> {
    for link in links {
        rss_info!("[Service] Add : {}", link.rss_link);
        let _ = create_rss_and_embedding(pool, link.rss_link)
            .await
            .unwrap_or_default();
    }
    Ok(true)
}

pub async fn create_rss_and_embedding(
    pool: &MySqlPool,
    link: String,
) -> Result<i32, OmniNewsError> {
    let rss_channel = parse_rss_link_to_channel(&link).await?;
    if &rss_channel.title == "Not Found" || rss_channel.title.is_empty() {
        error!(
            "[Service] Failed to parse RSS link: {}, title is empty or not found",
            link
        );
        return Err(OmniNewsError::NotFound(
            "Failed to parse RSS link".to_string(),
        ));
    }

    create_rss_and_embedding_by_channel(pool, rss_channel, link, false).await
}

pub async fn create_rss_and_embedding_with_web_driver(
    pool: &MySqlPool,
    link: String,
    driver: &WebDriver,
) -> Result<i32, OmniNewsError> {
    let rss_channel = parse_rss_link_to_channel_with_web_driver(&link, driver).await?;

    if &rss_channel.title == "Not Found" || rss_channel.title.is_empty() {
        error!(
            "[Service] Failed to parse RSS link: {}, title is empty or not found",
            link
        );
        return Err(OmniNewsError::NotFound(
            "Failed to parse RSS link".to_string(),
        ));
    }
    create_rss_and_embedding_by_channel(pool, rss_channel, link, true).await
}

pub async fn create_rss_and_embedding_by_channel(
    pool: &MySqlPool,
    rss_channel: Channel,
    rss_link: String,
    is_generated_channel: bool,
) -> Result<i32, OmniNewsError> {
    let channel = make_rss_channel(&rss_channel, rss_link, is_generated_channel);
    let channel_id = store_channel_and_embedding(pool, channel).await?;

    let _ = item_service::create_rss_items_and_embedding(pool, rss_channel, None, channel_id)
        .await
        .map_err(|e| {
            rss_error!("[Service] Failed to create rss item and embedding: {:?}", e);
            e
        });

    Ok(channel_id)
}

pub async fn parse_rss_link_to_channel(link: &str) -> Result<Channel, OmniNewsError> {
    let response = reqwest::get(link).await.map_err(|e| {
        rss_error!("[Service] Not found url : {}", link);
        OmniNewsError::Request(e)
    })?;

    let body = response.text().await.map_err(OmniNewsError::Request)?;
    let channel = Channel::read_from(body.as_bytes()).unwrap_or({
        // atom이나 더 범용적인 rss포맷임. 이럴 때 feed-rs사용
        parse_with_feed_rs(body)?
    });

    Ok(channel)
}

/// feed-rs를 사용해 atom등의 rss데이터를 뽑아 Channel로 build후 반환함.
fn parse_with_feed_rs(body: String) -> Result<Channel, OmniNewsError> {
    let data = parse(body.as_bytes());
    Ok(match &data {
        Ok(data) => {
            let fe_title = match data.title.clone() {
                Some(data) => data.content,
                None => "None".to_string(),
            };

            let fe_link = match data.links.first() {
                Some(data) => data.href.clone(),
                None => "None".to_string(),
            };
            let fe_description = match data.description.clone() {
                Some(data) => data.content,
                None => "None".to_string(),
            };
            let fe_language = data.language.clone().unwrap_or("None".to_string());
            let fe_generator = match data.generator.clone() {
                Some(data) => data.content,
                None => "None".to_string(),
            };
            let fe_image_link = match data.logo.clone() {
                Some(data) => data.uri,
                None => "None".to_string(),
            };

            let entries = &data.entries;
            let items = entries
                .iter()
                .map(|entry| {
                    let en_title = match entry.title.clone() {
                        Some(data) => data.content,
                        None => "None".to_string(),
                    };
                    let en_description = match entry.content.clone() {
                        Some(data) => data.body.unwrap_or_default(),
                        None => "None".to_string(),
                    };
                    let en_link = match entry.links.first() {
                        Some(data) => data.href.clone(),
                        None => "None".to_string(),
                    };
                    let en_author = match entry.authors.first() {
                        Some(data) => data.name.clone(),
                        None => "None".to_string(),
                    };
                    let en_pub_date = entry
                        .published
                        .unwrap_or(Utc::now())
                        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
                        .to_rfc2822();

                    ItemBuilder::default()
                        .title(en_title)
                        .description(en_description)
                        .link(en_link)
                        .author(en_author)
                        .pub_date(en_pub_date)
                        .build()
                })
                .collect::<Vec<Item>>();

            let mut image = Image::default();
            image.set_url(fe_image_link);

            ChannelBuilder::default()
                .title(fe_title)
                .link(fe_link)
                .description(fe_description)
                .language(fe_language)
                .image(image)
                .generator(fe_generator)
                .items(items)
                .build()
        }

        Err(_) => {
            rss_error!("[Fervice] Failed to parse feed-rs data: {:?}", data.err());
            return Err(OmniNewsError::ParseRssChannel);
        }
    })
}

pub async fn parse_rss_link_to_channel_with_web_driver(
    link: &str,
    driver: &WebDriver,
) -> Result<Channel, OmniNewsError> {
    if let Ok(u) = Url::parse(link) {
        let origin = format!("{}://{}/", u.scheme(), u.host_str().unwrap_or_default());
        let _ = driver.goto(&origin).await;
    }
    // async script로 fetch → text 본문 받기
    // file download이므로, 본문을 text로 변환하여 반환
    let js = r#"
                const url = arguments[0];
                const done = arguments[arguments.length - 1];
                fetch(url, {
                    method: 'GET',
                    headers: {
                    'Accept': 'application/rss+xml, application/atom+xml, application/xml;q=0.9, text/xml;q=0.8, */*;q=0.1',
                    'Cache-Control': 'no-cache',
                    },
                    credentials: 'include'
                }).then(async (r) => {
                    const body = await r.text();
                    done({
                        ok: r.ok,
                        status: r.status,
                        contentType: r.headers.get('content-type'),
                        body
                    });
                }).catch(e => done({ ok: false, status: 0, contentType: null, body: String(e) }));
            "#;

    let ret = driver
        .execute_async(js, vec![Value::String(link.to_string())])
        .await?;

    let obj = ret.json().as_object().unwrap();
    let ok = obj.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let status = obj.get("status").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let ctype = obj
        .get("contentType")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    let body = obj
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or("no body")
        .unwrap()
        .to_string();

    if !ok || status >= 400 {
        error!(
            "[Service] Failed to fetch rss link: {}, status: {}, content-type: {}, body: {}",
            link, status, ctype, body
        );
        return Err(OmniNewsError::WebDriverNotFound);
    }
    if ctype.contains("text/html") && body.contains("Attention Required") {
        error!(
            "[Service] WebDriver blocked by Cloudflare or similar service for link: {}",
            link
        );
        return Err(OmniNewsError::WebDriverNotFound);
    }
    let channel = Channel::read_from(body.as_bytes()).unwrap_or(
        // atom이나 더 범용적인 rss포맷임. 이럴 때 feed-rs사용
        parse_with_feed_rs(body)?,
    );

    Ok(channel)
}

pub fn make_rss_channel(
    channel: &Channel,
    rss_link: String,
    is_generated_channel: bool,
) -> NewRssChannel {
    NewRssChannel::new(
        channel.title().to_string(),
        channel.link().to_string(),
        channel.description().to_string(),
        channel.image().map(|e| e.url().to_string()),
        channel.language().unwrap_or("None").to_string(),
        channel
            .generator()
            .unwrap_or(if is_generated_channel {
                "Omninews_default"
            } else {
                "None"
            })
            .to_string(),
        0,
        rss_link,
    )
}

pub async fn store_channel_and_embedding(
    pool: &MySqlPool,
    rss_channel: NewRssChannel,
) -> Result<i32, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_rss_link(
        pool,
        rss_channel.channel_link.clone().unwrap_or_default(),
    )
    .await
    {
        Ok(_) => {
            rss_warn!(
                "[Service] Already exist channel: {}",
                rss_channel.channel_link.clone().unwrap()
            );
            Err(OmniNewsError::AlreadyExists("Channel".into()))
        }
        Err(_) => store_rss_channel(pool, rss_channel).await,
    }
}

async fn store_rss_channel(pool: &MySqlPool, channel: NewRssChannel) -> Result<i32, OmniNewsError> {
    let channel_link = channel.channel_link.clone().unwrap_or_default();

    match rss_channel_repository::select_rss_channel_by_rss_link(pool, channel_link).await {
        Ok(channel) => Ok(channel.channel_id.unwrap()),
        Err(_) => Ok(rss_channel_repository::insert_rss_channel(pool, channel)
            .await
            .map_err(|e| {
                rss_error!("[Service] Failed to insert rss channel: {:?}", e);
                OmniNewsError::Database(e)
            })?),
    }
}

pub async fn find_rss_channel_by_id(
    pool: &MySqlPool,
    channel_id: i32,
) -> Result<RssChannelResponseDto, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_id(pool, channel_id).await {
        Ok(res) => Ok(RssChannelResponseDto::from_model(res)),
        Err(e) => {
            rss_error!("[Service] Failed to select rss channel by id: {:?}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn find_rss_channel_by_rss_link(
    pool: &MySqlPool,
    channel_rss_link: String,
) -> Result<RssChannel, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_rss_link(pool, channel_rss_link).await {
        Ok(res) => Ok(res),
        Err(e) => {
            rss_warn!(
                "[Service] Failed to select rss channel by rss link: {:?}",
                e
            );
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn find_rss_channel_by_channel_link(
    pool: &MySqlPool,
    channel_link: &str,
) -> Result<RssChannel, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_channel_link(pool, channel_link).await {
        Ok(res) => Ok(res),
        Err(e) => Err(OmniNewsError::Database(e)),
    }
}

pub async fn get_channel_list(
    pool: &MySqlPool,
    value: SearchRequestDto,
) -> Result<SearchResponseDto, OmniNewsError> {
    let search_value = value.search_value.clone().unwrap_or_default();
    let page = value.search_page_size.unwrap_or_default();

    let mut searched_rss_channels =
        rss_channel_repository::select_rss_channel_by_embedding_id(pool, search_value.as_str(), 0)
            .await?;
    let total = searched_rss_channels.len() as i32;
    // Provide 20 rss item each select request
    let offset = (page - 1) * 20;

    let has_next = total > offset + 20;

    // too long page size
    if offset > total {
        return Ok(SearchResponseDto::new(vec![], vec![], total, page, false));
    }

    match value.search_type.clone().unwrap() {
        SearchType::Popularity => {
            searched_rss_channels.sort_by(|a, b| {
                b.channel_rank
                    .unwrap_or_default()
                    .cmp(&a.channel_rank.unwrap_or_default())
            });
        }
        // 스키마에 날짜 컬럼 없어 정확순으로 대체
        SearchType::Accuracy | SearchType::Latest => {}
    };

    Ok(SearchResponseDto::new(
        RssChannelResponseDto::from_model_list(searched_rss_channels),
        vec![],
        total,
        page,
        has_next,
    ))
}

// TODO 랭크 50순위 채널에서 20개 랜덤 반환
pub async fn get_recommend_channel(
    pool: &MySqlPool,
) -> Result<Vec<RssChannelResponseDto>, OmniNewsError> {
    match rss_channel_repository::select_rss_channels_order_by_channel_rank(pool).await {
        Ok(res) => {
            //            let mut rng = rng();
            //            res.shuffle(&mut rng);
            //            Ok(res.into_iter().take(20).collect())
            Ok(RssChannelResponseDto::from_model_list(res))
        }
        Err(e) => {
            rss_error!(
                "[Service] Failed to select channels order by channel rank: {:?}",
                e
            );
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn get_rss_preview(
    pool: &MySqlPool,
    rss_link: String,
) -> Result<RssChannelResponseDto, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_rss_link(pool, rss_link.clone()).await {
        Ok(res) => Ok(RssChannelResponseDto::from_model(res)),
        Err(_) => {
            let rss_channel = parse_rss_link_to_channel(&rss_link).await?;
            let new_channel = make_rss_channel(&rss_channel, rss_link.clone(), false);
            let channel = RssChannel::new(new_channel);
            Ok(RssChannelResponseDto::from_model(channel))
        }
    }
}

pub async fn is_channel_exist_by_link(
    pool: &MySqlPool,
    channel_link: String,
) -> Result<bool, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_rss_link(pool, channel_link).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub async fn is_channel_exist_by_id(
    pool: &MySqlPool,
    channel_id: i32,
) -> Result<bool, OmniNewsError> {
    match rss_channel_repository::select_rss_channel_by_id(pool, channel_id).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub async fn update_rss_channel_rank(
    pool: &MySqlPool,
    channel_id: i32,
    num: i32,
) -> Result<bool, OmniNewsError> {
    match rss_channel_repository::update_rss_channel_rank_by_id(pool, channel_id, num).await {
        Ok(res) => Ok(res),
        Err(e) => {
            rss_error!("[Service] Failed to update rss channel rank: {:?}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
