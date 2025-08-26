use rocket::{catch, catchers, response, response::Responder, Response};
use rocket::{Catcher, Request};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi;
use rocket_okapi::okapi::openapi3::{MediaType, Responses};
use rocket_okapi::okapi::schemars;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::OpenApiError;

// ----- Catchers -------

/// Error messages returned to user
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct MyError {
    /// The title of the error message
    pub err: String,
    /// The description of the error
    pub msg: Option<String>,
    // HTTP Status Code returned
    #[serde(skip)]
    pub http_status_code: u16,
}

#[catch(400)]
fn bad_request() -> MyError {
    MyError {
        err: "Badsadfsd Request".to_owned(),
        msg: Some("The request given is wrongly formatted or data was missing.".to_owned()),
        http_status_code: 400,
    }
}

#[catch(401)]
fn unauthorized() -> MyError {
    MyError {
        err: "Unauthorized".to_owned(),
        msg: Some("The authentication given was incorrect or insufficient.".to_owned()),
        http_status_code: 401,
    }
}

#[catch(404)]
fn not_found() -> MyError {
    MyError {
        err: "Not Found".to_owned(),
        msg: Some("The requested resource was not found.".to_owned()),
        http_status_code: 404,
    }
}

#[catch(500)]
fn internal_error() -> MyError {
    MyError {
        err: "Internal Server Error".to_owned(),
        msg: Some("An unexpected error occurred on the server.".to_owned()),
        http_status_code: 500,
    }
}

pub fn bad_request_response(gen: &mut OpenApiGenerator) -> okapi::openapi3::Response {
    let schema = gen.json_schema::<MyError>();
    okapi::openapi3::Response {
        description: "\
        # 400 Bad Request\n\
        The request given is wrongly formatted or data was missing. \
        "
        .to_owned(),
        content: okapi::map! {
            "application/json".to_owned() => MediaType {
                schema: Some(schema),
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

pub fn unauthorized_response(gen: &mut OpenApiGenerator) -> okapi::openapi3::Response {
    let schema = gen.json_schema::<MyError>();
    okapi::openapi3::Response {
        description: "\
        # 401 Unauthorized\n\
        The authentication given was incorrect or insufficient. \
        "
        .to_owned(),
        content: okapi::map! {
            "application/json".to_owned() => MediaType {
                schema: Some(schema),
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

pub fn not_found_response(gen: &mut OpenApiGenerator) -> okapi::openapi3::Response {
    let schema = gen.json_schema::<MyError>();
    okapi::openapi3::Response {
        description: "\
        # 404 Not Found\n\
        The requested resource was not found. \
        "
        .to_owned(),
        content: okapi::map! {
            "application/json".to_owned() => MediaType {
                schema: Some(schema),
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

pub fn internal_error_response(gen: &mut OpenApiGenerator) -> okapi::openapi3::Response {
    let schema = gen.json_schema::<MyError>();
    okapi::openapi3::Response {
        description: "\
        # 500 Internal Server Error\n\
        An unexpected error occurred on the server. \
        "
        .to_owned(),
        content: okapi::map! {
            "application/json".to_owned() => MediaType {
                schema: Some(schema),
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

impl<'r> Responder<'r, 'static> for MyError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(rocket::http::ContentType::JSON)
            .status(rocket::http::Status::new(self.http_status_code))
            .ok()
    }
}

impl OpenApiResponderInner for MyError {
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use okapi::openapi3::RefOr;
        Ok(Responses {
            responses: okapi::map! {
                "400".to_owned() => RefOr::Object(bad_request_response(gen)),
                "401".to_owned() => RefOr::Object(unauthorized_response(gen)),
                "404".to_owned() => RefOr::Object(not_found_response(gen)),
                "500".to_owned() => RefOr::Object(internal_error_response(gen)),
            },
            ..Default::default()
        })
    }
}
pub fn error_catchers() -> Vec<Catcher> {
    catchers![bad_request, unauthorized, not_found, internal_error]
}
