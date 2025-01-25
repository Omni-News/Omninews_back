use mysql::prelude::{FromRow, Queryable};
use r2d2::{Pool, PooledConnection};
use r2d2_mysql::MySqlConnectionManager;
use rocket::State;
use serde::Serialize;

#[derive(Debug, Serialize, FromRow)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[get("/users")]
pub async fn get_users(pool: &State<Pool<MySqlConnectionManager>>) -> String {
    let mut conn: PooledConnection<MySqlConnectionManager> =
        pool.get().expect("Failed to get connection from pool");

    let result: Vec<User> = conn
        .query("SELECT id, name, email FROM users")
        .expect("Query failed");

    let mut response = String::new();
    for user in result {
        response.push_str(&format!(
            "ID: {}, name: {}, Email: {}\n",
            user.id, user.name, user.email
        ));
    }

    response
}
