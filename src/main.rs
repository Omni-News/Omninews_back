#[macro_use]
extern crate rocket;

mod auth_middleware;
mod config;
mod db_util;
mod global;
mod handler;
mod model;
mod repository;
mod scheduler;
mod service;
mod utils;

use auth_middleware::{AuthCache, AuthMiddleware};
use handler::{
    error_handler::error_catchers, feedback_handler::*, health_handler::*, news_handler::*,
    rss_handler::*, search_handler::*, subscribe_handler::*, user_handler::*,
};
use rocket::routes;
use sqlx::MySqlPool;
use utils::embedding_util::EmbeddingService;

#[launch]
async fn rocket() -> _ {
    config::load_env();
    config::configure_logging();

    let pool = db_util::create_pool().await;
    let pool_scheduler = pool.clone();
    let pool_middleware = pool.clone();

    tokio::spawn(async move {
        start_scheduler(&pool_scheduler).await;
    });

    let embedding_service = EmbeddingService::new();

    let exempt_paths = vec!["/user/login".to_string()];
    rocket::build()
        .manage(pool)
        .manage(embedding_service)
        .manage(AuthCache::new())
        .attach(AuthMiddleware::new(exempt_paths, pool_middleware))
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
                get_rss_item_by_channel_link,
                get_rss_preview,
                is_rss_exist,
                update_rss_channel_rank,
                get_rss_channel_by_link,
                get_subscribe_items,
                update_rss_item_rank,
                create_feedback,
                get_feedbacks,
                login,
                logout,
            ],
        )
        .register("/", error_catchers())
}

async fn start_scheduler(pool: &MySqlPool) {
    use scheduler::{annoy_scheduler::*, news_scheduler::*};

    tokio::join!(
        delete_old_news_scheduler(pool),
        fetch_news_scheduler(pool),
        save_annoy_scheduler(pool)
    );
}
