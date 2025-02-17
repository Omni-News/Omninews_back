use rocket::State;
use sqlx::{query, query_as, MySqlPool};

use crate::{
    db::get_db,
    model::morpheme::{Morpheme, NewMorpheme},
};

pub async fn select_morpheme_by_word(
    pool: &State<MySqlPool>,
    morpheme_word: String,
) -> Result<Morpheme, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        Morpheme,
        "SELECT * FROM morpheme WHERE morpheme_word=?;",
        morpheme_word,
    )
    .fetch_one(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_morphemes_by_morpheme(
    pool: &State<MySqlPool>,
    morpheme: String,
) -> Result<Vec<Morpheme>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query_as!(
        Morpheme,
        "SELECT * FROM morpheme 
        WHERE morpheme_word LIKE ?",
        morpheme
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn update_morpheme_by_id(
    pool: &State<MySqlPool>,
    update_morpheme: Morpheme,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;
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
        Ok(_) => Ok(update_morpheme.morpheme_id.unwrap()),
        Err(e) => Err(e),
    }
}

pub async fn insert_morpheme(
    pool: &State<MySqlPool>,
    morpheme: NewMorpheme,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query!(
        "INSERT INTO morpheme (morpheme_word, morpheme_rank) VALUES (?, ?);",
        morpheme.morpheme_word,
        morpheme.morpheme_rank
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => Err(e),
    }
}
