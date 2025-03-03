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

#[launch]
async fn rocket() -> _ {
    config::load_env();
    config::configure_logging();

    let pool = db::create_pool().await;
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        use scheduler::news_scheduler::*;

        tokio::join!(
            delete_old_news_scheduler(&pool_clone),
            fetch_news_scheduler(&pool_clone)
        );
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
            ],
        )
        .register("/", error_catchers())
}
