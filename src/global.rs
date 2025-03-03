use lazy_static::lazy_static;
use tokio::sync::Mutex;

lazy_static! {
    pub static ref FETCH_FLAG: Mutex<bool> = Mutex::new(false);
}
