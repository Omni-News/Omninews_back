use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::{
    db::get_db,
    model::news::{NewNews, News},
};

pub async fn select_news_by_category(
    pool: &State<MySqlPool>,
    category: String,
) -> Result<Vec<News>, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query_as!(
        News,
        "SELECT * from news WHERE news_category=? ORDER BY news_pub_date DESC LIMIT 100",
        category,
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_news_by_title(
    pool: &State<MySqlPool>,
    news_title: String,
) -> Result<Option<i32>, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        r#"
        SELECT news_id FROM news WHERE news_title = ?
        "#,
        news_title,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(Some(res.news_id)),
        Err(e) => Err(e),
    }
}

pub async fn insert_news(pool: &State<MySqlPool>, news: NewNews) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query!(
        r#"
        INSERT INTO news (news_title, news_description, news_link, news_source, news_pub_date, news_image_link, news_category)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        news.news_title,
        news.news_description,
        news.news_link,
        news.news_source,
        news.news_pub_date,
        news.news_image_link,
        news.news_category,
    ).execute(&mut *conn).await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => Err(e),
    }
}
