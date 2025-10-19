use sqlx::{query, query_as, MySqlPool};

use crate::db_util::get_db;
use crate::model::rss::{NewRssChannel, RssChannel};

pub async fn select_rss_channel_by_id(
    pool: &MySqlPool,
    channel_id: i32,
) -> Result<RssChannel, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssChannel,
        "SELECT * FROM rss_channel
                WHERE channel_id = ?",
        channel_id,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}
pub async fn select_rss_channel_by_rss_link(
    pool: &MySqlPool,
    rss_channel_link: String,
) -> Result<RssChannel, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssChannel,
        "SELECT * FROM rss_channel
                WHERE channel_rss_link = ?",
        rss_channel_link,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_rss_channel_by_channel_link(
    pool: &MySqlPool,
    channel_link: &str,
) -> Result<RssChannel, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssChannel,
        "SELECT * FROM rss_channel
                WHERE channel_link LIKE ?",
        channel_link,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_rss_channel_by_embedding_id(
    pool: &MySqlPool,
    search_value: &str,
    _embedding_id: i32,
) -> Result<RssChannel, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let _ = "SELECT r.*
        FROM rss_channel r 
        JOIN embedding e 
        ON r.channel_id = e.channel_id
        WHERE e.embedding_id=?;";

    let result = query_as!(
        RssChannel,
        "SELECT *
        FROM rss_channel
        WHERE channel_title LIKE CONCAT('%', ?, '%')
        OR channel_description LIKE CONCAT('%', ?, '%');",
        search_value,
        search_value,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}
pub async fn select_rss_channels_order_by_channel_rank(
    pool: &MySqlPool,
) -> Result<Vec<RssChannel>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssChannel,
        "SELECT * FROM rss_channel
                ORDER BY channel_rank DESC",
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn insert_rss_channel(
    pool: &MySqlPool,
    rss_channel: NewRssChannel,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query!(
        "INSERT INTO rss_channel 
            (channel_title, channel_description, channel_link, channel_image_url, channel_language, rss_generator, channel_rank, channel_rss_link)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?);",
        rss_channel.channel_title,
        rss_channel.channel_description,
        rss_channel.channel_link,
        rss_channel.channel_image_url,
        rss_channel.channel_language,
        rss_channel.rss_generator,
        rss_channel.channel_rank,
        rss_channel.channel_rss_link,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => Err(e),
    }
}

pub async fn update_rss_channel_rank_by_id(
    pool: &MySqlPool,
    channel_id: i32,
    num: i32,
) -> Result<bool, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query!(
        "UPDATE rss_channel
        SET channel_rank = channel_rank + ?
        WHERE channel_id = ?;
        ",
        num,
        channel_id
    )
    .execute(&mut *conn)
    .await?;

    if result.rows_affected() > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}
