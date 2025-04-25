use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::{db_util::get_db, model::rss::RssItem};

pub async fn insert_user_subscribe_channel(
    pool: &State<MySqlPool>,
    user_id: i32,
    channel_id: i32,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "INSERT INTO user_subscription_channel(user_id, channel_id) VALUES (?, ?);",
        user_id,
        channel_id
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => Err(e),
    }
}

pub async fn select_subscription_items(
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

pub async fn delete_subscribe_channel(
    pool: &State<MySqlPool>,
    user_id: i32,
    channel_id: i32,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "DELETE FROM user_subscription_channel WHERE user_id = ? AND channel_id = ?",
        user_id,
        channel_id
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => Err(e),
    }
}
