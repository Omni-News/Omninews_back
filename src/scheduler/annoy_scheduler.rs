use std::time::Duration;

use sqlx::MySqlPool;
use tokio::time::{interval_at, Instant};

use crate::utils::annoy_util::save_annoy;

pub async fn save_annoy_scheduler(pool: &MySqlPool) {
    // 1 hour
    let mut interval = interval_at(Instant::now(), Duration::from_secs(3600));

    loop {
        interval.tick().await;

        match save_annoy(pool).await {
            Ok(_) => info!("[Scheduler] Successfully saved annoy"),
            Err(e) => error!("[Scheduler] Failed to save annoy: {:?}", e),
        };
    }
}
