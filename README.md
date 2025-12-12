# Omninews Backend

Rust + Rocket 기반의 RSS 뉴스 구독 및 관리 백엔드 API 서버입니다.

## About

Omninews는 다양한 RSS 피드를 한 곳에서 관리하고, AI 기반 검색 및 추천 기능을 제공하는 뉴스 서비스입니다.

- **Web**: <https://kang1027.com/omninews>
- **iOS App**: <https://apps.apple.com/kr/app/omninews/id6746567181?l=en-GB>

## Tech Stack

- **Language**: Rust (Edition 2021)
- **Framework**: Rocket 0.5
- **Database**: MySQL (SQLx)
- **Embedding**: rust-bert, Annoy (Approximate Nearest Neighbors)
- **Crawling**: scraper, thirtyfour (Selenium WebDriver)
- **Authentication**: JWT, Social Login (Google, Kakao, Apple)
- **API Documentation**: Swagger UI, RapiDoc
- **Containerization**: Docker, Docker Compose

## Features

- RSS 채널 구독 및 실시간 피드 수집
- 폴더 기반 채널 관리
- 소셜 로그인 (Google, Kakao, Apple)
- AI 기반 뉴스 검색 및 추천 (Sentence Embedding)
- Apple In-App Purchase 구독 관리
- Apple Server Notification 처리
- 프리미엄 RSS 생성 (Instagram 등 크롤링)
- AI 요약 기능 (Gemini API)

## Getting Started

### Prerequisites

- Rust (1.70+)
- Docker & Docker Compose
- MySQL 8.0+

### 1. Git Clone

```bash
git clone <repository-url>
cd Omninews_back
```

### 2. Environment Setup

프로젝트는 **Development**와 **Production** 두 가지 환경을 지원합니다.

#### 환경 변수 파일 구조

- `.env`: Docker Compose용 MySQL 설정 (development/product 환경 주석 처리로 선택)
- `.development.env`: 개발 환경 애플리케이션 환경 변수
- `.product.env`: 배포 환경 애플리케이션 환경 변수

`.env.example`을 복사하여 각 환경에 맞게 설정:

```bash
cp .env.example .env
cp .env.example .development.env
cp .env.example .product.env
```

`.env` 파일에서 사용할 환경을 주석 처리로 선택:

```bash
# for development environment
DATABASE_URL=mysql://user:password@host:3307/omninews_dev
MYSQL_ROOT_PASSWORD=your_password
MYSQL_DATABASE=omninews_dev
MYSQL_USER=user
MYSQL_PASSWORD=your_password

# for production environment (주석 처리)
# DATABASE_URL=mysql://user:password@host:3306/omninews
# MYSQL_ROOT_PASSWORD=your_password
# MYSQL_DATABASE=omninews
# MYSQL_USER=user
# MYSQL_PASSWORD=your_password
```

`.development.env` / `.product.env`에는 애플리케이션 설정:

- JWT 시크릿 키
- 소셜 로그인 API 키 (Apple)
- Naver API 키 (뉴스 검색용)
- Gemini API 키
- Instagram 크롤링 계정
- Selenium WebDriver URL

### 3. Docker Compose Setup

프로젝트는 4개의 Docker Compose 파일을 사용:

- `docker-compose.dev.base.yml`: 개발 환경 기반 서비스 (MySQL, Selenium 등)
- `docker-compose.dev.yml`: 개발 환경 애플리케이션
- `docker-compose.prod.base.yml`: 배포 환경 기반 서비스
- `docker-compose.prod.yml`: 배포 환경 애플리케이션

#### Development 환경

```bash
# 1. .env 파일에서 dev 부분 주석 해제, production 부분 주석 처리

# 2. 네트워크 생성 (최초 1회만)
docker network create omninews-dev-network

# 3. Base 컨테이너 실행 (MySQL, Selenium 등)
docker-compose -f docker-compose.dev.base.yml -p omninews_dev up --build -d

# 4. SQLx 쿼리 준비 (스키마/쿼리 변경 시마다 실행 필요)
# 127.0.0.1:3307은 dev mysql 주소
export DATABASE_URL='mysql://user:password@127.0.0.1:3307/omninews_dev'
cargo sqlx prepare -- --bin OmniNews

# 5. 애플리케이션 컨테이너 실행
docker-compose -f docker-compose.dev.yml -p omninews_dev up --build -d
```

#### Production 환경

```bash
# 1. .env 파일에서 production 부분 주석 해제, dev 부분 주석 처리

# 2. 네트워크 생성 (최초 1회만)
docker network create omninews-prod-network

# 3. Base 컨테이너 실행
docker-compose -f docker-compose.prod.base.yml -p omninews_prod up --build -d

# 4. SQLx 쿼리 준비 (스키마/쿼리 변경 시마다 실행 필요)
# 127.0.0.1:3306은 prod mysql 주소
export DATABASE_URL='mysql://user:password@127.0.0.1:3306/omninews'
cargo sqlx prepare -- --bin OmniNews

# 5. 애플리케이션 컨테이너 실행
docker-compose -f docker-compose.prod.yml -p omninews_prod up --build -d
```

> **중요**: SQLx는 컴파일 타임에 쿼리를 검증하므로, 쿼리나 스키마가 변경되면 반드시 `cargo sqlx prepare`를 다시 실행해야 합니다!

### 4. Database Schema Setup

데이터베이스 스키마는 `src/schema.sql`에 정의되어 있습니다.

```bash
# MySQL에 접속하여 스키마 실행
mysql -u user -p -h 127.0.0.1 -P 3307 omninews_dev < src/schema.sql
```

주요 테이블:

- `user`: 사용자 정보 (소셜 로그인)
- `rss_channel`: RSS 채널 정보
- `rss_item`: RSS 뉴스 아이템
- `embedding`: 뉴스 임베딩 벡터 (검색용)
- `omninews_subscription`: 구독 정보
- `rss_folder`: 채널 폴더
- `user_subscription_channel`: 사용자 채널 구독

### 5. Running & API Documentation

서버 실행 후 다음 URL에서 API 문서를 확인할 수 있습니다:

- **Swagger UI**: `http://localhost:1028/swagger-ui/`
- **RapiDoc**: `http://localhost:1028/rapidoc/`

주요 API 엔드포인트:

- `/v1/api/user/*`: 사용자 관리 (로그인, 회원가입, Apple 로그인)
- `/v1/api/rss/*`: RSS 채널 및 아이템 관리
- `/v1/api/news/*`: 뉴스 조회
- `/v1/api/search/*`: 뉴스 검색 (임베딩 기반)
- `/v1/api/folder/*`: 폴더 관리
- `/v1/api/subscription/*`: 채널 구독 관리
- `/v1/api/omninews-subscription/*`: Omninews 프리미엄 구독
- `/v1/api/apple/*`: Apple Server Notification

### 6. Additional Configuration

#### Selenium WebDriver

프리미엄 RSS 크롤링을 위해 Selenium이 필요합니다.

- Docker Compose base 파일에 selenium 서비스가 포함되어 있습니다
- `.development.env` / `.product.env`에 `SELENIUM_URL_*`, `SCHEDULER_SELENIUM_URL*` 설정

#### Firebase (Optional)

Push Notification을 위해 Firebase Admin SDK 키 파일이 필요합니다:

```bash
omninews_firebase_sdk.json
```

## Project Structure

```
Omninews_back/
├── src/
│   ├── main.rs              # 애플리케이션 진입점
│   ├── auth_middleware.rs   # JWT 인증 미들웨어
│   ├── config/              # 설정 (env, logging, webdriver, swagger)
│   ├── dto/                 # Data Transfer Objects
│   ├── handler/             # API 핸들러
│   ├── model/               # 도메인 모델
│   ├── repository/          # DB 레포지토리
│   ├── service/             # 비즈니스 로직
│   ├── utils/               # 유틸리티 (DB, Embedding)
│   └── schema.sql           # DB 스키마
├── docker-compose.*.yml     # Docker Compose 설정
├── Dockerfile.dev           # 개발용 Dockerfile
├── Dockerfile.prod          # 배포용 Dockerfile
├── Cargo.toml               # Rust 의존성
├── .env                     # MySQL 환경 변수
├── .development.env         # 개발 환경 변수
└── .product.env             # 배포 환경 변수
```

## Development

### Local Build & Run (without Docker)

```bash
# 환경 변수 로드
export DATABASE_URL='mysql://user:password@localhost:3307/omninews_dev'

# SQLx 준비
cargo sqlx prepare -- --bin OmniNews

# Debug 실행
cargo run

# Release 실행
cargo run --release
```

### Building

```bash
# Debug 빌드
cargo build

# Release 빌드
cargo build --release
```

### Testing

```bash
cargo test
```

## Troubleshooting

### SQLx 준비 에러

```bash
error: failed to prepare queries
```

**해결**: DATABASE_URL을 정확히 설정하고 DB가 실행 중인지 확인. 스키마가 적용되어 있는지 확인.

### Docker 네트워크 에러

```bash
network omninews-dev-network not found
```

**해결**: `docker network create omninews-dev-network`

### Port 충돌

- 개발 환경: Port 1028
- 배포 환경: Port 1027 (설정에 따라 다름)

포트가 이미 사용 중이면 `docker-compose.yml`에서 변경하세요.

### 컨테이너 재시작

```bash
# 개발 환경 중지
docker-compose -f docker-compose.dev.yml -p omninews_dev down
docker-compose -f docker-compose.dev.base.yml -p omninews_dev down

# 배포 환경 중지
docker-compose -f docker-compose.prod.yml -p omninews_prod down
docker-compose -f docker-compose.prod.base.yml -p omninews_prod down
```

## License

Proprietary

## Links

- **Web**: <https://kang1027.com/omninews>
- **iOS App**: <https://apps.apple.com/kr/app/omninews/id6746567181?l=en-GB>
