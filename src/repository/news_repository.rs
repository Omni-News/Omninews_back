use sqlx::{query_as, MySqlPool};

use crate::{db_util::get_db, model::news::News};

pub async fn select_news_by_category(
    pool: &MySqlPool,
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
