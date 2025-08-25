use sqlx::MySqlPool;

use crate::{
    model::{error::OmniNewsError, rss::ChannelCssElement},
    repository::channel_css_repository,
};

pub async fn store_channel_css_service(
    pool: &MySqlPool,
    channel_css_el: ChannelCssElement,
) -> Result<i32, OmniNewsError> {
    match channel_css_repository::insert_channel_css_element(pool, channel_css_el).await {
        Ok(res) => Ok(res),
        Err(e) => {
            info!("Failed to insert channel css element. {:?}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
