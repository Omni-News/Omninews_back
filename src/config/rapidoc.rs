use crate::CURRENT_VERSION;
use rocket::Route;
use rocket_okapi::{
    rapidoc::{make_rapidoc, GeneralConfig, HideShowConfig, RapiDocConfig, Theme, UiConfig},
    settings::UrlObject,
};

pub fn create_rapidoc() -> impl Into<Vec<Route>> {
    make_rapidoc(&RapiDocConfig {
        title: Some("OmniNews documentation | RapiDoc".to_owned()),
        general: GeneralConfig {
            spec_urls: vec![UrlObject::new(
                "General",
                &format!("../{CURRENT_VERSION}/openapi.json"),
            )],
            ..Default::default()
        },
        ui: UiConfig {
            theme: Theme::Dark,
            ..Default::default()
        },
        hide_show: HideShowConfig {
            allow_spec_url_load: false,
            allow_spec_file_load: false,
            ..Default::default()
        },
        ..Default::default()
    })
}
