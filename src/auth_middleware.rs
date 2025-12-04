use jsonwebtoken::{decode, DecodingKey, Validation};
use okapi::openapi3::{Object, SecurityRequirement, SecurityScheme, SecuritySchemeData};
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{ContentType, Header, Method, Status},
    request::{self, FromRequest, Outcome},
    Data, Request, Response,
};
use rocket_okapi::{
    r#gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::{collections::HashMap, env, io::Cursor};
use std::{collections::HashSet, sync::RwLock};
use uuid::Uuid;

use crate::{server_error, server_info, service::user_service};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub aud: String,     // 발급자
    pub sub: String,     // 사용자 이메일
    pub company: String, // 회사
    pub exp: u64,
}

// 인증 정보를 저장할 구조체
pub struct AuthCache {
    pub auth_failures: RwLock<HashMap<String, String>>,
    pub user_emails: RwLock<HashMap<String, String>>,
}

impl AuthCache {
    pub fn new() -> Self {
        Self {
            auth_failures: RwLock::new(HashMap::new()),
            user_emails: RwLock::new(HashMap::new()),
        }
    }
}

pub struct AuthMiddleware {
    pub exempt_paths: Vec<String>, // JWT 검증을 건너뛸 경로들
    pub pool: MySqlPool,
}

impl AuthMiddleware {
    pub fn new(exempt_paths: Vec<String>, pool: MySqlPool) -> Self {
        Self { exempt_paths, pool }
    }

    // 주어진 경로가 인증 면제 대상인지 확인
    fn is_exempt(&self, path: &str) -> bool {
        self.exempt_paths.iter().any(|exempt| {
            if exempt.ends_with('/') {
                path.starts_with(exempt)
            } else {
                path == exempt
            }
        })
    }
}

// 에러 응답을 JSON으로 직렬화하기 위한 구조체
#[derive(Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

#[allow(clippy::upper_case_acronyms)]
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        // OPTIONS 요청 처리
        if request.method() == Method::Options {
            // OPTIONS 요청은 요청 처리 중단하고 즉시 응답 반환
            request.set_method(Method::Get);
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        // OPTIONS 요청 응답 처리
        if request.method() == Method::Options {
            response.set_status(Status::NoContent);
        }

        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, DELETE, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[rocket::async_trait]
impl Fairing for AuthMiddleware {
    fn info(&self) -> Info {
        Info {
            name: "JWT Authentication",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        let path = req.uri().path().as_str();

        // 요청 ID 생성
        let request_id = format!(
            "{}-{}",
            req.client_ip()
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            Uuid::new_v4()
        );

        // 요청에 고유 ID 저장 (on_response에서 사용하기 위해)
        req.local_cache(|| request_id.clone());

        // OPTIONS 요청이나 면제 경로는 인증 검사 건너뛰기
        if req.method() == Method::Options || self.is_exempt(path) {
            return;
        }

        // path가 home일 경우 검사 스킵
        if path == "/" {
            return;
        }

        let auth_cache = match req.rocket().state::<AuthCache>() {
            Some(cache) => cache,
            None => {
                server_error!("Error: AuthCache not found");
                return;
            }
        };

        // Authorization 헤더에서 토큰 가져오기
        let token = match req.headers().get_one("Authorization") {
            Some(header) if header.starts_with("Bearer ") => &header[7..], // "Bearer " 이후의 문자열
            _ => {
                auth_cache.auth_failures.write().unwrap().insert(
                    request_id,
                    "인증 토큰이 없거나 올바르지 않습니다.".to_string(),
                );
                return;
            }
        };

        let jwt_secret = match env::var("JWT_SECRET_KEY") {
            Ok(secret) => secret,
            Err(_) => {
                server_error!("JWT_SECRET_KEY environment variable not set");
                auth_cache.auth_failures.write().unwrap().insert(
                    request_id,
                    "서버 구성 오류: JWT_SECRET_KEY가 설정되지 않았습니다.".to_string(),
                );
                return;
            }
        };

        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());

        let mut validation = Validation::default();
        validation.validate_aud = true;
        validation.aud = Some(HashSet::from([String::from("omninews")]));

        // JWT 토큰 검증
        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let user_email = token_data.claims.sub;

                // 토큰 DB 검증
                match user_service::vliadate_access_token(
                    &self.pool,
                    token.to_string(),
                    user_email.clone(),
                )
                .await
                {
                    Ok(res) => {
                        if res {
                            // 토큰 검증 성공
                            server_info!("Token validation successful for user: {}", user_email);
                        } else {
                            auth_cache.auth_failures.write().unwrap().insert(
                                request_id.clone(),
                                "사용자 이메일이 잘못되었습니다.".to_string(),
                            );
                        }
                    }
                    Err(_) => {
                        auth_cache
                            .auth_failures
                            .write()
                            .unwrap()
                            .insert(request_id.clone(), "유효하지 않은 토큰입니다.".to_string());
                    }
                };

                // 인증 성공
                auth_cache
                    .user_emails
                    .write()
                    .unwrap()
                    .insert(request_id, user_email);
            }
            Err(e) => {
                server_error!("JWT decode error: {}", e);
                auth_cache
                    .auth_failures
                    .write()
                    .unwrap()
                    .insert(request_id, format!("유효하지 않은 토큰입니다: {e}"));
            }
        };
    }

    #[allow(clippy::redundant_closure)]
    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        // 요청 ID 가져오기
        let request_id = req.local_cache(|| String::new());

        // request_id가 비어있다면 일찍 리턴
        if request_id.is_empty() {
            return;
        }

        if let Some(auth_cache) = req.rocket().state::<AuthCache>() {
            // 인증 실패 메시지 가져오기
            let error_message = {
                let failures = auth_cache.auth_failures.read().unwrap();
                failures.get(request_id).cloned()
            };

            // 인증 실패가 있으면 401 응답으로 변경하고 에러 메시지 추가
            if let Some(message) = error_message {
                res.set_status(Status::Unauthorized);

                // JSON 형식의 에러 메시지 생성
                let error_response = ErrorResponse {
                    status: "권한 없음".to_string(),
                    message,
                };

                // JSON으로 직렬화
                let json = serde_json::to_string(&error_response).unwrap_or_else(|_| {
                    r#"{"status":"권한 없음","message":"인증에 실패했습니다."}"#.to_string()
                });

                // 응답 본문 설정
                res.set_sized_body(json.len(), Cursor::new(json));
                res.set_header(ContentType::JSON);
            }

            // 요청 처리가 끝났으므로 캐시에서 정보 정리
            auth_cache.auth_failures.write().unwrap().remove(request_id);
            auth_cache.user_emails.write().unwrap().remove(request_id);
        }
    }
}

// 인증된 사용자 정보를 간편하게 가져오는 Request Guard
#[derive(JsonSchema)]
pub struct AuthenticatedUser {
    pub user_email: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = &'static str;

    #[allow(clippy::redundant_closure)]
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // 요청 ID 가져오기
        let request_id = req.local_cache(|| String::new());

        // request_id가 비어있다면 권한 없음
        if request_id.is_empty() {
            return Outcome::Error((
                Status::Unauthorized,
                "requiest_id가 없음. 인증되지 않은 요청입니다.",
            ));
        }

        if let Some(auth_cache) = req.rocket().state::<AuthCache>() {
            // 인증된 사용자 이메일 확인
            let user_emails = auth_cache.user_emails.read().unwrap();
            if let Some(user_email) = user_emails.get(request_id).cloned() {
                return Outcome::Success(AuthenticatedUser { user_email });
            }
        }

        // 인증 실패
        Outcome::Error((Status::Unauthorized, "인증되지 않은 요청입니다."))
    }
}

// rapidoc, swagger-ui 전용
#[allow(clippy::needless_lifetimes)]
impl<'a> OpenApiFromRequest<'a> for AuthenticatedUser {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        // Setup global requirement for Security scheme
        let security_scheme = SecurityScheme {
            description: Some("Requires an Bearer token to access.".to_owned()),
            // Setup data requirements.
            // In this case the header `Authorization: mytoken` needs to be set.
            data: SecuritySchemeData::Http {
                scheme: "bearer".to_owned(), // `basic`, `digest`, ...
                // Just gives use a hint to the format used
                bearer_format: Some("bearer".to_owned()),
            },
            extensions: Object::default(),
        };
        // Add the requirement for this route/endpoint
        // This can change between routes.
        let mut security_req = SecurityRequirement::new();
        // Each security requirement needs to be met before access is allowed.
        security_req.insert("HttpAuth".to_owned(), Vec::new());
        // These vvvvvvv-----^^^^^^^^ values need to match exactly!
        Ok(RequestHeaderInput::Security(
            "HttpAuth".to_owned(),
            security_scheme,
            security_req,
        ))
    }
}
