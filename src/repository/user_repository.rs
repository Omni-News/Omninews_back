use mysql::{params, prelude::Queryable};
use r2d2::{Pool, PooledConnection};
use r2d2_mysql::MySqlConnectionManager;
use rocket::State;

use crate::model::{NewUser, User};

pub async fn insert_user(
    new_user: NewUser,
    pool: &State<Pool<MySqlConnectionManager>>,
) -> Result<(), ()> {
    let mut conn = get_db(pool);
    conn.exec_drop(
        "INSERT INTO users (name, email) VALUES (:name, :email);",
        params! {
            "name" => new_user.name,
            "email" => new_user.email,
        },
    )
    .unwrap();

    Ok(())
}

pub async fn find_users(pool: &State<Pool<MySqlConnectionManager>>) -> Result<Vec<User>, ()> {
    let mut conn = get_db(pool);

    let users: Vec<User> = conn
        .query_map("SELECT * FROM users", |(id, name, email)| User {
            id,
            name,
            email,
        })
        .unwrap();

    Ok(users)
}

fn get_db(pool: &State<Pool<MySqlConnectionManager>>) -> PooledConnection<MySqlConnectionManager> {
    pool.get().expect("Failed to get connection from pool")
}
