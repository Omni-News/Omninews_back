use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{
        rss::{RssChannel, RssItem},
        search::SearchType,
    },
    repository::{rss_channel_repository, rss_item_repository},
};

pub async fn find_rss_channel_by_morpheme_id(
    pool: &State<MySqlPool>,
    morpheme_id: i32,
    order_by: SearchType,
) -> Result<Vec<RssChannel>, ()> {
    let result = match order_by {
        SearchType::Accuracy => {
            rss_channel_repository::select_rss_channels_by_morpheme_id_order_by_source_rank(
                pool,
                morpheme_id,
            )
            .await
        }
        SearchType::Popularity => {
            rss_channel_repository::select_rss_channels_by_morpheme_id_order_by_channel_rank(
                pool,
                morpheme_id,
            )
            .await
        }
        SearchType::Latest => Ok(Vec::new()), // Channel doesn't have pub_date()
    };

    result.map_err(|_| ())
}

pub async fn find_rss_item_by_morpheme_id(
    pool: &State<MySqlPool>,
    morpheme_id: i32,
    order_by: SearchType,
) -> Result<Vec<RssItem>, ()> {
    let result = match order_by {
        SearchType::Accuracy => {
            rss_item_repository::select_rss_items_by_morpheme_id_order_by_source_rank(
                pool,
                morpheme_id,
            )
            .await
        }
        SearchType::Popularity => {
            rss_item_repository::select_rss_items_by_morpheme_id_order_by_rss_rank(
                pool,
                morpheme_id,
            )
            .await
        }
        SearchType::Latest => {
            rss_item_repository::select_rss_items_by_morpheme_id_order_by_pub_date(
                pool,
                morpheme_id,
            )
            .await
        }
    };

    result.map_err(|_| ())
}
