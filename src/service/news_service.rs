use std::env;

use crate::{
    dto::news::{
        request::{ApiNewsRequestDto, NewsRequestDto},
        response::{NewsApiResponseDto, NewsResponseDto},
    },
    model::error::OmniNewsError,
    news_error, news_info,
    repository::news_repository,
};
use chrono::NaiveDateTime;
use reqwest::{header::HeaderMap, Response};
use sqlx::MySqlPool;

pub async fn get_news(
    pool: &MySqlPool,
    value: NewsRequestDto,
) -> Result<Vec<NewsResponseDto>, OmniNewsError> {
    let category = value.category.unwrap_or_default();
    let page = value.page.unwrap_or(1);
    let size = 20;
    let offset = (page - 1) * size;

    match news_repository::select_news_by_category(pool, &category, size, offset).await {
        Ok(news) => Ok(NewsResponseDto::from_model_list(news)),

        Err(e) => {
            news_error!("[Service] Failed to fetch news: {:?}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn get_news_by_api(
    params: ApiNewsRequestDto,
) -> Result<Vec<NewsApiResponseDto>, OmniNewsError> {
    let res = request_naver_news_api(params).await?;

    let xml_data = res.text().await.map_err(|e| {
        news_error!("[Service] Failed to fetch news items: {:?}", e);
        OmniNewsError::FetchNews
    })?;

    get_news_items_by_xml(xml_data)
}

async fn request_naver_news_api(params: ApiNewsRequestDto) -> Result<Response, OmniNewsError> {
    let client = reqwest::Client::new();

    let mut head = HeaderMap::new();
    head.append(
        "X-Naver-Client-Id",
        env::var("NAVER_CLIENT_ID")
            .expect("NAVER_CLIENT_ID is must be set")
            .parse()
            .unwrap(),
    );
    head.append(
        "X-Naver-Client-Secret",
        env::var("NAVER_CLIENT_SECRET")
            .expect("NAVER_CLIENT_SECRET is must be set")
            .parse()
            .unwrap(),
    );

    let url = format!(
        "https://openapi.naver.com/v1/search/news.xml?query={}&display={}&sort={}",
        params.query.unwrap_or_default(),
        params.display.unwrap_or_default(),
        params.sort.unwrap_or_default()
    );

    client.get(url).headers(head).send().await.map_err(|e| {
        news_error!("[Service] Failed to fetch news: {:?}", e);
        OmniNewsError::FetchNews
    })
}

fn get_news_items_by_xml(xml_data: String) -> Result<Vec<NewsApiResponseDto>, OmniNewsError> {
    use quick_xml::de::from_str;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct NewsRss {
        channel: Channel,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[allow(non_snake_case)]
    struct Channel {
        title: String,
        link: String,
        description: String,
        lastBuildDate: String,
        total: u32,
        start: u32,
        display: u32,
        item: Vec<NewsItemAPI>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[allow(non_snake_case)]
    struct NewsItemAPI {
        title: String,
        originallink: String,
        link: String,
        description: String,
        pubDate: String,
    }

    let rss: NewsRss = from_str(xml_data.as_str()).map_err(|e| {
        news_error!("[Service] Failed to parse xml: {:?}", e);
        OmniNewsError::FetchNews
    })?;

    let items = rss.channel.item;

    let mut result: Vec<NewsApiResponseDto> = Vec::new();
    for item in items {
        result.push(NewsApiResponseDto::new(
            item.title,
            item.originallink,
            item.link,
            item.description,
            NaiveDateTime::parse_from_str(&item.pubDate, "%a, %d %b %Y %H:%M:%S %z")
                .unwrap_or_default(),
        ));
    }

    Ok(result)
}
