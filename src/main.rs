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

use handler::user_handler::*;
use rocket::routes;

#[launch]
fn rocket() -> _ {
    //morpheme::analyze::analyze_morpheme();
    config::load_env();
    rocket::build()
        .manage(db::create_pool())
        .mount("/", routes![create_user, get_users])
}
