use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use sqlx::MySqlPool;

use crate::model::{NewticleList, RssChannel, RssItem, UserPrompt};
use crate::service::user_service;

#[post("/prompt", data = "<prompt_value>")]
pub async fn user_prompt(
    pool: &State<MySqlPool>,
    prompt_value: Json<UserPrompt>,
) -> Result<Json<NewticleList>, Status> {
    match user_service::get_newticle_list(pool, prompt_value.into_inner()).await {
        Ok(newticle_list) => Ok(Json(newticle_list)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/prompt/rss", data = "<prompt_rss>")]
pub async fn get_rss_list(
    pool: &State<MySqlPool>,
    prompt_rss: Json<UserPrompt>,
) -> Result<Json<Vec<RssItem>>, Status> {
    match user_service::get_rss_list(pool, prompt_rss.into_inner()).await {
        Ok(rss_list) => Ok(Json(rss_list)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[post("/prompt/channel", data = "<prompt_channel>")]
pub async fn get_channel_list(
    pool: &State<MySqlPool>,
    prompt_channel: Json<UserPrompt>,
) -> Result<Json<Vec<RssChannel>>, Status> {
    match user_service::get_channel_list(pool, prompt_channel.into_inner()).await {
        Ok(channel_list) => Ok(Json(channel_list)),
        Err(_) => Err(Status::InternalServerError),
    }
}
