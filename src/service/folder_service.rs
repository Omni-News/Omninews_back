use sqlx::MySqlPool;

use crate::{
    dto::folder::{
        request::{
            ChannelFolderRequestDto, CreateFolderRequestDto, DeleteFolderRequestDto,
            UpdateFolderRequestDto,
        },
        response::RssFolderResponseDto,
    },
    folder_error,
    model::error::OmniNewsError,
    repository::folder_repository,
};

use super::user_service;

pub async fn create_folder(
    pool: &MySqlPool,
    user_email: String,
    folder: CreateFolderRequestDto,
) -> Result<i32, OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;

    match folder_repository::insert_folder(pool, user_id, folder.folder_name.unwrap()).await {
        Ok(res) => Ok(res),
        Err(e) => {
            folder_error!("[Service] Failed to create folder: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn add_channel_to_folder(
    pool: &MySqlPool,
    channel_folder_id: ChannelFolderRequestDto,
) -> Result<(), OmniNewsError> {
    match folder_repository::insert_channel_to_folder(
        pool,
        channel_folder_id.folder_id.unwrap(),
        channel_folder_id.channel_id.unwrap(),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            folder_error!("[Service] Failed to add channel in folder: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn fetch_folders(
    pool: &MySqlPool,
    user_email: String,
) -> Result<Vec<RssFolderResponseDto>, OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email).await?;
    let mut result: Vec<RssFolderResponseDto> = Vec::new();

    match folder_repository::select_folders(pool, user_id).await {
        Ok(res) => {
            for folder in res {
                match folder_repository::select_channels_in_folder(pool, folder.folder_id.unwrap())
                    .await
                {
                    Ok(channels) => result.push(RssFolderResponseDto::new(
                        folder.folder_id,
                        folder.folder_name,
                        channels,
                    )),
                    Err(e) => {
                        folder_error!("[Service] Failed to fetch channels in folder: {}", e);
                        return Err(OmniNewsError::Database(e));
                    }
                }
            }
            Ok(result)
        }
        Err(e) => {
            folder_error!("[Service] Failed to fetch folders: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn update_folder(
    pool: &MySqlPool,
    folder: UpdateFolderRequestDto,
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
            folder_error!("[Service] Failed to update folder: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn delete_folder(
    pool: &MySqlPool,
    folder_id: DeleteFolderRequestDto,
) -> Result<(), OmniNewsError> {
    let folder_id = folder_id.folder_id.unwrap();
    match folder_repository::delete_folder(pool, folder_id).await {
        Ok(_) => Ok(()),
        Err(e) => {
            folder_error!("[Service] Failed to delete folder: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}

pub async fn delete_channel_from_folder(
    pool: &MySqlPool,
    channel_folder_id: ChannelFolderRequestDto,
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
            folder_error!("[Service] Failed to delete channel from folder: {}", e);
            Err(OmniNewsError::Database(e))
        }
    }
}
