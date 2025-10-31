use sqlx::{query, query_as, MySqlPool};

use crate::{
    db_util::get_db,
    model::rss::{RssChannel, RssItem},
};

pub async fn insert_user_subscribe_channel(
    pool: &MySqlPool,
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

pub async fn select_subscription_channels(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<Vec<RssChannel>, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query_as!(
        RssChannel,
        "SELECT rc.*
            FROM rss_channel rc
            JOIN user_subscription_channel usc ON rc.channel_id = usc.channel_id
            WHERE usc.user_id = ?;",
        user_id
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_subscription_items(
    pool: &MySqlPool,
    channels: Vec<i32>,
    offset: i32,
    size: i32,
) -> Result<Vec<RssItem>, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let placeholder = (0..channels.len())
        .map(|_| "?".to_string())
        .collect::<Vec<String>>()
        .join(",");

    let query = format!(
        "SELECT * FROM rss_item 
        WHERE channel_id IN ({placeholder})
        ORDER BY rss_item.rss_pub_date DESC
        LIMIT ? OFFSET ?;",
    );

    let mut qurey_builder = query_as::<_, RssItem>(&query);

    for id in channels {
        qurey_builder = qurey_builder.bind(id);
    }
    qurey_builder = qurey_builder.bind(size);
    qurey_builder = qurey_builder.bind(offset);

    let result = qurey_builder.fetch_all(&mut *conn).await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn is_already_subscribe_channel(
    pool: &MySqlPool,
    user_id: i32,
    channel_id: i32,
) -> Result<bool, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        "SELECT * FROM user_subscription_channel WHERE user_id = ? AND channel_id = ?",
        user_id,
        channel_id
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(_) => Ok(true),
        Err(e) => match e {
            sqlx::Error::RowNotFound => Ok(false),
            _ => Err(e),
        },
    }
}

pub async fn delete_subscribe_channel(
    pool: &MySqlPool,
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
