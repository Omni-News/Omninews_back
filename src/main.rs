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

use handler::{rss_handler::*, search_handler::*};
use rocket::routes;

#[launch]
async fn rocket() -> _ {
    config::load_env();
    rocket::build()
        .manage(db::create_pool().await)
        .mount("/", routes![create_rss, get_rss_list, get_channel_list])
}
