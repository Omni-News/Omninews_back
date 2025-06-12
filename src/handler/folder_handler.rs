use rocket::{http::Status, serde::json::Json, State};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::AuthenticatedUser,
    model::folder::{ChannelFolderId, FolderId, NewRssFolder, RssFolderResponse, UpdateFolder},
    service::folder_service,
};

#[post("/folder", data = "<folder>")]
pub async fn create_folder(
    pool: &State<MySqlPool>,
    folder: Json<NewRssFolder>,
    user: AuthenticatedUser,
) -> Result<Json<i32>, Status> {
    match folder_service::create_folder(pool, user.user_email, folder.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/folder/channel", data = "<channel_folder_id>")]
pub async fn add_channel_to_folder(
    pool: &State<MySqlPool>,
    channel_folder_id: Json<ChannelFolderId>,
) -> Result<Status, Status> {
    match folder_service::add_channel_to_folder(pool, channel_folder_id.into_inner()).await {
        Ok(_) => Ok(Status::Created),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/folder")]
pub async fn find_folders(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<RssFolderResponse>>, Status> {
    match folder_service::fetch_folders(pool, user.user_email).await {
        Ok(folders) => Ok(Json(folders)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[put("/folder", data = "<folder>")]
pub async fn update_folder(
    pool: &State<MySqlPool>,
    folder: Json<UpdateFolder>,
) -> Result<Json<i32>, Status> {
    match folder_service::update_folder(pool, folder.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[delete("/folder", data = "<folder_id>")]
pub async fn delete_folder(
    pool: &State<MySqlPool>,
    folder_id: Json<FolderId>,
) -> Result<&str, Status> {
    match folder_service::delete_folder(pool, folder_id.into_inner()).await {
        Ok(_) => Ok("Success"),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[delete("/folder/channel", data = "<channel_folder_id>")]
pub async fn delete_channel_from_folder(
    pool: &State<MySqlPool>,
    channel_folder_id: Json<ChannelFolderId>,
) -> Result<&str, Status> {
    match folder_service::delete_channel_from_folder(pool, channel_folder_id.into_inner()).await {
        Ok(_) => Ok("Success"),
        Err(_) => Err(Status::InternalServerError),
    }
}

// // TODO casecade설정ㅎㅏ기
