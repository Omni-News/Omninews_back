use sqlx::MySqlPool;

use crate::{
    db_util::get_db,
    model::{folder::RssFolder, rss::RssChannel},
};

pub async fn insert_folder(
    pool: &MySqlPool,
    user_id: i32,
    folder_name: String,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = sqlx::query!(
        "INSERT INTO rss_folder (user_id, folder_name) VALUES (?, ?)",
        user_id,
        folder_name
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => Err(e),
    }
}

pub async fn insert_channel_to_folder(
    pool: &MySqlPool,
    folder_id: i32,
    channel_id: i32,
) -> Result<(), sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = sqlx::query!(
        "INSERT INTO channels_in_folder (folder_id, channel_id) VALUES (?, ?)",
        folder_id,
        channel_id
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn select_folders(pool: &MySqlPool, user_id: i32) -> Result<Vec<RssFolder>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = sqlx::query_as!(
        RssFolder,
        "SELECT * FROM rss_folder WHERE user_id = ?",
        user_id
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn select_channels_in_folder(
    pool: &MySqlPool,
    folder_id: i32,
) -> Result<Vec<RssChannel>, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = sqlx::query_as!(
        RssChannel,
        "SELECT rc.* FROM rss_channel rc 
         JOIN channels_in_folder cic ON rc.channel_id = cic.channel_id 
         WHERE cic.folder_id = ?",
        folder_id
    )
    .fetch_all(&mut *conn)
    .await;

    match result {
        Ok(res) => Ok(res),
        Err(e) => Err(e),
    }
}

pub async fn update_folder(
    pool: &MySqlPool,
    folder_id: i32,
    folder_name: String,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = sqlx::query!(
        "UPDATE rss_folder SET folder_name = ? WHERE folder_id = ?",
        folder_name,
        folder_id
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                Ok(folder_id)
            } else {
                Err(sqlx::Error::RowNotFound)
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn delete_folder(pool: &MySqlPool, folder_id: i32) -> Result<(), sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = sqlx::query!("DELETE FROM rss_folder WHERE folder_id = ?", folder_id)
        .execute(&mut *conn)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                Ok(())
            } else {
                Err(sqlx::Error::RowNotFound)
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn delete_channel_from_folder(
    pool: &MySqlPool,
    folder_id: i32,
    channel_id: i32,
) -> Result<(), sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = sqlx::query!(
        "DELETE FROM channels_in_folder WHERE folder_id = ? AND channel_id = ?",
        folder_id,
        channel_id
    )
    .execute(&mut *conn)
    .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                Ok(())
            } else {
                Err(sqlx::Error::RowNotFound)
            }
        }
        Err(e) => Err(e),
    }
}
