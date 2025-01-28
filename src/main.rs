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
use handler::rss_handler::*;

use rocket::routes;

#[launch]
async fn rocket() -> _ {
    config::load_env();
    rocket::build()
        .manage(db::create_pool().await)
        .mount("/", routes![create_rss])
}
