use crate::CURRENT_VERSION;
use rocket::Route;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

pub fn create_swagger_ui() -> impl Into<Vec<Route>> {
    make_swagger_ui(&SwaggerUIConfig {
        url: format!("../{}/openapi.json", CURRENT_VERSION).to_owned(),
        ..Default::default()
    })
}
