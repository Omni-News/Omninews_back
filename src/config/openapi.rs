use okapi::openapi3::OpenApi;

use crate::CURRENT_VERSION;

pub fn custom_openapi_spec() -> OpenApi {
    use rocket_okapi::okapi::map;
    use rocket_okapi::okapi::openapi3::*;
    use rocket_okapi::okapi::schemars::schema::*;
    OpenApi {
        openapi: OpenApi::default_version(),
        info: Info {
            title: "The Omninews API specs".to_owned(),
            description: Some("This is a API specs of Omninews.".to_owned()),
            terms_of_service: Some("https://github.com/kang1027".to_owned()),
            contact: Some(Contact {
                name: Some("Omninews".to_owned()),
                url: Some("https://github.com/kang1027".to_owned()),
                email: None,
                ..Default::default()
            }),
            license: Some(License {
                name: "MIT".to_owned(),
                // TODO 라이센스 등 버전 수정.
                url: Some("https://github.com/GREsau/okapi/blob/master/LICENSE".to_owned()),
                ..Default::default()
            }),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            ..Default::default()
        },
        servers: vec![
            Server {
                url: format!("http://127.0.0.1:1027/{CURRENT_VERSION}").to_owned(),
                description: Some("Localhost".to_owned()),
                ..Default::default()
            },
            Server {
                url: format!("http://61.253.113.42:1027/{CURRENT_VERSION}").to_owned(),
                description: Some("Possible Remote".to_owned()),
                ..Default::default()
            },
        ],
        // Add paths that do not exist in Rocket (or add extra info to existing paths)
        // swagger, rapidoc에 국한되어 부가설명 가능. 굳이 사용 안할듯?
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
