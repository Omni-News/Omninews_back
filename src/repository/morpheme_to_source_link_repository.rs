use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::{db::get_db, model::MorphemeToSourceLink};

pub async fn select_morphemes_link_by_morpheme_id(
    pool: &State<MySqlPool>,
    morpheme_id: i32,
) -> Result<Vec<MorphemeToSourceLink>, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query_as!(
        MorphemeToSourceLink,
        "SELECT * FROM morpheme_to_source_link WHERE morpheme_id=? ORDER BY source_rank desc",
        morpheme_id,
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!("Error selecting morpheme link by morpheme id: {}", e);
            Err(e)
        }
    }
}

pub async fn insert_morpheme_to_source_link(
    pool: &State<MySqlPool>,
    morpheme_to_source_link: MorphemeToSourceLink,
) -> Result<u64, sqlx::Error> {
    let mut conn = get_db(pool).await;

    let result = query!(
        "INSERT INTO morpheme_to_source_link (morpheme_id, channel_id, rss_id, news_id, source_link, source_rank) VALUES (?, ?, ?, ?, ?, ?);",
        morpheme_to_source_link.morpheme_id,
        morpheme_to_source_link.channel_id,
        morpheme_to_source_link.rss_id,
        morpheme_to_source_link.news_id,
        morpheme_to_source_link.source_link,
        morpheme_to_source_link.source_rank,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id()),
        Err(e) => {
            eprintln!("Error inserting newticle link: {}", e);
            Err(e)
        }
    }
}
