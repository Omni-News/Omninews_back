use dotenv::dotenv;
use std::env;

pub fn load_env() {
    dotenv().ok();
}

#[allow(dead_code)]
pub fn configure_logging() {
    use env_logger::Builder;
    use log::LevelFilter;

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    Builder::new().filter(None, LevelFilter::Info).init();
}
