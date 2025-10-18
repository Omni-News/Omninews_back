pub fn load_env() {
    let env_file = if cfg!(debug_assertions) {
        ".development.env"
    } else {
        ".production.env"
    };
    dotenv::from_filename(env_file).ok();
}
