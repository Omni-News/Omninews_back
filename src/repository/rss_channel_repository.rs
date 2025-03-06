use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::db::get_db;
use crate::model::rss::{NewRssChannel, RssChannel};

pub async fn select_rss_channel_by_link(
    pool: &State<MySqlPool>,
    rss_channel_link: String,
) -> Result<RssChannel, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssChannel,
        "SELECT * FROM rss_channel
                WHERE channel_link = ?",
        rss_channel_link,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_rss_channel_by_channel_rss_link(
    pool: &State<MySqlPool>,
    rss_link: String,
) -> Result<RssChannel, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssChannel,
        "SELECT * FROM rss_channel
                WHERE channel_rss_link = ?",
        rss_link,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_rss_channels_by_morpheme_id_order_by_source_rank(
    pool: &State<MySqlPool>,
    morpheme_id: i32,
) -> Result<Vec<RssChannel>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssChannel,
        "SELECT r.* 
        FROM rss_channel r 
        JOIN morpheme_link_mapping m 
        ON r.channel_id = m.channel_id
        WHERE m.morpheme_id=?
        ORDER BY m.source_rank DESC;",
        morpheme_id as i32,
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_rss_channels_by_morpheme_id_order_by_channel_rank(
    pool: &State<MySqlPool>,
    morpheme_id: i32,
) -> Result<Vec<RssChannel>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssChannel,
        "SELECT r.* 
        FROM rss_channel r 
        JOIN morpheme_link_mapping m 
        ON r.channel_id = m.channel_id
        WHERE m.morpheme_id=?
        ORDER BY r.channel_rank DESC;",
        morpheme_id as i32,
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_rss_channels_order_by_channel_rank(
    pool: &State<MySqlPool>,
) -> Result<Vec<RssChannel>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssChannel,
        "SELECT * FROM rss_channel
        ORDER BY channel_rank DESC
        LIMIT 50;",
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn insert_rss_channel(
    pool: &State<MySqlPool>,
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

pub async fn update_rss_channel(
    pool: &State<MySqlPool>,
    rss_channel: RssChannel,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query!(
        "UPDATE rss_channel
        SET channel_title = ?,
        channel_description = ?,
        channel_link = ?,
        channel_image_url = ?,
        channel_language = ?,
        rss_generator = ?,
        channel_rank = ?,
        channel_rss_link = ?
    WHERE channel_id = ?;",
        rss_channel.channel_title,
        rss_channel.channel_description,
        rss_channel.channel_link,
        rss_channel.channel_image_url,
        rss_channel.channel_language,
        rss_channel.rss_generator,
        rss_channel.channel_rank,
        rss_channel.channel_rss_link,
        rss_channel.channel_id,
    )
    .execute(&mut *conn)
    .await?;

    if result.rows_affected() > 0 {
        Ok(result.last_insert_id() as i32)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

pub async fn update_rss_channel_rank_by_link(
    pool: &State<MySqlPool>,
    rss_link: String,
    num: i32,
) -> Result<bool, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    info!("rss_link: {}", rss_link);
    let result = query!(
        "UPDATE rss_channel
        SET channel_rank = channel_rank + ?
        WHERE channel_rss_link = ?;
        ",
        num,
        rss_link
    )
    .execute(&mut *conn)
    .await?;

    if result.rows_affected() > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}
