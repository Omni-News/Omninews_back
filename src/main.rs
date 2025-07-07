#[macro_use]
extern crate rocket;

mod auth_middleware;
mod config;
mod db_util;
mod dto;
mod global;
mod handler;
mod model;
mod repository;
mod scheduler;
mod service;
mod utils;

use auth_middleware::{AuthCache, AuthMiddleware};
use handler::{
    error_handler::error_catchers, folder_handler::*, health_handler::*, news_handler::*,
    rss_handler::*, search_handler::*, subscription_handler::*, user_handler::*,
};
use okapi::openapi3::OpenApi;
use rocket::{routes, Build, Rocket};
use rocket_okapi::{
    mount_endpoints_and_merged_docs,
    rapidoc::{make_rapidoc, GeneralConfig, HideShowConfig, RapiDocConfig},
    settings::UrlObject,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use sqlx::MySqlPool;
use utils::embedding_util::EmbeddingService;

#[launch]
async fn rocket() -> _ {
    config::load_env();
    config::configure_logging();

    let pool = db_util::create_pool().await;
    let pool_scheduler = pool.clone();
    let pool_middleware = pool.clone();

    tokio::spawn(async move {
        start_scheduler(&pool_scheduler).await;
    });

    let embedding_service = EmbeddingService::new();

    let exempt_paths = vec![
        "/user/login".to_string(),
        "/rss/all".to_string(),
        "/user/refresh-token".to_string(),
        "/rapidoc/".to_string(),
        "/swagger-ui/".to_string(),
        // TODO 이거 잘 만들어보기. /aa , /aa/ 이렇게 구분. 끝 슬래시면 하위 포함
        "/".to_string(),
    ];
    let mut rocket = rocket::build()
        .manage(pool)
        .manage(embedding_service)
        .manage(AuthCache::new())
        .attach(AuthMiddleware::new(exempt_paths, pool_middleware))
        .mount(
            "/",
            routes![
                // rss
                create_rss_all,
                create_channel,
                get_rss_list,
                get_channel_list,
                get_recommend_channel,
                get_recommend_item,
                get_rss_item_by_channel_id,
                get_rss_preview,
                is_rss_exist,
                get_rss_channel_by_id,
                update_rss_item_rank,
                get_channel_id_by_rss_link,
                // news
                get_news,
                get_news_by_api,
                // news fetch
                fetch_start,
                fetch_stop,
                // account
                login,
                logout,
                verify_access_token,
                verify_refresh_token,
                // Subscription
                subscribe_channel,
                get_subscribe_items,
                unsubscribe_channel,
                validate_already_subscribe_channel,
                get_subscribe_channels,
                // folder
                create_folder,
                add_channel_to_folder,
                find_folders,
                update_folder,
                delete_folder,
                delete_channel_from_folder,
                // system
                health_check,
            ],
        )
        .register("/", error_catchers())
        .mount(
            "/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                title: Some("OmniNews documentation | RapiDoc".to_owned()),
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../v1/openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../v1/openapi.json".to_owned(), // openapi spec 경로
                ..Default::default()
            }),
        );

    let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();
    let custom_route_spec = (vec![], custom_openapi_spec());
    mount_endpoints_and_merged_docs! {
        rocket, "/v1".to_owned(), openapi_settings,
        "/external" => custom_route_spec,
        "/api" => handler::get_routes_and_docs(&openapi_settings),
    };

    rocket
}

async fn start_scheduler(pool: &MySqlPool) {
    use scheduler::{annoy_scheduler::*, news_scheduler::*};

    tokio::join!(
        delete_old_news_scheduler(pool),
        fetch_news_scheduler(pool),
        save_annoy_scheduler(pool)
    );
}

// TODO이거 파일 따로 뺴기
pub fn create_rapidoc_server() -> Rocket<Build> {
    let mut building_rocket = rocket::build().mount(
        "/rapidoc/",
        make_rapidoc(&RapiDocConfig {
            title: Some("OmniNews documentation | RapiDoc".to_owned()),
            general: GeneralConfig {
                spec_urls: vec![UrlObject::new("General", "../v1/openapi.json")],
                ..Default::default()
            },
            hide_show: HideShowConfig {
                allow_spec_url_load: false,
                allow_spec_file_load: false,
                ..Default::default()
            },
            ..Default::default()
        }),
    );

    let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();
    let custom_route_spec = (vec![], custom_openapi_spec());
    mount_endpoints_and_merged_docs! {
        building_rocket, "/v1".to_owned(), openapi_settings,
        "/external" => custom_route_spec,
        "/api" => handler::get_routes_and_docs(&openapi_settings),
    };

    building_rocket
}
fn custom_openapi_spec() -> OpenApi {
    use rocket_okapi::okapi::map;
    use rocket_okapi::okapi::openapi3::*;
    use rocket_okapi::okapi::schemars::schema::*;
    OpenApi {
        openapi: OpenApi::default_version(),
        info: Info {
            title: "The best API ever".to_owned(),
            description: Some("This is the best API ever, please use me!".to_owned()),
            terms_of_service: Some(
                "https://github.com/GREsau/okapi/blob/master/LICENSE".to_owned(),
            ),
            contact: Some(Contact {
                name: Some("okapi example".to_owned()),
                url: Some("https://github.com/GREsau/okapi".to_owned()),
                email: None,
                ..Default::default()
            }),
            license: Some(License {
                name: "MIT".to_owned(),
                url: Some("https://github.com/GREsau/okapi/blob/master/LICENSE".to_owned()),
                ..Default::default()
            }),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            ..Default::default()
        },
        servers: vec![
            Server {
                url: "http://127.0.0.1:1027/".to_owned(),
                description: Some("Localhost".to_owned()),
                ..Default::default()
            },
            Server {
                url: "https://example.com/".to_owned(),
                description: Some("Possible Remote".to_owned()),
                ..Default::default()
            },
        ],
        // Add paths that do not exist in Rocket (or add extra info to existing paths)
        paths: {
            map! {
                "/home".to_owned() => PathItem{
                get: Some(
                    Operation {
                    tags: vec!["HomePage".to_owned()],
                    summary: Some("This is my homepage".to_owned()),
                    responses: Responses{
                        responses: map!{
                        "200".to_owned() => RefOr::Object(
                            Response{
                            description: "Return the page, no error.".to_owned(),
                            content: map!{
                                "text/html".to_owned() => MediaType{
                                schema: Some(SchemaObject{
                                    instance_type: Some(SingleOrVec::Single(Box::new(
                                        InstanceType::String
                                    ))),
                                    ..Default::default()
                                }),
                                ..Default::default()
                                }
                            },
                            ..Default::default()
                            }
                        )
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                    }
                ),
                ..Default::default()
                }
            }
        },
        ..Default::default()
    }
}
