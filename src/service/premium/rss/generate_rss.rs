use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Url;
use rss::{Channel, ChannelBuilder, Image, ItemBuilder};
use sqlx::MySqlPool;
use thirtyfour::By;

use crate::{
    config::webdriver::{AcquireStrategy, DriverPool},
    dto::{
        premium::rss::{
            request::{RssGenerateByCssReqeustDto, RssGenerateRequestDto},
            response::RssGenerateResponseDto,
        },
        rss::response::RssChannelResponseDto,
    },
    model::{error::OmniNewsError, premium::rss_generate::SiteType, rss::ChannelCssElement},
    service::{channel_css_service, channel_service, item_service},
    utils::embedding_util::EmbeddingService,
};

use super::site::{default, instagram, medium, naver, tistory};

pub async fn generate(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    driver_pool: &DriverPool,
    data: RssGenerateRequestDto,
) -> Result<RssGenerateResponseDto, OmniNewsError> {
    //let generated_channel = generate_rss_channel(data.channel_link, data.kind).await?;
    let link = data.channel_link;

    if let Ok(channel) = channel_service::find_rss_channel_by_channel_link(pool, &link).await {
        return Ok(RssGenerateResponseDto {
            is_exist: true,
            channel: RssChannelResponseDto::from_model(channel),
        });
    };

    let channel_id = match data.kind {
        SiteType::Naver => naver::generate_rss(pool, embedding_service, &link).await?,
        SiteType::Tistory => tistory::generate_rss(pool, embedding_service, &link).await?,
        //SiteType::Instagram => todo!(),
        SiteType::Medium => medium::generate_rss(pool, embedding_service, &link).await?,
        SiteType::Instagram => {
            instagram::generate_rss(pool, embedding_service, driver_pool, &link).await?
        }
        SiteType::Default => {
            default::generate_rss(pool, embedding_service, driver_pool, &link).await?
        }
    };

    let channel = channel_service::find_rss_channel_by_id(pool, channel_id).await?;
    Ok(RssGenerateResponseDto {
        is_exist: false,
        channel,
    })
}

pub async fn rss_by_css(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    driver_pool: &DriverPool,
    data: RssGenerateByCssReqeustDto,
) -> Result<RssGenerateResponseDto, OmniNewsError> {
    // validate already exist channel
    if let Ok(channel) =
        channel_service::find_rss_channel_by_channel_link(pool, &data.channel_link).await
    {
        return Ok(RssGenerateResponseDto {
            is_exist: true,
            channel: RssChannelResponseDto::from_model(channel),
        });
    }

    let strategy = AcquireStrategy::Wait(Some(Duration::from_secs(10)));
    let driver_handle = driver_pool
        .acquire(strategy)
        .await
        .map_err(OmniNewsError::WebDriverPool)?;
    let driver = driver_handle.driver();

    driver
        .goto(&data.channel_link)
        .await
        .map_err(OmniNewsError::WebDriverError)?;

    let (channel_id, mut rss_channel) = make_channel(pool, embedding_service, &data).await?;
    let items = make_items(&data, driver).await?;

    rss_channel.set_items(items.0);
    let _ = item_service::create_rss_items_and_embedding(
        pool,
        embedding_service,
        rss_channel,
        Some(items.1),
        channel_id,
    )
    .await
    .map_err(|e| {
        error!(
            "[Service-Rss_By_Css] Failed to create rss item and embedding: {:?}",
            e
        );
        e
    });

    let channel_css_el = ChannelCssElement {
        channel_id: Some(channel_id),
        item_title_css: Some(data.item_title_css),
        item_description_css: Some(data.item_description_css),
        item_link_css: Some(data.item_link_css),
        item_author_css: Some(data.item_author_css),
        item_pub_date_css: Some(data.item_pub_date_css),
        item_image_css: Some(data.item_image_css),
    };

    // store item css elements
    let _ = channel_css_service::store_channel_css_service(pool, channel_css_el).await?;

    let channel = channel_service::find_rss_channel_by_id(pool, channel_id).await?;

    Ok(RssGenerateResponseDto {
        is_exist: false,
        channel,
    })
}

async fn make_channel(
    pool: &sqlx::Pool<sqlx::MySql>,
    embedding_service: &EmbeddingService,
    data: &RssGenerateByCssReqeustDto,
) -> Result<(i32, Channel), OmniNewsError> {
    let channel_image_link = &data.channel_image_link;
    let channel_title = &data.channel_title;
    let channel_description = &data.channel_description;
    let channel_language = &data.channel_language;
    let mut image = Image::default();
    image.set_url(channel_image_link);
    let rss_channel = ChannelBuilder::default()
        .title(channel_title)
        .link(&data.channel_link)
        .description(channel_description)
        .language(channel_language.to_string())
        .image(image)
        // Css요소는 스케쥴 시 추가 관리 필요함
        .generator("Omninews_css".to_string())
        .build();
    let channel = channel_service::make_rss_channel(
        &rss_channel,
        format!("Generated by Omninews, {channel_title}"),
        true,
    );
    Ok((
        channel_service::store_channel_and_embedding(pool, embedding_service, channel).await?,
        rss_channel,
    ))
}

async fn make_items(
    data: &RssGenerateByCssReqeustDto,
    driver: &thirtyfour::WebDriver,
) -> Result<(Vec<rss::Item>, Vec<String>), OmniNewsError> {
    let mut items = (Vec::new(), Vec::new());

    let base_url = Url::parse(&data.channel_link).unwrap();

    let item_titles = driver.find_all(By::Css(&data.item_title_css)).await?;
    let item_descriptions = driver.find_all(By::Css(&data.item_description_css)).await?;
    let item_links = driver.find_all(By::Css(&data.item_link_css)).await?;
    let item_authors = (driver.find_all(By::Css(&data.item_author_css)).await).ok();
    let item_pub_date_raws = (driver.find_all(By::Css(&data.item_pub_date_css)).await).ok();
    let item_images = (driver.find_all(By::Css(&data.item_image_css)).await).ok();

    let len = *[item_titles.len(), item_descriptions.len(), item_links.len()]
        .iter()
        .min()
        .unwrap_or(&0);

    for idx in 0..len {
        let item_title = item_titles
            .get(idx)
            .unwrap()
            .text()
            .await
            .unwrap_or_default();

        let item_description = item_descriptions
            .get(idx)
            .unwrap()
            .text()
            .await
            .unwrap_or_default();

        let raw_link = item_links
            .get(idx)
            .unwrap()
            .attr("href")
            .await?
            .unwrap_or_default();

        let item_link = match Url::parse(&raw_link) {
            Ok(u) => u.to_string(),
            Err(_) => base_url
                .join(&raw_link)
                .map(|u| u.to_string())
                .unwrap_or(raw_link.clone()),
        };

        let item_image_link = if let Some(images) = &item_images {
            images
                .get(idx)
                .unwrap()
                .attr("src")
                .await?
                .and_then(|raw_src| match Url::parse(&raw_src) {
                    Ok(u) => Some(u.to_string()),
                    Err(_) => base_url.join(&raw_src).map(|u| u.to_string()).ok(),
                })
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        // --- author ---
        let item_author = if let Some(authors) = &item_authors {
            authors.get(idx).unwrap().text().await.unwrap_or_default()
        } else {
            "".to_string()
        };

        // --- pub_date ---
        let item_pub_date_rfc2822 = if let Some(pub_dates) = &item_pub_date_raws {
            let item_pub_date_raw = pub_dates.get(idx).unwrap().text().await.unwrap_or_default();

            match DateTime::parse_from_rfc3339(&item_pub_date_raw) {
                Ok(dt) => dt.to_rfc2822(),
                Err(_) => {
                    if let Ok(dt2) = DateTime::parse_from_rfc3339(&(item_pub_date_raw + "Z")) {
                        dt2.to_rfc2822()
                    } else {
                        Utc::now().to_rfc2822()
                    }
                }
            }
        } else {
            Utc::now().to_rfc2822()
        };

        let item = ItemBuilder::default()
            .title(item_title)
            .description(item_description)
            .link(item_link)
            .author(item_author)
            .pub_date(item_pub_date_rfc2822)
            .build();
        items.0.push(item);
        items.1.push(item_image_link);
    }
    Ok(items)
}
