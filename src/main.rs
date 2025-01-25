#[macro_use]
extern crate rocket;

mod config;
mod db;
mod routes;

use rocket::routes;

#[launch]
fn rocket() -> _ {
    use routes::*;
    config::load_env();
    rocket::build()
        .manage(db::create_pool())
        .mount("/", routes![get_users])
}
