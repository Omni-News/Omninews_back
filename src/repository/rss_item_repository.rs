use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::{
    db::get_db,
    model::{NewRssItem, RssItem},
};

pub async fn select_rss_item_by_link(
    pool: &State<MySqlPool>,
    rss_link: String,
) -> Result<RssItem, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query_as!(
        RssItem,
        "SELECT * FROM rss_item WHERE rss_link=?;",
        rss_link,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!("Error selecting rss item by link: {}", e);
            Err(e)
        }
    }
}

pub async fn insert_rss_item(
    pool: &State<MySqlPool>,
    rss_item: NewRssItem,
) -> Result<u64, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query!(
        "INSERT INTO rss_item 
            (channel_id, rss_title, rss_description, rss_link, rss_author, rss_pub_date, rss_rank)
            VALUES (?, ?, ?, ?, ?, ?, ?)",
        rss_item.channel_id,
        rss_item.rss_title,
        rss_item.rss_description,
        rss_item.rss_link,
        rss_item.rss_author,
        rss_item.rss_pub_date,
        rss_item.rss_rank,
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

pub async fn update_rss_item_by_id(
    pool: &State<MySqlPool>,
    update_rss_item: RssItem,
) -> Result<u64, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query!(
        "UPDATE rss_item
    SET
        channel_id=?,
        rss_title=?,
        rss_description=?,
        rss_link=?,
        rss_author=?,
        rss_pub_date=?,
        rss_rank=?
    WHERE rss_id=?",
        update_rss_item.channel_id,
        update_rss_item.rss_title,
        update_rss_item.rss_description,
        update_rss_item.rss_link,
        update_rss_item.rss_author,
        update_rss_item.rss_pub_date,
        update_rss_item.rss_rank,
        update_rss_item.rss_id,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(_) => Ok(update_rss_item.rss_id.unwrap() as u64),
        Err(e) => {
            eprintln!("Error updating rss item: {}", e);
            Err(e)
        }
    }
}
