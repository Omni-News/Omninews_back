use sqlx::{query, query_as, MySqlPool};

use crate::{
    db_util::get_db,
    model::rss::{NewRssItem, RssItem},
};

pub async fn select_item_by_link(
    pool: &MySqlPool,
    item_link: String,
) -> Result<RssItem, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssItem,
        "SELECT * FROM rss_item WHERE rss_link=?;",
        item_link,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_rss_item_by_embedding_id(
    pool: &MySqlPool,
    search_value: &str,
    _embedding_id: i32,
) -> Result<RssItem, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    // using embedding
    let _ = "SELECT r.* 
        FROM rss_item r 
        JOIN embedding e 
        ON r.rss_id = e.rss_id
        WHERE e.embedding_id=?;";

    let result = query_as!(
        RssItem,
        "SELECT *
        FROM rss_item
        WHERE rss_title LIKE CONCAT('%', ? ,'%')
        OR rss_description LIKE CONCAT('%', ? ,'%')
        ORDER BY rss_pub_Date DESC;",
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

pub async fn select_rss_items_order_by_rss_rank(
    pool: &MySqlPool,
) -> Result<Vec<RssItem>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssItem,
        "SELECT * FROM rss_item ORDER BY rss_rank DESC
         LIMIT 100;",
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_rss_items_by_channel_id(
    pool: &MySqlPool,
    channel_id: i32,
) -> Result<Vec<RssItem>, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query_as!(
        RssItem,
        "SELECT * FROM rss_item r
        WHERE r.channel_id = ?
        ORDER BY r.rss_pub_date DESC;",
        channel_id,
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn insert_rss_item(pool: &MySqlPool, rss_item: NewRssItem) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query!(
        "INSERT INTO rss_item 
            (channel_id, rss_title, rss_description, rss_link, rss_author, rss_pub_date, rss_rank, rss_image_link)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rss_item.channel_id,
        rss_item.rss_title,
        rss_item.rss_description,
        rss_item.rss_link,
        rss_item.rss_author,
        rss_item.rss_pub_date,
        rss_item.rss_rank,
        rss_item.rss_image_link,
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
    rss_id: i32,
    num: i32,
) -> Result<bool, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query!(
        "UPDATE rss_item
        SET rss_rank = rss_rank + ?
        WHERE rss_id = ?;
        ",
        num,
        rss_id
    )
    .execute(&mut *conn)
    .await?;

    if result.rows_affected() > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}
