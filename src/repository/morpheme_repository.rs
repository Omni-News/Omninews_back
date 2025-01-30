use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::{
    db::get_db,
    model::{Morpheme, NewMorpheme},
};

pub async fn select_morpheme_by_id(
    pool: &State<MySqlPool>,
    morpheme_id: String,
) -> Result<Morpheme, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query_as!(
        Morpheme,
        "SELECT * FROM morpheme WHERE morpheme_id=?;",
        morpheme_id,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!("Error selecting morpheme by id: {}", e);
            Err(e)
        }
    }
}

pub async fn select_morpheme_by_word(
    pool: &State<MySqlPool>,
    morpheme_word: String,
) -> Result<Morpheme, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query_as!(
        Morpheme,
        "SELECT * FROM morpheme WHERE morpheme_word=?;",
        morpheme_word,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => {
            eprintln!("Error selecting morpheme by word: {}", e);
            Err(e)
        }
    }
}
pub async fn update_morpheme_by_id(
    pool: &State<MySqlPool>,
    update_morpheme: Morpheme,
) -> Result<u64, sqlx::Error> {
    let mut conn = get_db(pool).await;
    let result = query!(
        "UPDATE morpheme
    SET morpheme_rank=?
    WHERE morpheme_id=?",
        update_morpheme.morpheme_rank,
        update_morpheme.morpheme_id,
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(_) => Ok(update_morpheme.morpheme_id.unwrap() as u64),
        Err(e) => {
            eprintln!("Error updating morpheme by id: {}", e);
            Err(e)
        }
    }
}

pub async fn insert_morpheme(
    pool: &State<MySqlPool>,
    morpheme: NewMorpheme,
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
