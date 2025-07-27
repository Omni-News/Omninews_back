use std::time::Duration;

use tokio::time::{interval_at, Instant};

use crate::service::rss::channel_service;

pub async fn rss_info_update_scheduler(pool: &sqlx::MySqlPool) {
    // 1 day
    let mut interval = interval_at(Instant::now(), Duration::from_secs(60 * 60 * 24));

    loop {
        interval.tick().await;
        info!("[Scheduler] Rss Information Update Scheduler started");
        /*
         * 1. DB에서 보관중인 RSS 링크 가져오기.
         * 2. 해당 링크 통해 아래 정보 스크래이핑.
         *
         *            pub channel_title: Option<String>,
         *            pub channel_link: Option<String>,
         *            pub channel_description: Option<String>,
         *            pub channel_image_url: Option<String>,
         *            pub channel_language: Option<String>,
         *            pub rss_generator: Option<String>,
         *            pub channel_rank: Option<i32>,
         *            pub channel_rss_link: Option<String>
         * 3. DB에 내용 업데이트.
         *
         */
        let rss_links = channel_service::get_all_rss_links(pool).await.unwrap();
        for rss_link in &rss_links {
            let update_rss = channel_service::get_rss_info(rss_link).await.unwrap();
            match channel_service::update_rss_channel(pool, &update_rss).await {
                Ok(_) => info!(
                    "[Scheduler] Rss Information Update Scheduler updated: {}",
                    rss_link
                ),
                Err(e) => error!(
                    "[Scheduler] Failed Rss Information Update Scheduler: {}: {}",
                    rss_link, e
                ),
            };
        }
    }
}
