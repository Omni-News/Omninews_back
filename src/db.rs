use rocket::State;
use sqlx::pool::PoolOptions;
use sqlx::{mysql::MySql, pool::PoolConnection, MySqlPool};
use std::env;

pub async fn create_pool() -> MySqlPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PoolOptions::new()
        .max_connections(10)
        .min_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create MySQL pool")
}

pub async fn get_db(pool: &State<MySqlPool>) -> PoolConnection<MySql> {
    pool.acquire()
        .await
        .expect("Failed to acquire a connection from the pool")
}
