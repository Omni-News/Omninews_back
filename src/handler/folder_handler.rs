use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sqlx::MySqlPool;

use crate::{
    auth_middleware::AuthenticatedUser,
    dto::folder::{
        request::{
            ChannelFolderRequestDto, CreateFolderRequestDto, DeleteFolderRequestDto,
            UpdateFolderRequestDto,
        },
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

/// # 폴더 생성 API
///
/// 사용자가 사용하는 새 폴더를 생성합니다.
///
/// ### `folder_name`: 폴더 이름 (예: "Development")
///
//TODO: 이거 expected Expr뜨는거, okapi 문제니까 0.9.1 업데이트 후 해결되는지 확인하기
//https://github.com/GREsau/okapi/issues/166
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

/// # 폴더에 채널 추가 API
///
/// 폴더에 채널을 추가합니다.
///
/// ### `folder_id`: 폴더 ID (예: 3)
///
/// ### `channel_id`: 채널 ID (예: 2)
///
#[openapi(tag = "Folder")]
#[post("/folder/channel", data = "<channel_folder_id>")]
pub async fn add_channel_to_folder(
    pool: &State<MySqlPool>,
    channel_folder_id: Json<ChannelFolderRequestDto>,
    _auth: AuthenticatedUser,
) -> Result<Status, Status> {
    match folder_service::add_channel_to_folder(pool, channel_folder_id.into_inner()).await {
        Ok(_) => Ok(Status::Created),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 폴더 조회 API
///
/// 사용자의 폴더 목록을 반환합니다.
///
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

/// # 폴더 업데이트 API
///
/// 폴더의 이름을 업데이트합니다.
///
/// ### `folder_id`: 수정할 폴더 ID (예: 3)
///
/// ### `folder_name`: 새 폴더 이름 (예: "Development")
///
#[openapi(tag = "Folder")]
#[put("/folder", data = "<folder>")]
pub async fn update_folder(
    pool: &State<MySqlPool>,
    folder: Json<UpdateFolderRequestDto>,
    _auth: AuthenticatedUser,
) -> Result<Json<i32>, Status> {
    match folder_service::update_folder(pool, folder.into_inner()).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 폴더 삭제 API
///
/// 폴더를 삭제합니다.
///
/// ### `folder_id`: 삭제할 폴더 ID (예: 3)
///
#[openapi(tag = "Folder")]
#[delete("/folder", data = "<folder_id>")]
pub async fn delete_folder(
    pool: &State<MySqlPool>,
    folder_id: Json<DeleteFolderRequestDto>,
    _auth: AuthenticatedUser,
) -> Result<&str, Status> {
    match folder_service::delete_folder(pool, folder_id.into_inner()).await {
        Ok(_) => Ok("Success"),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// # 폴더에서 채널 삭제 API
///
/// 폴더에서 채널을 삭제합니다.
///
/// ### `folder_id`: 폴더 ID (예: 3)
///
/// ### `channel_id`: 삭제할 채널 ID (예: 2)
///
#[openapi(tag = "Folder")]
#[delete("/folder/channel", data = "<channel_folder_id>")]
pub async fn delete_channel_from_folder(
    pool: &State<MySqlPool>,
    channel_folder_id: Json<ChannelFolderRequestDto>,
    _auth: AuthenticatedUser,
) -> Result<&str, Status> {
    match folder_service::delete_channel_from_folder(pool, channel_folder_id.into_inner()).await {
        Ok(_) => Ok("Success"),
        Err(_) => Err(Status::InternalServerError),
    }
}

// // TODO casecade설정ㅎㅏ기
