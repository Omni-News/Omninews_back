#[macro_use]
extern crate rocket;

mod bindings;
mod config;
mod db;
mod global;
mod handler;
mod model;
mod morpheme;
mod repository;
mod scheduler;
mod service;

use handler::{
    error_handler::error_catchers, health_handler::*, news_handler::*, rss_handler::*,
    search_handler::*,
};
use rocket::routes;
use sqlx::MySqlPool;

#[launch]
async fn rocket() -> _ {
    config::load_env();
    config::configure_logging();

    let pool = db::create_pool().await;
    let clone_pool = pool.clone();

    tokio::spawn(async move {
        start_scheduler(&clone_pool).await;
    });

    rocket::build()
        .manage(pool)
        .mount(
            "/",
            routes![
                create_rss_all,
                create_rss,
                get_rss_list,
                get_channel_list,
                fetch_start,
                fetch_stop,
                get_news,
                get_news_by_api,
                health_check,
                get_recommend_channel,
                get_recommend_item,
                get_rss_item_by_channel_title,
                get_rss_preview,
                is_rss_exist,
            ],
        )
        .register("/", error_catchers())
}

async fn start_scheduler(pool: &MySqlPool) {
    use scheduler::news_scheduler::*;

    tokio::join!(delete_old_news_scheduler(pool), fetch_news_scheduler(pool));
}
