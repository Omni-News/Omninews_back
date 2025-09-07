use serde_json::Value;
use std::{env, time::Duration};

use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use rss::{Channel, ChannelBuilder, Image, Item, ItemBuilder};
use sqlx::MySqlPool;
use thirtyfour::{error::WebDriverError, By, WebDriver};
use tokio::time::sleep;

use crate::{
    config::webdriver::{AcquireStrategy, DriverPool},
    model::error::OmniNewsError,
    service::{
        channel_service::{self},
        item_service::{self},
    },
    utils::embedding_util::EmbeddingService,
};

/// 인스타그램 프로필로부터 RSS 채널 생성 (게시물 12개 수집)
pub async fn generate_rss(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    driver_pool: &DriverPool,
    link: &str,
) -> Result<i32, OmniNewsError> {
    // acquire driver
    let strategy = AcquireStrategy::Wait(Some(Duration::from_secs(10)));
    let driver_handle = driver_pool.acquire(strategy).await.map_err(|e| {
        error!("[Service-instagram] Failed to acquire WebDriver: {:?}", e);
        OmniNewsError::WebDriverPool(e)
    })?;
    let driver = driver_handle.driver();

    let username = extract_username(link).ok_or_else(|| OmniNewsError::ExtractLinkError)?;
    let link = &format!("http://instagram.com/{username}");
    let feeds_graphql_url = format!(
        r#"
            http://www.instagram.com/graphql/query?variables={{"data":{{"count":12,"include_relationship_info":false,"latest_besties_reel_media":false,"latest_reel_media":true}},"username":"{username}","__relay_internal__pv__PolarisFeedShareMenurelayprovider":false}}&doc_id=7898261790222653&server_timestamps=true
        "#
    );

    let _ = driver
        .goto(feeds_graphql_url.clone())
        .await
        .map_err(map_wd_err);
    let is_sign_in = is_sign_in_by_graphql(driver).await?;

    let mut channel_id = -1;
    info!("is_sign_in: {}", is_sign_in);
    if is_sign_in {
        channel_id = generate_channel_and_items(
            pool,
            embedding_service,
            driver,
            link,
            username,
            feeds_graphql_url,
        )
        .await?;
    } else {
        // 로그인
        info!("[Instagram] Sign in...");
        let _ = driver
            .goto("http://www.instagram.com")
            .await
            .map_err(map_wd_err);
        if is_login_page(driver).await? {
            attempt_login(driver).await?;
            channel_id = generate_channel_and_items(
                pool,
                embedding_service,
                driver,
                link,
                username,
                feeds_graphql_url,
            )
            .await?;
        } else {
            // 혹시 로그인 유도 모달(닫기 버튼) 존재시 닫기 (한국어/영어 모두 대응)
            dismiss_close_overlay(driver).await.ok();
            generate_channel_and_items(
                pool,
                embedding_service,
                driver,
                link,
                username,
                feeds_graphql_url,
            )
            .await?;
        }
    }
    info!("channel id : {}", channel_id);
    Ok(channel_id)
}

async fn generate_channel_and_items(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    driver: &WebDriver,
    link: &str,
    username: String,
    feeds_graphql_url: String,
) -> Result<i32, OmniNewsError> {
    let channel = generate_rss_channel(pool, embedding_service, link, driver, username).await?;

    generate_rss_items(
        pool,
        embedding_service,
        driver,
        feeds_graphql_url,
        channel.clone(),
    )
    .await
}

async fn generate_rss_items(
    pool: &sqlx::Pool<sqlx::MySql>,
    embedding_service: &EmbeddingService,
    driver: &WebDriver,
    feeds_graphql_url: String,
    mut channel: (Channel, i32),
) -> Result<i32, OmniNewsError> {
    let mut items: (Vec<Item>, Vec<String>) = (Vec::new(), Vec::new());
    let _ = driver.goto(feeds_graphql_url).await.map_err(map_wd_err);
    let data = driver.find(By::Css("body")).await.map_err(map_wd_err);
    match data {
        Ok(res) => {
            let data_s = res.text().await.map_err(map_wd_err)?;
            let data_v: Value = serde_json::from_str(data_s.as_str()).unwrap();

            let items_json = data_v
                .get("data")
                .and_then(|v| v.get("xdt_api__v1__feed__user_timeline_graphql_connection"))
                .and_then(|v| v.get("edges"))
                .and_then(|v| v.as_array())
                .unwrap();

            for v in items_json {
                let raw_texts = v
                    .get("node")
                    .and_then(|v| v.get("caption"))
                    .and_then(|v| v.get("text"))
                    .unwrap_or_default()
                    .to_string();

                let texts = raw_texts.split("\\n").collect::<Vec<&str>>();

                let title = texts.first().unwrap_or(&"");
                let description = texts.join(" ");
                let feed_code = v
                    .get("node")
                    .and_then(|v| v.get("code"))
                    .and_then(|v| v.as_str());

                let link = if let Some(code) = feed_code {
                    format!("http://instagram.com/p/{code}")
                } else {
                    "".to_string()
                };

                let author = v
                    .get("node")
                    .and_then(|v| v.get("user"))
                    .and_then(|v| v.get("full_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let pub_date_timestamp = match v
                    .get("node")
                    .and_then(|v| v.get("caption"))
                    .and_then(|v| v.get("created_at"))
                {
                    Some(v) => {
                        // feed
                        v.as_i64()
                    }
                    None => {
                        // reel
                        v.get("node")
                            .and_then(|v| v.get("taken_at"))
                            .and_then(|v| v.as_i64())
                    }
                }
                .map(|v| Utc.timestamp_opt(v, 0))
                .unwrap()
                .unwrap();

                let kst = FixedOffset::east_opt(9 * 3600).unwrap();
                let pub_date_kst: DateTime<FixedOffset> = pub_date_timestamp.with_timezone(&kst);
                let pub_date_rfc2822 = pub_date_kst.to_rfc2822();

                let image_link = v
                    .get("node")
                    .and_then(|v| v.get("image_versions2"))
                    .and_then(|v| v.get("candidates"))
                    .and_then(|v| v.get(0))
                    .and_then(|v| v.get("url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let item = ItemBuilder::default()
                    .title(title.to_string())
                    .description(description)
                    .link(link)
                    .author(author.to_string())
                    .pub_date(pub_date_rfc2822)
                    .build();

                items.0.push(item);
                items.1.push(image_link);
            }
            channel.0.items = items.0;
            let _ = item_service::create_rss_items_and_embedding(
                pool,
                embedding_service,
                channel.0,
                Some(items.1),
                channel.1,
            )
            .await
            .map_err(|e| {
                error!(
                    "[Service-instagram] Failed to create rss item and embedding: {:?}",
                    e
                );
                e
            });
            Ok(channel.1)
        }
        Err(_) => {
            error!("[Instagram] Failed to get body in graphql data.");
            Err(OmniNewsError::NotFound(
                "Failed to get body in graphql data.".to_string(),
            ))
        }
    }
}

async fn generate_rss_channel(
    pool: &sqlx::Pool<sqlx::MySql>,
    embedding_service: &EmbeddingService,
    link: &str,
    driver: &WebDriver,
    username: String,
) -> Result<(Channel, i32), OmniNewsError> {
    let _ = driver
        .goto(format!("http://instagram.com/{username}"))
        .await
        .map_err(map_wd_err);

    let (channel_title, channel_description, channel_image_url) =
        extract_profile_meta(driver).await?;
    let mut image = Image::default();
    image.set_url(channel_image_url.clone());
    image.set_title(channel_title.clone());
    image.set_description(Some(channel_description.clone()));
    let rss_channel: Channel = ChannelBuilder::default()
        .title(channel_title.clone())
        .description(channel_description)
        .link(link.to_string())
        .image(image)
        .generator("Omninews_instagram".to_string())
        .build();
    let channel = channel_service::make_rss_channel(
        &rss_channel,
        format!("Generated by Omninews, {channel_title}"),
        true,
    );
    let channel_id =
        channel_service::store_channel_and_embedding(pool, embedding_service, channel).await?;
    Ok((rss_channel, channel_id))
}

/* ---------------- Helper Functions ---------------- */

fn extract_username(link: &str) -> Option<String> {
    // https://www.instagram.com/{username}/
    let trimmed = link.trim_end_matches('/');
    trimmed.rsplit('/').next().map(|s| s.to_string())
}

async fn is_sign_in_by_graphql(driver: &WebDriver) -> Result<bool, OmniNewsError> {
    let body = driver.find(By::XPath(".//body")).await?;
    let body_len = body.text().await.unwrap().len();
    Ok(body_len > 200)
}

async fn is_login_page(driver: &WebDriver) -> Result<bool, OmniNewsError> {
    Ok(driver.find(By::Name("username")).await.is_ok()
        || driver.find(By::Name("password")).await.is_ok())
}

async fn attempt_login(driver: &WebDriver) -> Result<(), OmniNewsError> {
    let username = env::var("INSTAGRAM_ID").expect("INSTAGRAM_ID is must be set.");

    let password = env::var("INSTAGRAM_PW").expect("INSTAGRAM_PW is must be set.");

    info!("[Instagram] Attempting login...");

    let user_field = driver
        .find(By::Name("username"))
        .await
        .map_err(map_wd_err)?;
    let pass_field = driver
        .find(By::Name("password"))
        .await
        .map_err(map_wd_err)?;

    user_field.send_keys(username).await.map_err(map_wd_err)?;
    pass_field.send_keys(password).await.map_err(map_wd_err)?;

    sleep(Duration::from_millis(600)).await;
    if let Ok(btn) = driver.find(By::XPath("//button[@type='submit']")).await {
        btn.click().await.map_err(map_wd_err)?;
    }

    // save login info window
    sleep(Duration::from_millis(7000)).await;
    let save_info_el = driver
        .find(By::XPath(
            "//button[text()='정보 저장'] | //button[text()='Save info']",
        ))
        .await;

    if save_info_el.is_ok() {
        if let Ok(btn) = save_info_el {
            btn.click().await.map_err(map_wd_err)?;
            info!("[Instagram] Saved login info.");
        } else {
            info!("[Instagram] No 'Save login info' button found.");
        }
    }

    // 로그인 처리 대기 (최대 30초)
    for _ in 0..10 {
        if !is_login_page(driver).await? {
            info!("[Instagram] Login success (username/password gone).");
            return Ok(());
        }
        info!("[Instagram] Waiting for login...");
        sleep(Duration::from_secs(3)).await;
    }

    error!("[Service-Instagram] Login failed or timed out.");
    Err(OmniNewsError::WebDriverNotFound)
}

// TODO: 동작 안함.
async fn dismiss_close_overlay(driver: &WebDriver) -> Result<(), OmniNewsError> {
    // 한국어 title='닫기', 영어 title='Close'
    if let Ok(close_div) = driver
        .find(By::XPath(
            "//div[.//svg/title[text()='닫기' or text()='Close']]",
        ))
        .await
    {
        close_div.click().await.map_err(map_wd_err)?;
        sleep(Duration::from_millis(400)).await;
        info!("[Instagram] Closed overlay (Close/닫기).");
    }
    Ok(())
}

async fn extract_profile_meta(
    driver: &WebDriver,
) -> Result<(String, String, String), OmniNewsError> {
    // meta og:title, og:description, og:image
    let title = get_meta_content(driver, "og:title").await?;
    let desc = get_meta_content(driver, "og:description")
        .await
        .unwrap_or_else(|_| "".into());
    let img = get_meta_content(driver, "og:image")
        .await
        .unwrap_or_default();

    Ok((title, desc, img))
}

async fn get_meta_content(driver: &WebDriver, property: &str) -> Result<String, OmniNewsError> {
    let selector = format!("meta[property='{property}']");
    let el = driver.find(By::Css(&selector)).await.map_err(map_wd_err)?;
    let content = el
        .attr("content")
        .await
        .map_err(map_wd_err)?
        .unwrap_or_default();
    Ok(content)
}

fn map_wd_err(e: WebDriverError) -> OmniNewsError {
    OmniNewsError::WebDriverError(e)
}
