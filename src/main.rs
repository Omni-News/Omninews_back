#[macro_use]
extern crate rocket;

mod bindings;
mod config;
mod db;
mod morpheme;
mod routes;

use rocket::routes;

#[launch]
fn rocket() -> _ {
    use routes::*;
    morpheme::mecab_test();
    config::load_env();
    rocket::build()
        .manage(db::create_pool())
        .mount("/", routes![get_users])
}
