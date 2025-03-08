use rocket::State;
use sqlx::{query_as, MySqlPool};

use crate::{db::get_db, model::rss::RssItem};

pub async fn select_subscribe_items(
    pool: &State<MySqlPool>,
    channels: Vec<i32>,
) -> Result<Vec<RssItem>, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let placeholder = (0..channels.len())
        .map(|_| "?".to_string())
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "SELECT * FROM rss_item WHERE channel_id IN ({})",
        placeholder
    );

    let mut qurey_builder = query_as::<_, RssItem>(&query);

    for id in channels {
        qurey_builder = qurey_builder.bind(id);
    }

    let result = qurey_builder.fetch_all(&mut *conn).await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}
