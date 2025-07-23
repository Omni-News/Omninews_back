// 10분마다 내 DB에 등록된 RSS들을 가져와서 추가로 등록된 글 있는지 검증 후, 추가로 등록되었다면,
// 내 DB에 추가하고, 그 Rss를 구독하고 있던 사용자들에게 알림을 보낸다.

/*
* 1. DB에서 RSS채널 목록 가져오기.
* 2. 해당 RSS 채널들 스크래핑하기.
*   2-1. 가장 최신 글부터,내 DB에 없다면 글 추가하기.
*   2-2. 만약 내 DB에 있는 글이라면, 그때부터 추가 X
* 3. 추가할 때마다, 해당 Rss채널을 구독하고 있는 사용자에게 알림 보냄.
*/

use std::time::Duration;

use rss::Item;
use sqlx::MySqlPool;
use tokio::time::{interval_at, Instant};

use crate::{
    model::{error::OmniNewsError, rss::RssChannel},
    service::{
        rss::{
            channel_service::{get_all_rss_channels, parse_rss_link_to_channel},
            item_service::{self, create_rss_item_and_embedding},
        },
        user_service,
    },
    utils::{embedding_util::EmbeddingService, firebase::send_fcm::send_fcm_message},
};

///
/// DB에 저장된 Rss Channel 최신화 및 알림 보내기
///
pub async fn rss_scheduler(pool: &MySqlPool, embedding_service: &EmbeddingService) {
    // loop for 10 minutes
    let mut interval = interval_at(Instant::now(), Duration::from_secs(10));

    loop {
        interval.tick().await;
        info!("[Scheduler] Rss Scheduler started");
        let channels = get_all_rss_channels(pool).await.unwrap();

        for channel in channels {
            let channel_id = channel.channel_id.unwrap_or_default();

            let items_len = item_service::get_items_len_by_channel_id(pool, channel_id)
                .await
                .unwrap();

            for index in 0..items_len {
                let mut item = get_rss_item_by_channel_from_scraping(index, channel.clone())
                    .await
                    .unwrap();

                // 이미 있는 글임
                if let Ok(res) = item_service::is_exist_rss_item_by_link(
                    pool,
                    item.link.clone().unwrap_or_default(),
                )
                .await
                {
                    if res {
                        break;
                    }
                }

                let _ = create_rss_item_and_embedding(
                    pool,
                    embedding_service,
                    channel_id,
                    channel.channel_image_url.clone().unwrap_or_default(),
                    &mut item,
                )
                .await
                .unwrap();

                // Rss채널 구독한 사람들 토큰 가져와서 뿌리기
                let users_tokens =
                    user_service::get_users_fcm_token_subscribed_channel_by_channel_id(
                        pool, channel_id,
                    )
                    .await
                    .unwrap();

                send_notification_each_user(users_tokens, channel.clone(), &mut item)
                    .await
                    .unwrap();
            }
        }
        info!("[Scheduler] Rss Scheduler Ended");
    }
}

pub async fn get_rss_item_by_channel_from_scraping(
    index: i32,
    channel: RssChannel,
) -> Result<Item, OmniNewsError> {
    let link = channel.channel_rss_link.clone().unwrap_or_default();

    let mut channel = parse_rss_link_to_channel(&link).await?;
    Ok(channel
        .items_mut()
        .get(index as usize)
        .cloned()
        .unwrap_or_default())
}

pub async fn send_notification_each_user(
    tokens: Vec<String>,
    channel: RssChannel,
    item: &mut Item,
) -> Result<(), OmniNewsError> {
    for token in tokens {
        send_fcm_message(
            token,
            format!(
                "{:?}의 새로운 RSS",
                channel.channel_title.clone().unwrap_or_default().as_str()
            ),
            format!("{:?}", item.title.clone().unwrap_or_default().as_str()),
        )
        .await
        .map_err(|_| OmniNewsError::FirebaseError)?;
    }
    Ok(())
}
