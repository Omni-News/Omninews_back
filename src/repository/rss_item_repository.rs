use log::info;
use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::{
    db::get_db,
    model::rss::{NewRssItem, RssItem},
};

pub async fn select_item_by_link(
    pool: &State<MySqlPool>,
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

pub async fn select_rss_items_by_morpheme_id_order_by_source_rank(
    pool: &State<MySqlPool>,
    morpheme_id: i32,
) -> Result<Vec<RssItem>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssItem,
        "SELECT r.* 
        FROM rss_item r 
        JOIN morpheme_link_mapping m 
        ON r.rss_id = m.rss_id
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

pub async fn select_rss_items_by_morpheme_id_order_by_rss_rank(
    pool: &State<MySqlPool>,
    morpheme_id: i32,
) -> Result<Vec<RssItem>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssItem,
        "SELECT r.* 
        FROM rss_item r 
        JOIN morpheme_link_mapping m 
        ON r.rss_id = m.rss_id
        WHERE m.morpheme_id=?
        ORDER BY r.rss_rank DESC;",
        morpheme_id as i32,
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_rss_items_by_morpheme_id_order_by_pub_date(
    pool: &State<MySqlPool>,
    morpheme_id: i32,
) -> Result<Vec<RssItem>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        RssItem,
        "SELECT r.* 
        FROM rss_item r 
        JOIN morpheme_link_mapping m 
        ON r.rss_id = m.rss_id
        WHERE m.morpheme_id=?
        ORDER BY r.rss_pub_date DESC;",
        morpheme_id as i32,
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_rss_items_order_by_rss_rank(
    pool: &State<MySqlPool>,
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

pub async fn select_rss_items_by_channel_title(
    pool: &State<MySqlPool>,
    channel_title: String,
) -> Result<Vec<RssItem>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let query = format!(
        "SELECT * FROM rss_item r
        WHERE r.channel_id = (SELECT channel_id FROM rss_channel WHERE channel_title={})
        ORDER BY r.rss_pub_date DESC;",
        channel_title
    );

    let result = query_as!(
        RssItem,
        "SELECT * FROM rss_item r
        WHERE r.channel_id = (SELECT channel_id FROM rss_channel WHERE channel_title=?)
        ORDER BY r.rss_pub_date DESC;",
        channel_title,
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn insert_rss_item(
    pool: &State<MySqlPool>,
    rss_item: NewRssItem,
) -> Result<i32, sqlx::Error> {
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

pub async fn update_rss_item(
    pool: &State<MySqlPool>,
    update_rss_item: RssItem,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;
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
        Ok(_) => Ok(update_rss_item.rss_id.unwrap()),
        Err(e) => Err(e),
    }
}
