use r2d2::Pool;
use r2d2_mysql::MySqlConnectionManager;
use rocket::State;

use crate::{
    model::{NewUser, User},
    repository::user_repository,
};

pub async fn create_user(
    new_user: NewUser,
    pool: &State<Pool<MySqlConnectionManager>>,
) -> Result<(), ()> {
    user_repository::insert_user(new_user, pool)
        .await
        .map_err(|_| ())
}

pub async fn get_users(pool: &State<Pool<MySqlConnectionManager>>) -> Result<Vec<User>, ()> {
    user_repository::find_users(pool).await.map_err(|_| ())
}
