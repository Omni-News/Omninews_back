#[macro_use]
extern crate rocket;

mod bindings;
mod config;
mod db;
mod handler;
mod model;
mod morpheme;
mod repository;
mod service;

use handler::{error_handler::error_catchers, news_handler::*, rss_handler::*, search_handler::*};
use rocket::routes;

#[launch]
async fn rocket() -> _ {
    config::load_env();
    config::configure_logging();
    rocket::build()
        .manage(db::create_pool().await)
        .mount(
            "/",
            routes![
                create_rss_all,
                create_rss,
                get_rss_list,
                get_channel_list,
                create_news,
                get_news,
            ],
        )
        .register("/", error_catchers())
}
