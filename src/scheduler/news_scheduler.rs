use std::time::Duration;

use chrono::{Datelike, Utc};
use sqlx::MySqlPool;
use tokio::{
    task,
    time::{interval_at, Instant},
};

use crate::global::FETCH_FLAG;

pub async fn fetch_news_scheduler(pool: &MySqlPool) {
    let mut interval = interval_at(Instant::now(), Duration::from_secs(300)); // 5 minutes

    loop {
        interval.tick().await;

        if !*FETCH_FLAG.lock().unwrap() {
            info!("[Scheduler] Stop fetching news");
            continue;
        }

        // 뉴스 패치 중에는 fetch_flag를 false로 설정
        // 비동기 함수에 Send 트레이트가 필요하므로, task::spawn_blocking을 사용하여 처리
        task::spawn_blocking(move || {
            let mut fetch_flag = FETCH_FLAG.lock().unwrap();
            *fetch_flag = false;
            info!("[Scheduler] Fetching news");
        })
        .await
        .unwrap();

        match crate::service::news_service::crawl_news_and_store_every_5_minutes(pool).await {
            Ok(_) => {
                info!("[Scheduler] Successfully fetched news");
            }
            Err(e) => error!("[Scheduler] Failed to fetch news: {:?}", e),
        };
    }
}

// Delete old news that past 1 week
pub async fn delete_old_news_scheduler(pool: &MySqlPool) {
    let now = Utc::now();
    let midnight = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let next_midnight = if now.time() < midnight.time() {
        midnight
    } else {
        midnight + chrono::Duration::days(1)
    };

    let wait_time = (next_midnight - now.naive_utc()).to_std().unwrap();

    tokio::time::sleep(wait_time).await;

    let mut interval = interval_at(Instant::now(), Duration::from_secs(86400)); // 24
                                                                                // hours
    loop {
        interval.tick().await;

        let today = Utc::now().weekday();
        if today == chrono::Weekday::Sun {
            match crate::service::news_service::delete_old_news(pool).await {
                Ok(_) => warn!("Successfully deleted old news"),
                Err(e) => error!("Failed to delete old news: {:?}", e),
            }
        }
    }
}
