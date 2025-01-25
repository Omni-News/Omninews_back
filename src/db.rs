use mysql::*;
use r2d2::Pool;
use r2d2_mysql::MySqlConnectionManager;
use std::env;

pub fn create_pool() -> Pool<MySqlConnectionManager> {
    let user = env::var("MYSQL_USER").unwrap();
    let pw = env::var("MYSQL_PW").unwrap();
    let db = env::var("DATABASE_NAME").unwrap();

    let opts = OptsBuilder::new()
        .ip_or_hostname(Some("localhost"))
        .user(Some(user))
        .pass(Some(pw))
        .db_name(Some(db))
        .tcp_port(3306);

    let manager = MySqlConnectionManager::new(opts);

    Pool::builder()
        .max_size(10)
        .min_idle(Some(5))
        .build(manager)
        .expect("Failed to create Pool")
}
