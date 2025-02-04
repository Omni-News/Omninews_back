use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::{db::get_db, model::morpheme::MorphemeLinkMapping};

pub async fn select_morphemes_link_by_morpheme_id(
    pool: &State<MySqlPool>,
    morpheme_id: i32,
) -> Result<Vec<MorphemeLinkMapping>, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query_as!(
        MorphemeLinkMapping,
        "SELECT * FROM morpheme_link_mapping WHERE morpheme_id=? ORDER BY source_rank desc",
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

pub async fn insert_morpheme_link_mapping(
    pool: &State<MySqlPool>,
    morpheme_link_mapping: MorphemeLinkMapping,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await;

    let result = query!(
        "INSERT INTO morpheme_link_mapping (morpheme_id, channel_id, rss_id, source_link, source_rank) VALUES (?, ?, ?, ?, ? );",
        morpheme_link_mapping.morpheme_id,
        morpheme_link_mapping.channel_id,
        morpheme_link_mapping.rss_id,
        morpheme_link_mapping.source_link,
        morpheme_link_mapping.source_rank,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => {
            eprintln!("Error inserting newticle link: {}", e);
            Err(e)
        }
    }
}
