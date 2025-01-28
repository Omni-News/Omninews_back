use rocket::State;
use sqlx::{query, MySqlPool};

use crate::{db::get_db, model::Morpheme};

pub async fn insert_morpheme(
    pool: &State<MySqlPool>,
    morpheme: Morpheme,
) -> Result<u64, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query!(
        "INSERT INTO morpheme (morpheme_word, morpheme_rank) VALUES (?, ?);",
        morpheme.morpheme_word,
        morpheme.morpheme_rank
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id()),
        Err(e) => {
            eprintln!("Error inserting Morpheme: {}", e);
            Err(e)
        }
    }
}
