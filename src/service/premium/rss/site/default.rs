use std::collections::HashSet;
use std::time::{Duration, Instant};

use reqwest::Url;
use serde_json::Value;
use sqlx::MySqlPool;
use thirtyfour::{error::WebDriverResult, WebDriver};

use crate::config::webdriver::{AcquireStrategy, DriverPool};
use crate::{
    model::error::OmniNewsError, service::channel_service, utils::embedding_util::EmbeddingService,
};

pub async fn generate_rss(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    driver_pool: &DriverPool,
    link: &str,
) -> Result<i32, OmniNewsError> {
    // acquire driver
    let strategy = AcquireStrategy::Wait(Some(Duration::from_secs(10)));
    let driver_handler = driver_pool.acquire(strategy).await.map_err(|e| {
        error!("[Service] Failed to acquire WebDriver: {}", e);
        OmniNewsError::WebDriverPool(e)
    })?;
    let driver = driver_handler.driver();

    let feeds = extract_feed(driver, link)
        .await
        .map_err(OmniNewsError::WebDriverError)?;

    info!("Discovered feed URLs: {:?}", feeds);

    if !feeds.is_empty() {
        let rss_link;
        if let Some(u) = feeds.iter().find(|u| !u.contains("comments")) {
            rss_link = u.to_string();
        } else {
            rss_link = feeds.first().unwrap_or(&"".to_string()).to_string();
        }

        return channel_service::create_rss_and_embedding_with_web_driver(
            pool,
            embedding_service,
            rss_link,
            driver,
        )
        .await;
    }

    error!(
        "[Service] No feed links found for the provided link: {}",
        link
    );
    Err(OmniNewsError::NotFound(
        "Failed to find feed links".to_string(),
    ))
}

pub async fn extract_feed(driver: &WebDriver, start_url: &str) -> WebDriverResult<Vec<String>> {
    driver.goto(start_url).await?;
    wait_for_dom_ready(driver, Duration::from_secs(5))
        .await
        .ok();

    // link[rel="alternate"] 피드 링크 찾기
    let mut feed_urls = detect_feed_links(driver).await.unwrap_or_default();

    // 일반적인 RSS 링크 패턴 찾기
    if feed_urls.is_empty() {
        info!("[Service] No feed links found in link[rel='alternate'], trying common RSS patterns");

        // 일반적인 RSS 패턴 생성
        let candidates = build_feed_candidates(start_url);

        // 후보 URL 검증
        if !candidates.is_empty() {
            let valid = verify_feed_candidates(driver, &candidates).await?;
            feed_urls.extend(valid);
        }
    }

    Ok(feed_urls)
}

pub async fn wait_for_dom_ready(driver: &WebDriver, timeout: Duration) -> WebDriverResult<()> {
    let start = Instant::now();
    loop {
        let ready = driver
            .execute("return document.readyState;", Vec::<Value>::new())
            .await?
            .json()
            .to_string();

        if ready == "interactive" || ready == "complete" {
            break;
        }
        if start.elapsed() > timeout {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Ok(())
}

// <link rel="alternate">로 Rss/Atom/JSON Feed 링크 탐지
async fn detect_feed_links(driver: &WebDriver) -> WebDriverResult<Vec<String>> {
    // head에 있는 rel=alternamte 링크를 찾기
    let js = r#"
        const links = Array.from(document.querySelectorAll('link[rel="alternate"]'));
        return links.map(l => ({
            type: (l.getAttribute('type') || '').toLowerCase(),
            href: l.getAttribute('href') || ''
            }));
    "#;

    let mut out = Vec::new();
    if let Ok(ret) = driver.execute(js, Vec::<Value>::new()).await {
        if let Some(vals) = ret.json().as_array() {
            for v in vals {
                let type_str = v
                    .get("type")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let href = v
                    .get("href")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                if href.is_empty() {
                    continue;
                }

                // RSS/Atom/JSON Feed 타입 필터링
                let is_feed = type_str.contains("rss")
                    || type_str.contains("atom")
                    || type_str == "application/feed+json";
                if is_feed {
                    // 상대 URL을 절대 URL로 변환
                    let abs_url = if let Ok(current) = driver.current_url().await {
                        make_absolute_url(&href, current.as_ref())
                    } else {
                        href
                    };
                    out.push(abs_url);
                }
            }
        }
    }
    Ok(out)
}

/// 일반적인 RSS/Atom/JSON Feed 패턴 기반 후보 URL 생성
fn build_feed_candidates(input_url: &str) -> Vec<String> {
    let url = match Url::parse(input_url) {
        Ok(u) => u,
        Err(_) => return vec![],
    };

    let origin = {
        let mut s = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());
        if let Some(port) = url.port() {
            s.push(':');
            s.push_str(&port.to_string());
        }
        s
    };

    // 경로 세그먼트 추출
    let segments: Vec<String> = url
        .path_segments()
        .map(|it| {
            it.filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(Vec::new);

    // 1) 루트 레벨 일반 피드 경로
    let root_suffixes = [
        "/rss",
        "/rss.xml",
        "/feed",
        "/feed/",
        "/feed.xml",
        "/atom.xml",
        "/index.xml",
        "/rss/",
        "/feed.json",
        "/?format=rss",
        "/feeds/posts/default?alt=rss",
    ];

    // 2) 섹션 접두사 (blog/news/posts/articles/stories/updates/press)
    let mut prefixes: Vec<String> = vec![];
    if let Some(first) = segments.first() {
        if looks_like_collection(first) {
            prefixes.push(format!("/{first}"));
        }
    }

    // 3) 카테고리/태그 피드 (워드프레스 스타일)
    let mut category_tag_candidates: Vec<String> = vec![];
    if segments.len() >= 2 {
        let first = &segments[0].to_lowercase();
        let second = &segments[1];
        if (first == "category" || first == "tag") && !second.is_empty() {
            category_tag_candidates.push(format!("{origin}/category/{second}/feed"));
            category_tag_candidates.push(format!("{origin}/tag/{second}/feed"));
            category_tag_candidates.push(format!("{origin}/category/{second}/rss"));
            category_tag_candidates.push(format!("{origin}/tag/{second}/rss"));
        }
    }

    // 4) 최종 목록 구성
    let mut out = Vec::new();

    // 루트 레벨
    for suf in root_suffixes {
        out.push(format!("{origin}{suf}"));
    }

    // 섹션 레벨 (예: /blog/feed, /blog/atom.xml)
    let section_suffixes = [
        "/rss",
        "/rss.xml",
        "/rss/",
        "/feed",
        "/feed/",
        "/feed.xml",
        "/atom.xml",
        "/index.xml",
        "/feed.json",
    ];
    for pre in prefixes {
        for suf in &section_suffixes {
            out.push(format!("{origin}{pre}{suf}"));
        }
    }

    // 카테고리/태그 레벨
    out.extend(category_tag_candidates);

    // 5) 워드프레스 쿼리 파라미터 스타일
    out.push(format!("{origin}/?feed=rss2"));
    out.push(format!("{origin}/?feed=atom"));

    dedup(out)
}

/// 브라우저의 fetch를 사용해 피드 URL 후보들의 유효성을 검증
async fn verify_feed_candidates(
    driver: &WebDriver,
    candidates: &[String],
) -> WebDriverResult<Vec<String>> {
    let mut valid_feeds = Vec::new();

    for candidate in candidates {
        // 각 후보 URL에 대해 비동기 fetch 요청으로 검증
        let js = format!(
            r#"
            return (async () => {{
                try {{
                    const response = await fetch("{candidate}", {{
                        method: 'GET',
                        headers: {{
                            'Accept': 'application/rss+xml, application/atom+xml, application/xml;q=0.9, text/xml;q=0.8, application/feed+json;q=0.9, */*;q=0.1'
                        }},
                        cache: 'no-store'
                    }});
                    
                    if (!response.ok) return false;
                    
                    const contentType = response.headers.get('content-type') || '';
                    const isFeedType = contentType.includes('xml') || 
                                       contentType.includes('rss') || 
                                       contentType.includes('atom') ||
                                       contentType.includes('feed+json');
                    
                    if (isFeedType) return true;
                    
                    // 컨텐츠 타입이 명확하지 않으면 본문 확인
                    const text = await response.text();
                    const lowerText = text.toLowerCase();
                    return lowerText.includes('<rss') || 
                           lowerText.includes('<feed') || 
                           (lowerText.includes('<?xml') && 
                            (lowerText.includes('<channel') || lowerText.includes('<feed'))) ||
                           lowerText.includes('jsonfeed');
                }} catch (e) {{
                    return false;
                }}
            }})();
        "#
        );

        let result = driver.execute(js, Vec::<Value>::new()).await?;

        if let Some(is_valid) = result.json().as_bool() {
            if is_valid {
                valid_feeds.push(candidate.clone());
            }
        }
    }

    Ok(valid_feeds)
}

/// 상대 URL을 절대 URL로 변환
fn make_absolute_url(href: &str, base: &str) -> String {
    if let Ok(base_url) = Url::parse(base) {
        if let Ok(abs_url) = base_url.join(href) {
            return abs_url.to_string();
        }
    }
    href.to_string()
}

/// 컬렉션 세그먼트 확인 (blog/news/posts/articles/stories/updates/press)
fn looks_like_collection(seg: &str) -> bool {
    matches!(
        seg.to_lowercase().as_str(),
        "blog" | "news" | "posts" | "articles" | "stories" | "updates" | "press"
    )
}

/// 중복 제거
fn dedup(v: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in v {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}
