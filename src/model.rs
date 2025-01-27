use mysql::prelude::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, FromRow)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, FromRow)]
#[serde(crate = "rocket::serde")]
pub struct NewUser {
    pub name: String,
    pub email: String,
}
