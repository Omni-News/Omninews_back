use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::db::get_db;
use crate::model::{NewRssChannel, RssChannel};

pub async fn select_all_rss_channel(
    pool: &State<MySqlPool>,
) -> Result<Vec<RssChannel>, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query_as!(RssChannel, "SELECT * FROM rss_channel;")
        .fetch_all(&mut *conn)
        .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!("Error finding rss_channels: {}", e);
            Err(e)
        }
    }
}

pub async fn select_rss_channel_by_id(
    pool: &State<MySqlPool>,
    rss_channel_id: u64,
) -> Result<RssChannel, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query_as!(
        RssChannel,
        "SELECT * FROM rss_channel
                WHERE channel_id = ?",
        rss_channel_id,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!("Error selecting rss channel by id: {}", e);
            Err(e)
        }
    }
}

pub async fn select_rss_channel_by_link(
    pool: &State<MySqlPool>,
    rss_channel_link: String,
) -> Result<RssChannel, sqlx::Error> {
    let mut conn = get_db(pool).await;
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
        Err(e) => {
            eprintln!("Error selecting rss channel by link: {}", e);
            Err(e)
        }
    }
}

pub async fn select_rss_channels_by_morpheme_id_order_by_source_rank(
    pool: &State<MySqlPool>,
    morpheme_id: u64,
) -> Result<Vec<RssChannel>, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query_as!(
        RssChannel,
        "SELECT r.* 
        FROM rss_channel r 
        JOIN morpheme_to_source_link m 
        ON r.channel_id = m.channel_id
        WHERE m.morpheme_id=?
        ORDER BY m.source_rank DESC;",
        morpheme_id as i32,
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!(
                "Error selecting rss channels by morpheme id order by source rank : {}",
                e
            );
            Err(e)
        }
    }
}

pub async fn select_rss_channels_by_morpheme_id_order_by_channel_rank(
    pool: &State<MySqlPool>,
    morpheme_id: u64,
) -> Result<Vec<RssChannel>, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query_as!(
        RssChannel,
        "SELECT r.* 
        FROM rss_channel r 
        JOIN morpheme_to_source_link m 
        ON r.channel_id = m.channel_id
        WHERE m.morpheme_id=?
        ORDER BY r.channel_rank DESC;",
        morpheme_id as i32,
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!(
                "Error selecting rss channels by morpheme id order by channel rank: {}",
                e
            );
            Err(e)
        }
    }
}

pub async fn insert_rss_channel(
    pool: &State<MySqlPool>,
    rss_channel: NewRssChannel,
) -> Result<u64, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query!(
        "INSERT INTO rss_channel 
            (channel_title, channel_description, channel_link, channel_image_url, channel_language, rss_generator, channel_rank)
            VALUES (?, ?, ?, ?, ?, ?, ?);",
        rss_channel.channel_title,
        rss_channel.channel_description,
        rss_channel.channel_link,
        rss_channel.channel_image_url,
        rss_channel.channel_language,
        rss_channel.rss_generator,
        rss_channel.channel_rank,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id()),
        Err(e) => {
            eprintln!("Error inserting RSS channel: {}", e);
            Err(e)
        }
    }
}

pub async fn update_rss_channel_by_id(
    pool: &State<MySqlPool>,
    rss_channel: RssChannel,
) -> Result<u64, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query!(
        "UPDATE rss_channel
        SET channel_title = ?,
        channel_description = ?,
        channel_link = ?,
        channel_image_url = ?,
        channel_language = ?,
        rss_generator = ?,
        channel_rank = ?
    WHERE channel_id = ?;",
        rss_channel.channel_title,
        rss_channel.channel_description,
        rss_channel.channel_link,
        rss_channel.channel_image_url,
        rss_channel.channel_language,
        rss_channel.rss_generator,
        rss_channel.channel_rank,
        rss_channel.channel_id,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(_) => Ok(rss_channel.channel_id.unwrap() as u64),
        Err(e) => {
            eprintln!("Error updating rss channel: {}", e);
            Err(e)
        }
    }
}
