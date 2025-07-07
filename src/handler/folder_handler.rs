use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::AuthenticatedUser,
    dto::folder::{
        request::{ChannelFolderRequestDto, CreateFolderRequestDto, UpdateFolderRequestDto},
        response::RssFolderResponseDto,
    },
    service::folder_service,
};

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings:
        create_folder,
        add_channel_to_folder,
        find_folders,
        update_folder,
        delete_folder,
        delete_channel_from_folder
    ]
}

/// # create_folder
///
/// Returns the ID of the newly created folder if successful.
#[openapi(tag = "Folder")]
#[post("/folder", data = "<folder>")]
pub async fn create_folder(
    pool: &State<MySqlPool>,
    folder: Json<CreateFolderRequestDto>,
    user: AuthenticatedUser,
) -> Result<Json<i32>, Status> {
    match folder_service::create_folder(pool, user.user_email, folder.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # add_channel_to_folder
///
/// Returns a status indicating whether the channel was successfully added to the folder.
#[openapi(tag = "Folder")]
#[post("/channel_folder", data = "<channel_folder_id>")]
pub async fn add_channel_to_folder(
    pool: &State<MySqlPool>,
    channel_folder_id: Json<ChannelFolderRequestDto>,
) -> Result<Status, Status> {
    match folder_service::add_channel_to_folder(pool, channel_folder_id.into_inner()).await {
        Ok(_) => Ok(Status::Created),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # find_folders
///
/// Returns a list of folders associated with the authenticated user.
#[openapi(tag = "Folder")]
#[get("/folder")]
pub async fn find_folders(
    pool: &State<MySqlPool>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<RssFolderResponseDto>>, Status> {
    match folder_service::fetch_folders(pool, user.user_email).await {
        Ok(folders) => Ok(Json(folders)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # update_folder
///
/// Returns the ID of the updated folder if successful.
#[openapi(tag = "Folder")]
#[put("/", data = "<folder>")]
pub async fn update_folder(
    pool: &State<MySqlPool>,
    folder: Json<UpdateFolderRequestDto>,
) -> Result<Json<i32>, Status> {
    match folder_service::update_folder(pool, folder.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # delete_folder
///
/// Returns a success message if the folder was successfully deleted.
#[openapi(tag = "Folder")]
#[delete("/", data = "<folder_id>")]
pub async fn delete_folder(pool: &State<MySqlPool>, folder_id: Json<i32>) -> Result<&str, Status> {
    match folder_service::delete_folder(pool, folder_id.into_inner()).await {
        Ok(_) => Ok("Success"),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # delete_channel_from_folder
///
/// Returns a success message if the channel was successfully removed from the folder.
#[openapi(tag = "Folder")]
#[delete("/channel", data = "<channel_folder_id>")]
pub async fn delete_channel_from_folder(
    pool: &State<MySqlPool>,
    channel_folder_id: Json<ChannelFolderRequestDto>,
) -> Result<&str, Status> {
    match folder_service::delete_channel_from_folder(pool, channel_folder_id.into_inner()).await {
        Ok(_) => Ok("Success"),
        Err(_) => Err(Status::InternalServerError),
    }
}

// // TODO casecade설정ㅎㅏ기
