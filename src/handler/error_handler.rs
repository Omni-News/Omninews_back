use rocket::{serde::json::Json, Catcher, Request};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    status: u16,
    message: String,
}

#[catch(404)]
fn not_found(req: &Request) -> Json<ErrorResponse> {
    error!("404 Not Found: {}", req.uri());
    Json(ErrorResponse {
        status: 404,
        message: format!("해당 경로 '{}'는 존재하지 않습니다.", req.uri()),
    })
}

#[catch(500)]
fn internal_error(req: &Request) -> Json<ErrorResponse> {
    error!("500 Internal servcer Error: {}", req.uri());
    Json(ErrorResponse {
        status: 500,
        message: "서버 내부 오류가 발생했습니다.".to_string(),
    })
}

pub fn error_catchers() -> Vec<Catcher> {
    catchers![not_found, internal_error]
}
