use sqlx::MySqlPool;

use crate::{model::error::OmniNewsError, utils::annoy_util::save_annoy};

pub async fn save_annoy_scheduler(pool: &MySqlPool) -> Result<(), OmniNewsError> {
    save_annoy(pool).await?;

    Ok(())
}
