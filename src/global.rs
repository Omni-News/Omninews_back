use std::sync::Mutex;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref FETCH_FLAG: Mutex<bool> = Mutex::new(true);
}
