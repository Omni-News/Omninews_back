use rocket::http::Status;
use std::path::PathBuf;

// catch-all handler
#[options("/<path..>")]
pub fn options_handler(path: PathBuf) -> Status {
    info!("path: {:?}", path);
    Status::NoContent
}
