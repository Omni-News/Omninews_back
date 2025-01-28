use rocket::State;
use sqlx::{query, MySqlPool};

use crate::db::get_db;
use crate::model::{RssChannel, RssItem};

pub async fn insert_rss_channel(
    pool: &State<MySqlPool>,
    rss_channel: RssChannel,
) -> Result<u64, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query!(
        "INSERT INTO rss_channel 
            (channel_title, channel_description, channel_link, channel_image_url, channel_language, rss_generator)
            VALUES (?, ?, ?, ?, ?, ?);",
        rss_channel.channel_title,
        rss_channel.channel_description,
        rss_channel.channel_link,
        rss_channel.channel_image_url,
        rss_channel.channel_language,
        rss_channel.rss_generator,
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

pub async fn insert_rss_item(
    pool: &State<MySqlPool>,
    rss_item: RssItem,
) -> Result<u64, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query!(
        "INSERT INTO rss_item 
            (channel_id, rss_title, rss_description, rss_link, rss_author, rss_pub_date)
            VALUES (?, ?, ?, ?, ?, ?)",
        rss_item.channel_id,
        rss_item.rss_title,
        rss_item.rss_description,
        rss_item.rss_link,
        rss_item.rss_creator,
        rss_item.rss_pub_date,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id()),
        Err(e) => {
            eprintln!("Error inserting RSS item: {}", e);
            Err(e)
        }
    }
}
