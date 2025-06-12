use rocket::State;
use sqlx::MySqlPool;

use crate::{
    model::{
        error::OmniNewsError,
        folder::{ChannelFolderId, FolderId, NewRssFolder, RssFolderResponse, UpdateFolder},
    },
    repository::folder_repository,
};

use super::user_service;

pub async fn create_folder(
    pool: &State<MySqlPool>,
    user_email: String,
    folder: NewRssFolder,
) -> Result<i32, OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;

    match folder_repository::insert_folder(pool, user_id, folder.folder_name.unwrap()).await {
        Ok(res) => Ok(res),
        Err(e) => {
            error!("[Service] Failed to create folder: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn add_channel_to_folder(
    pool: &State<MySqlPool>,
    add_channel_to_folder: ChannelFolderId,
) -> Result<(), OmniNewsError> {
    match folder_repository::insert_channel_to_folder(
        pool,
        add_channel_to_folder.folder_id.unwrap(),
        add_channel_to_folder.channel_id.unwrap(),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("[Service] Failed to add channel in folder: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn fetch_folders(
    pool: &State<MySqlPool>,
    user_email: String,
) -> Result<Vec<RssFolderResponse>, OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;
    let mut result: Vec<RssFolderResponse> = Vec::new();

    match folder_repository::select_folders(pool, user_id).await {
        Ok(res) => {
            for folder in res {
                match folder_repository::select_channels_in_folder(pool, folder.folder_id.unwrap())
                    .await
                {
                    Ok(channels) => result.push(RssFolderResponse {
                        folder_id: folder.folder_id,
                        folder_name: folder.folder_name,
                        folder_channels: Some(channels),
                    }),
                    Err(e) => {
                        error!("[Service] Failed to fetch channels in folder: {}", e);
                        return Err(OmniNewsError::Database(e));
                    }
                }
            }
            Ok(result)
        }
        Err(e) => {
            error!("[Service] Failed to fetch folders: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn update_folder(
    pool: &State<MySqlPool>,
    folder: UpdateFolder,
) -> Result<i32, OmniNewsError> {
    match folder_repository::update_folder(
        pool,
        folder.folder_id.unwrap(),
        folder.folder_name.unwrap(),
    )
    .await
    {
        Ok(res) => Ok(res),
        Err(e) => {
            error!("[Service] Failed to update folder: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn delete_folder(
    pool: &State<MySqlPool>,
    folder_id: FolderId,
) -> Result<(), OmniNewsError> {
    match folder_repository::delete_folder(pool, folder_id.folder_id).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("[Service] Failed to delete folder: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn delete_channel_from_folder(
    pool: &State<MySqlPool>,
    channel_folder_id: ChannelFolderId,
) -> Result<(), OmniNewsError> {
    match folder_repository::delete_channel_from_folder(
        pool,
        channel_folder_id.folder_id.unwrap(),
        channel_folder_id.channel_id.unwrap(),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("[Service] Failed to delete channel from folder: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
