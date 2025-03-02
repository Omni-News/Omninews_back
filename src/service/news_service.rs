use std::collections::HashMap;

use crate::{
    model::{
        error::OmniNewsError,
        news::{NewNews, News},
    },
    repository::news_repository,
};
use chrono::{Duration, NaiveDateTime};
use reqwest::Client;
use rocket::State;
use scraper::{Html, Selector};
use sqlx::MySqlPool;
use tokio::time;

type NewsType = HashMap<String, i32>;

pub async fn get_news(
    pool: &State<MySqlPool>,
    category: String,
) -> Result<Vec<News>, OmniNewsError> {
    match news_repository::select_news_by_category(pool, category.clone()).await {
        Ok(news) => Ok(news),
        Err(e) => {
            error!("Failed to select news with {}: {:?}", category, e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn crawl_news_and_store_every_5_minutes(
    pool: &State<MySqlPool>,
) -> Result<(), OmniNewsError> {
    let mut handle = time::interval(time::Duration::from_secs(300));

    let news_type = set_news_type();

    loop {
        handle.tick().await;

        match fetch_news_and_store(pool, news_type.clone()).await {
            Ok(_) => info!("Successfully fetched news"),
            Err(e) => {
                error!("Failed to fetch news: {:?}", e);
                return Err(OmniNewsError::FetchNews);
            }
        };
    }
}

/*
* 정치 : 100
* 경제 : 101
* 사회 : 102
* 생활/문화 : 103
* 세계 : 104
* IT/과학 : 105
*/
fn set_news_type() -> NewsType {
    let mut news_type = HashMap::new();
    news_type.insert("정치".to_string(), 100);
    news_type.insert("경제".to_string(), 101);
    news_type.insert("사회".to_string(), 102);
    news_type.insert("생활/문화".to_string(), 103);
    news_type.insert("세계".to_string(), 104);
    news_type.insert("IT/과학".to_string(), 105);

    news_type
}

async fn fetch_news_and_store(
    pool: &State<MySqlPool>,
    news_type: NewsType,
) -> Result<(), OmniNewsError> {
    let client = Client::new();

    for (subject, code) in &news_type {
        let res = client
            .get(format!("https://news.naver.com/section/{}", code))
            .send()
            .await?
            .text()
            .await?;

        let document = Html::parse_document(&res);
        let news_selector = Selector::parse(".sa_item_flex").unwrap();

        let newsses = make_news(document, news_selector, subject);

        for news in newsses {
            match news_repository::select_news_by_title(pool, news.news_title.clone().unwrap())
                .await
            {
                Ok(i) => (),
                Err(e) => {
                    let _ = news_repository::insert_news(pool, news).await.map_err(|e| {
                        error!("Failed to insert news: {:?}", e);
                        OmniNewsError::Database(e)
                    });
                }
            };
        }
    }
    Ok(())
}

fn make_news(document: Html, news_selector: Selector, subject: &String) -> Vec<NewNews> {
    let title_selector = Selector::parse(".sa_text_strong").unwrap();
    let description_selector = Selector::parse(".sa_text_lede").unwrap();
    let link_selector = Selector::parse(".sa_thumb_link").unwrap();
    let source_selector = Selector::parse(".sa_text_press").unwrap();
    let pub_date_selector = Selector::parse(".sa_text_datetime > b").unwrap();
    let image_link_selector = Selector::parse(".sa_thumb_link > img").unwrap();

    document
        .select(&news_selector)
        .map(|news| NewNews {
            news_title: Some(
                news.select(&title_selector)
                    .next()
                    .map(|e| e.inner_html())
                    .unwrap_or_default(),
            ),
            news_description: Some(
                news.select(&description_selector)
                    .next()
                    .map(|e| e.inner_html())
                    .unwrap_or_default(),
            ),
            news_link: Some(
                news.select(&link_selector)
                    .next()
                    .map(|e| e.attr("href").unwrap())
                    .unwrap_or_default()
                    .to_string(),
            ),
            news_source: Some(
                news.select(&source_selector)
                    .next()
                    .map(|e| e.inner_html())
                    .unwrap_or_default(),
            ),
            // TODO pub_date to NaiveTime
            news_pub_date: pub_date_to_naive_time(
                news.select(&pub_date_selector)
                    .next()
                    .map(|e| e.inner_html())
                    .unwrap_or_default(),
            ),
            news_image_link: Some(
                news.select(&image_link_selector)
                    .next()
                    .map(|e| e.attr("data-src").unwrap_or_default())
                    .unwrap_or_default()
                    .to_string(),
            ),
            news_category: Some(subject.to_string()),
        })
        .collect::<Vec<NewNews>>()
}

fn pub_date_to_naive_time(pub_date: String) -> Option<NaiveDateTime> {
    let current_time = chrono::Local::now();

    if pub_date.is_empty() {
        return None;
    }

    // 한글은 글자 당 3바이트
    if pub_date.ends_with("분전") {
        let (value, _) = pub_date.split_at(pub_date.len() - 6);
        Some(
            NaiveDateTime::parse_from_str(
                &current_time
                    .checked_sub_signed(Duration::minutes(value.parse::<i64>().unwrap_or_default()))
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap_or_default(),
        )
    } else if pub_date.ends_with("시간전") {
        let (value, _) = pub_date.split_at(pub_date.len() - 9);
        Some(
            NaiveDateTime::parse_from_str(
                &current_time
                    .checked_sub_signed(Duration::hours(value.parse::<i64>().unwrap_or_default()))
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap_or_default(),
        )
    } else {
        None
    }
}

async fn get_news() {
    let client = reqwest::Client::new();

    let mut head = HeaderMap::new();
    head.append("X-Naver-Client-Id", "4jfC5AiK9mIpPzUSJWGG".parse().unwrap());
    head.append("X-Naver-Client-Secret", "Aw__VeIGuh".parse().unwrap());

    let res = client
    .get("https://openapi.naver.com/v1/search/news.xml?query=\"Rust언어\"&display=10&sort=sim")
    .headers(head)
    .send()
    .await
}