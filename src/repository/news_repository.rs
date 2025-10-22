use sqlx::{query_as, MySqlPool};

use crate::{db_util::get_db, model::news::News};

pub async fn select_news_by_category(
    pool: &MySqlPool,
    category: &str,
    size: i32,
    offset: i32,
) -> Result<Vec<News>, sqlx::Error> {
    let mut conn = get_db(pool).await?;

    let result = query_as!(
        News,
        "SELECT * FROM news WHERE news_category=? ORDER BY news_pub_date DESC LIMIT ? OFFSET ?",
        category,
        size,
        offset
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}
