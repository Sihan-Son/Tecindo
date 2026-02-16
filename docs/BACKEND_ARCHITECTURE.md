# Tecindo 백엔드 아키텍처 가이드

> Rust 초보자를 위한 Tecindo 백엔드 구조 설명서

---

## 목차

1. [전체 아키텍처 개요](#1-전체-아키텍처-개요)
2. [디렉토리 & 파일 맵](#2-디렉토리--파일-맵)
3. [핵심 개념 설명](#3-핵심-개념-설명-rust-초보-대상)
4. [데이터 흐름 예시: 문서 생성](#4-데이터-흐름-예시-문서-생성)
5. [주요 타입 & 트레이트 관계도](#5-주요-타입--트레이트-관계도)
6. [확장 가이드](#6-확장-가이드)
7. [의존성 크레이트 가이드](#7-의존성-크레이트-가이드)

---

## 1. 전체 아키텍처 개요

### 요청 처리 흐름

클라이언트에서 HTTP 요청이 들어오면 다음 순서로 처리됩니다:

```
클라이언트 (브라우저)
    │
    ▼
┌─────────────────────────────────────────┐
│  tower-http 미들웨어 레이어              │
│  ┌─────────────┐  ┌──────────────────┐  │
│  │ TraceLayer   │  │ CorsLayer        │  │
│  │ (요청 로깅)  │  │ (CORS 헤더 처리) │  │
│  └─────────────┘  └──────────────────┘  │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│  Axum Router (/api/v1/*)                │
│  URL 패턴 매칭 → 적절한 핸들러로 분배    │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│  routes/ (핸들러 계층)                   │
│  - 요청 파라미터 추출 (Extractor)        │
│  - 비즈니스 로직 오케스트레이션           │
│  - 응답 JSON 직렬화                      │
└─────────────────────────────────────────┘
    │                    │
    ▼                    ▼
┌──────────────┐  ┌──────────────────┐
│  db/ 계층     │  │  services/ 계층  │
│  SQLite 쿼리  │  │  파일 I/O 처리   │
│  (sqlx)       │  │  (.md 읽기/쓰기) │
└──────────────┘  └──────────────────┘
    │                    │
    ▼                    ▼
 SQLite DB          디스크 (.md 파일)
```

### 레이어 구조

Tecindo 백엔드는 **4개의 레이어**로 나뉩니다:

| 레이어 | 위치 | 역할 |
|--------|------|------|
| **진입점** | `main.rs` | 서버 초기화, 라우터 조립, 미들웨어 등록 |
| **라우트** | `routes/` | HTTP 요청/응답 처리, 파라미터 추출, 핸들러 함수 |
| **데이터 접근** | `db/` | SQLite 쿼리 실행, CRUD 작업 |
| **서비스** | `services/` | 파일 I/O, 유틸리티 함수 (마크다운 읽기/쓰기, 글자 수 세기) |

이 외에 레이어를 가로지르는 **공통 모듈**이 있습니다:

| 모듈 | 위치 | 역할 |
|------|------|------|
| **모델** | `models/` | 데이터 구조체 (Document, Folder, User 등) |
| **에러** | `error.rs` | 통합 에러 타입 |
| **설정** | `config.rs` | 환경변수 로딩 |
| **미들웨어** | `middleware/` | JWT 인증 처리 |

---

## 2. 디렉토리 & 파일 맵

```
backend/src/
├── main.rs                 # 서버 시작점: 라우터 조립, DB 연결, 미들웨어 등록
├── config.rs               # Config 구조체: 환경변수(.env)에서 설정값 로딩
├── error.rs                # AppError 열거형: 통합 에러 타입 + HTTP 응답 변환
│
├── models/                 # 데이터 구조체 (DB 행 ↔ Rust 구조체 ↔ JSON)
│   ├── mod.rs              #   모듈 재내보내기 (pub use document::*, user::*)
│   ├── document.rs         #   Document, Folder, 요청/응답 DTO
│   ├── user.rs             #   User, UserResponse, 인증 요청 DTO
│   └── tag.rs              #   Tag, 태그 관련 요청 DTO (아직 미사용)
│
├── db/                     # SQLite 쿼리 함수들 (데이터 접근 계층)
│   ├── mod.rs              #   모듈 재내보내기
│   ├── documents.rs        #   문서/폴더 CRUD 쿼리
│   └── users.rs            #   사용자/리프레시토큰 쿼리
│
├── routes/                 # HTTP 핸들러 (요청 → 비즈니스 로직 → 응답)
│   ├── mod.rs              #   모듈 재내보내기
│   ├── documents.rs        #   AppState 정의 + 문서 CRUD 핸들러
│   ├── folders.rs          #   폴더 CRUD 핸들러
│   ├── auth.rs             #   회원가입/로그인/토큰갱신/로그아웃 핸들러
│   └── health.rs           #   헬스체크 핸들러
│
├── services/               # 비즈니스 로직 & 유틸리티
│   ├── mod.rs              #   모듈 재내보내기
│   └── markdown.rs         #   .md 파일 읽기/쓰기, 글자 수 세기, 파일 경로 생성
│
└── middleware/             # 미들웨어 (요청 전처리)
    ├── mod.rs              #   모듈 재내보내기
    └── auth.rs             #   JWT 토큰 생성/검증, AuthUser Extractor, Claims 구조체
```

### 파일 간 의존 관계

```
main.rs
  ├── config.rs          (Config::from_env)
  ├── routes/*           (핸들러 함수들, AppState)
  └── sqlx               (DB 풀 생성, 마이그레이션)

routes/documents.rs
  ├── db/documents.rs    (문서/폴더 쿼리 호출)
  ├── services/markdown.rs (파일 I/O)
  ├── models/*           (Document, Folder, 요청 DTO)
  └── error.rs           (AppError 반환)

routes/auth.rs
  ├── db/users.rs        (사용자/토큰 쿼리)
  ├── middleware/auth.rs  (토큰 생성/검증 유틸)
  ├── models/user.rs     (User, 인증 DTO)
  └── error.rs           (AppError 반환)

routes/folders.rs
  ├── db/documents.rs    (폴더 쿼리 - documents.rs에 함께 위치)
  ├── models/*           (Folder, 요청 DTO)
  └── error.rs

middleware/auth.rs
  └── routes/documents.rs (AppState 참조 - JWT 시크릿 접근)

db/documents.rs
  ├── models/*           (Document, Folder)
  └── error.rs

db/users.rs
  ├── models/user.rs     (User)
  └── error.rs
```

---

## 3. 핵심 개념 설명 (Rust 초보 대상)

### 3.1 Axum의 동작 원리

#### Router — URL과 핸들러를 연결하는 지도

`Router`는 "어떤 URL로 요청이 오면 어떤 함수를 실행할지"를 정의합니다.

```rust
// main.rs:84-96
let api_routes = Router::new()
    .route("/documents", get(list_documents).post(create_document))
    .route("/documents/:id", get(get_document).patch(update_document).delete(delete_document))
    .route("/health", get(health_check))
    .with_state(state);  // 모든 핸들러가 공유할 상태를 주입
```

- `.route("/documents", get(list_documents))` → `GET /api/v1/documents` 요청이 오면 `list_documents` 함수 실행
- `.post(create_document)` → 같은 URL이지만 `POST` 메서드면 `create_document` 실행
- `.nest("/api/v1", api_routes)` → 모든 API 라우트 앞에 `/api/v1` 접두사 추가

#### Handler — 요청을 받아 응답을 만드는 함수

Axum에서 핸들러는 **일반 async 함수**입니다. 특별한 트레이트를 구현할 필요 없이, 매개변수와 반환 타입의 규칙만 지키면 됩니다.

```rust
// routes/documents.rs:29-37
pub async fn get_document(
    State(state): State<AppState>,    // 매개변수 1: 공유 상태 추출
    Path(id): Path<String>,           // 매개변수 2: URL의 :id 추출
) -> Result<Json<Document>, AppError> { // 반환: JSON 응답 또는 에러
    let document = db::get_document(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(document))
}
```

핵심 규칙:
- 매개변수 = Extractor (요청에서 데이터를 꺼냄)
- 반환 타입 = `IntoResponse`를 구현하는 타입 (자동으로 HTTP 응답으로 변환)

#### Extractor — 요청에서 데이터를 자동으로 꺼내는 도구

Axum은 함수 매개변수의 **타입**을 보고 자동으로 요청에서 데이터를 추출합니다:

| Extractor | 역할 | 예시 |
|-----------|------|------|
| `State(state)` | 앱 공유 상태 (DB 풀, 설정 등) | `State(state): State<AppState>` |
| `Path(id)` | URL 경로의 변수 (`:id`) | `Path(id): Path<String>` |
| `Json(body)` | 요청 본문의 JSON 파싱 | `Json(req): Json<CreateDocumentRequest>` |
| `AuthUser` | JWT에서 사용자 정보 추출 (커스텀) | `auth_user: AuthUser` |

Extractor는 **선언적**입니다. 타입만 적으면 Axum이 알아서 처리합니다:

```rust
// 이 함수 시그니처만으로 Axum은:
// 1. 앱 상태에서 pool, documents_path 등을 꺼내고
// 2. URL에서 :id를 String으로 파싱하고
// 3. 요청 바디를 DocumentContent로 역직렬화합니다
pub async fn update_document_content(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<DocumentContent>,
) -> Result<StatusCode, AppError> { ... }
```

#### State — 모든 핸들러가 공유하는 앱 상태

```rust
// routes/documents.rs:16-20
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,        // DB 커넥션 풀
    pub documents_path: String,  // .md 파일 저장 경로
    pub jwt_secret: String,      // JWT 서명 비밀키
}
```

`AppState`는 `main.rs`에서 한 번 생성되어 `.with_state(state)`로 모든 라우트에 주입됩니다. 핸들러에서 `State(state): State<AppState>`로 꺼내 쓸 수 있습니다.

`#[derive(Clone)]`이 필요한 이유: Axum은 내부적으로 각 요청마다 상태를 복제(clone)합니다. `SqlitePool`과 `String`은 모두 `Clone`을 구현하므로 문제없습니다. (`SqlitePool`은 내부적으로 `Arc`를 사용하여 실제 커넥션은 공유됩니다.)

---

### 3.2 sqlx로 SQLite 다루기

#### Pool — 데이터베이스 커넥션을 효율적으로 관리

데이터베이스에 연결할 때마다 새 연결을 만드는 대신, **풀(Pool)**에 미리 여러 연결을 만들어놓고 재사용합니다.

```rust
// main.rs:44-47
let pool = SqlitePoolOptions::new()
    .max_connections(5)           // 최대 5개 동시 연결
    .connect(&config.database_url) // "sqlite:data/tecindo.db" 같은 URL
    .await?;                       // 비동기로 연결 대기
```

#### query_as — SQL 결과를 Rust 구조체로 매핑

```rust
// db/documents.rs:6-17
let docs = sqlx::query_as::<_, Document>(
    r#"
    SELECT id, folder_id, title, slug, file_path, word_count, char_count,
           excerpt, is_pinned, is_archived, created_at, updated_at
    FROM documents
    ORDER BY is_pinned DESC, updated_at DESC
    "#,
)
.fetch_all(pool)   // 모든 행 가져오기
.await?;           // 비동기 대기 + 에러 전파
```

- `query_as::<_, Document>` — SQL 결과의 각 행을 `Document` 구조체로 자동 변환
- 이것이 가능한 이유: `Document`가 `#[derive(sqlx::FromRow)]`를 구현하고 있기 때문
- `r#"..."#` — Rust의 raw string 리터럴. 이스케이프 없이 SQL을 작성 가능

#### fetch 메서드 종류

| 메서드 | 반환 타입 | 용도 |
|--------|-----------|------|
| `.fetch_all(pool)` | `Vec<T>` | 모든 행 반환 |
| `.fetch_optional(pool)` | `Option<T>` | 0~1개 행 (없으면 `None`) |
| `.fetch_one(pool)` | `T` | 정확히 1개 행 (없으면 에러) |
| `.execute(pool)` | `QueryResult` | INSERT/UPDATE/DELETE (행 데이터 불필요 시) |

#### bind — SQL 파라미터 바인딩 (SQL 인젝션 방지)

```rust
// db/documents.rs:21-31
let doc = sqlx::query_as::<_, Document>("SELECT ... FROM documents WHERE id = ?")
    .bind(id)              // ?에 id 값을 안전하게 바인딩
    .fetch_optional(pool)
    .await?;
```

`.bind()`를 사용하면 SQL 인젝션을 자동으로 방지합니다. 문자열 포맷팅(`format!`)으로 SQL에 값을 직접 넣으면 안 됩니다.

#### Migrations — DB 스키마 버전 관리

```rust
// main.rs:51-53
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

`migrations/` 디렉토리의 `.sql` 파일들을 순서대로 실행합니다:
- `001_initial.sql` — folders, documents, tags, writing_sessions, settings 테이블 생성
- `002_users.sql` — users, refresh_tokens 테이블 생성

이미 실행된 마이그레이션은 건너뜁니다. sqlx가 내부적으로 `_sqlx_migrations` 테이블로 추적합니다.

---

### 3.3 에러 처리 패턴 (thiserror + IntoResponse)

Tecindo은 **하나의 에러 타입**(`AppError`)으로 모든 에러를 통합합니다.

#### AppError 정의 (thiserror)

```rust
// error.rs:9-31
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),      // sqlx::Error → AppError 자동 변환

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),         // io::Error → AppError 자동 변환

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Conflict: {0}")]
    Conflict(String),
    // ...
}
```

- `#[derive(Error)]` — `thiserror` 크레이트가 `std::error::Error` 구현을 자동 생성
- `#[error("...")]` — 에러 메시지 포맷 지정
- `#[from]` — **자동 변환**: `sqlx::Error`가 발생하면 `?` 연산자로 `AppError::Database`로 자동 변환

이것이 핸들러에서 `?`를 쓸 수 있는 이유입니다:

```rust
// ?는 에러 발생 시 자동으로 AppError로 변환하여 반환
let docs = sqlx::query_as::<_, Document>("SELECT ...")
    .fetch_all(pool)
    .await?;   // sqlx::Error → AppError::Database 자동 변환
```

#### IntoResponse 구현 — 에러를 HTTP 응답으로 변환

```rust
// error.rs:33-81
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found", ...),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request", ...),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error", ...),
            // ...
        };

        // 모든 에러를 통일된 JSON 형식으로 응답
        let body = Json(json!({
            "error": {
                "code": code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}
```

이 패턴 덕분에:
1. 핸들러 반환 타입이 `Result<Json<T>, AppError>`로 통일됨
2. 성공 시 `Ok(Json(data))`, 실패 시 `Err(AppError::NotFound)` 식으로 깔끔하게 작성 가능
3. Axum이 `AppError`를 자동으로 HTTP 응답으로 변환

---

### 3.4 인증 흐름 (JWT)

#### 전체 흐름

```
회원가입/로그인
    │
    ▼
┌──────────────────────────────────────────────┐
│ routes/auth.rs::register / login             │
│ 1. 비밀번호를 Argon2id로 해시                 │
│ 2. access_token (15분) 생성                  │
│ 3. refresh_token (7일) 생성                   │
│ 4. refresh_token의 SHA-256 해시를 DB에 저장    │
│ 5. 두 토큰을 클라이언트에게 응답               │
└──────────────────────────────────────────────┘
    │
    ▼
보호된 API 요청 (Authorization: Bearer <access_token>)
    │
    ▼
┌──────────────────────────────────────────────┐
│ middleware/auth.rs::AuthUser (Extractor)      │
│ 1. Authorization 헤더에서 토큰 추출           │
│ 2. "Bearer " 접두사 제거                     │
│ 3. JWT 서명 검증 + 만료 확인                  │
│ 4. claims.sub에서 user_id 추출               │
│ 5. AuthUser { user_id } 반환                 │
└──────────────────────────────────────────────┘
    │
    ▼
핸들러에서 auth_user.user_id로 사용자 식별
```

#### 토큰 생성 (middleware/auth.rs)

```rust
// middleware/auth.rs:90-106
pub fn create_access_token(user_id: &str, secret: &str) -> Result<String, ...> {
    let claims = Claims {
        sub: user_id.to_string(),          // 누구의 토큰인지 (subject)
        iat: now.timestamp(),               // 발급 시각
        exp: (now + Duration::minutes(15)).timestamp(), // 만료: 15분 후
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}
```

- **access_token**: 15분 유효, API 요청 시 사용
- **refresh_token**: 7일 유효, access_token 재발급 시 사용

#### AuthUser Extractor — 커스텀 Extractor

```rust
// middleware/auth.rs:27-50
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // 1. Authorization 헤더 가져오기
        let auth_header = parts.headers.get(AUTHORIZATION)...;
        // 2. "Bearer " 접두사 제거
        let token = auth_header.strip_prefix("Bearer ")...;
        // 3. JWT 검증
        let claims = verify_access_token(token, &state.jwt_secret)?;
        // 4. user_id 추출
        Ok(AuthUser { user_id: claims.sub })
    }
}
```

`FromRequestParts` 트레이트를 구현하면, 핸들러 매개변수에 `AuthUser`를 넣기만 하면 자동으로 인증이 처리됩니다:

```rust
// routes/auth.rs:167-174 — AuthUser를 매개변수에 넣으면 자동 인증
pub async fn logout(
    State(state): State<AppState>,
    auth_user: AuthUser,               // ← 이것만으로 인증 완료!
) -> Result<Json<Value>, AppError> {
    db_users::delete_user_refresh_tokens(&state.pool, &auth_user.user_id).await?;
    Ok(Json(json!({ "message": "Logged out successfully" })))
}
```

#### 비밀번호 해싱 (Argon2id)

```rust
// routes/auth.rs:42-47
let salt = SaltString::generate(&mut OsRng);   // 랜덤 솔트 생성
let argon2 = Argon2::default();                // Argon2id 기본 설정
let password_hash = argon2
    .hash_password(req.password.as_bytes(), &salt)  // 비밀번호 해시
    .to_string();
```

비밀번호는 절대 평문으로 저장하지 않습니다. Argon2id는 현재 가장 안전한 비밀번호 해싱 알고리즘입니다.

---

### 3.5 비동기 프로그래밍 (tokio, async/await)

#### 왜 비동기인가?

웹 서버는 동시에 많은 요청을 처리해야 합니다. 동기 방식에서는 DB 쿼리 응답을 기다리는 동안 스레드가 아무것도 못 합니다. 비동기 방식에서는 기다리는 동안 다른 요청을 처리할 수 있습니다.

#### async/await 기본

```rust
// "이 함수는 비동기입니다" — 즉시 실행되지 않고 Future를 반환
pub async fn list_documents(pool: &SqlitePool) -> Result<Vec<Document>, AppError> {
    let docs = sqlx::query_as::<_, Document>("SELECT ...")
        .fetch_all(pool)
        .await?;    // ← .await: "여기서 결과가 올 때까지 기다림 (다른 작업 처리 가능)"
    Ok(docs)
}
```

- `async fn` — 이 함수는 비동기 함수입니다. 호출하면 `Future`를 반환합니다.
- `.await` — Future의 결과를 기다립니다. 기다리는 동안 tokio가 다른 작업을 실행합니다.
- `?` — 에러가 발생하면 즉시 `Err`로 반환합니다. `.await?`는 "기다린 후 에러 확인"을 의미합니다.

#### tokio — 비동기 런타임

```rust
// main.rs:25-26
#[tokio::main]
async fn main() -> Result<()> { ... }
```

`#[tokio::main]`은 `main` 함수를 tokio 비동기 런타임 위에서 실행합니다. 이것이 없으면 `.await`를 사용할 수 없습니다. tokio는 스레드 풀을 관리하고, `.await` 지점에서 다른 작업으로 전환하는 일을 담당합니다.

파일 I/O도 비동기로 처리합니다:

```rust
// services/markdown.rs:7-8
let content = tokio::fs::read_to_string(&full_path).await?;
// std::fs가 아닌 tokio::fs를 사용하여 파일 읽기도 비동기로 처리
```

---

## 4. 데이터 흐름 예시: 문서 생성

`POST /api/v1/documents`에 `{ "title": "내 첫 글", "folder_id": "abc-123" }` 요청이 올 때의 전체 흐름입니다.

### Step 1: 라우터 매칭 (main.rs:88)

```rust
.route("/documents", get(list_documents).post(create_document))
//                                       ^^^^^^^^^^^^^^^^^^^^
// POST 메서드이므로 create_document 핸들러가 선택됨
```

### Step 2: Extractor가 요청 데이터를 자동 추출 (routes/documents.rs:39-41)

```rust
pub async fn create_document(
    State(state): State<AppState>,            // ① AppState에서 pool, documents_path 추출
    Json(req): Json<CreateDocumentRequest>,    // ② 요청 바디 JSON → CreateDocumentRequest 역직렬화
) -> Result<Json<Document>, AppError> {
```

Axum이 자동으로 처리하는 일:
1. `State(state)` — `.with_state(state)`로 등록한 `AppState`를 꺼냄
2. `Json(req)` — 요청 바디를 읽고 `CreateDocumentRequest`로 JSON 파싱
   - 파싱 실패 시 자동으로 400 Bad Request 응답

### Step 3: 폴더 slug 조회 (routes/documents.rs:46-51)

```rust
let folder_slug = if let Some(folder_id) = &req.folder_id {
    let folder = db::get_folder(&state.pool, folder_id).await?;
    //           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //           db/documents.rs:160-173의 get_folder 함수 호출
    //           SELECT ... FROM folders WHERE id = ?
    folder.map(|f| f.slug)  // Some("my-folder") 또는 None
} else {
    None
};
```

### Step 4: 파일 경로 생성 (services/markdown.rs:35-42)

```rust
let file_path = services::generate_file_path(title, folder_slug.as_deref());
// title="내 첫 글", folder_slug=Some("my-folder") 이면
// → "my-folder/nae-ceos-geul.md"
// folder_slug=None 이면
// → "nae-ceos-geul.md"
```

### Step 5: 빈 마크다운 파일 생성 (services/markdown.rs:11-25)

```rust
services::write_markdown(&state.documents_path, &file_path, "").await?;
// 1. documents_path + file_path 조합 → "data/documents/my-folder/nae-ceos-geul.md"
// 2. 부모 디렉토리가 없으면 생성 (create_dir_all)
// 3. 빈 내용으로 .md 파일 생성
```

### Step 6: DB에 메타데이터 저장 (db/documents.rs:36-62)

```rust
let document = db::create_document(&state.pool, &req, file_path, slug).await?;
// 1. UUIDv7 생성 (시간순 정렬 가능한 고유 ID)
// 2. INSERT INTO documents (id, folder_id, title, slug, file_path) VALUES (?, ?, ?, ?, ?)
// 3. 방금 생성한 행을 SELECT해서 Document 구조체로 반환
```

### Step 7: JSON 응답 반환 (routes/documents.rs:60)

```rust
Ok(Json(document))
// Document 구조체가 serde를 통해 JSON으로 직렬화되어 응답
// → { "id": "...", "title": "내 첫 글", "slug": "nae-ceos-geul", ... }
```

### 에러 발생 시?

어느 단계에서든 `?`로 에러가 전파되면:
- `sqlx::Error` → `AppError::Database` → HTTP 500 + `{ "error": { "code": "database_error", ... } }`
- `std::io::Error` → `AppError::Io` → HTTP 500 + `{ "error": { "code": "io_error", ... } }`

---

## 5. 주요 타입 & 트레이트 관계도

### 핵심 구조체

```
┌─────────────────────────────────────────────────────────────┐
│                        AppState                              │
│  ┌─────────────┐  ┌─────────────────┐  ┌────────────────┐  │
│  │ pool:        │  │ documents_path:  │  │ jwt_secret:    │  │
│  │ SqlitePool   │  │ String           │  │ String         │  │
│  └─────────────┘  └─────────────────┘  └────────────────┘  │
│  derive: Clone                                               │
│  위치: routes/documents.rs:16-20                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                        Document                              │
│  id, folder_id?, title, slug, file_path,                    │
│  word_count, char_count, excerpt?,                           │
│  is_pinned, is_archived, created_at, updated_at              │
│  derive: Debug, Clone, Serialize, Deserialize, FromRow       │
│  위치: models/document.rs:3-17                               │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                         Folder                               │
│  id, parent_id?, name, slug, sort_order,                    │
│  created_at, updated_at                                      │
│  derive: Debug, Clone, Serialize, Deserialize, FromRow       │
│  위치: models/document.rs:19-28                              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                          User                                │
│  id, username, email, password_hash (직렬화 시 숨김),        │
│  created_at, updated_at                                      │
│  derive: Debug, Clone, Serialize, Deserialize, FromRow       │
│  위치: models/user.rs:4-12                                   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                      UserResponse                            │
│  User에서 password_hash를 제외한 응답용 구조체               │
│  impl From<User> for UserResponse로 변환                    │
│  위치: models/user.rs:14-33                                  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                        Claims                                │
│  sub: String (user_id), exp: i64, iat: i64                  │
│  JWT 토큰의 페이로드                                         │
│  derive: Debug, Serialize, Deserialize, Clone                │
│  위치: middleware/auth.rs:15-20                              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                       AuthUser                               │
│  user_id: String                                             │
│  JWT 토큰에서 추출된 인증 정보                               │
│  impl FromRequestParts<AppState> (커스텀 Extractor)         │
│  위치: middleware/auth.rs:22-50                              │
└─────────────────────────────────────────────────────────────┘
```

### 요청 DTO (Data Transfer Object)

| 구조체 | 용도 | 위치 |
|--------|------|------|
| `CreateDocumentRequest` | 문서 생성 요청 (title?, folder_id?) | models/document.rs:30-34 |
| `UpdateDocumentRequest` | 문서 수정 요청 (title?, folder_id?, is_pinned?, is_archived?) | models/document.rs:36-42 |
| `DocumentContent` | 문서 내용 읽기/쓰기 (content) | models/document.rs:44-47 |
| `CreateFolderRequest` | 폴더 생성 요청 (name, parent_id?) | models/document.rs:49-53 |
| `UpdateFolderRequest` | 폴더 수정 요청 (name?, parent_id?, sort_order?) | models/document.rs:55-60 |
| `RegisterRequest` | 회원가입 (username, email, password) | models/user.rs:35-40 |
| `LoginRequest` | 로그인 (username, password) | models/user.rs:42-46 |
| `RefreshRequest` | 토큰 갱신 (refresh_token) | models/user.rs:48-51 |
| `AuthResponse` | 인증 응답 (user, access_token, refresh_token) | models/user.rs:53-58 |

### 트레이트 역할 정리

| 트레이트 | 무엇을 하는가 | 어디에 쓰이는가 |
|----------|---------------|-----------------|
| `Serialize` (serde) | Rust 구조체 → JSON 변환 | `Json(document)`로 응답 시 |
| `Deserialize` (serde) | JSON → Rust 구조체 변환 | `Json(req): Json<CreateDocumentRequest>`로 요청 파싱 시 |
| `FromRow` (sqlx) | DB 행 → Rust 구조체 변환 | `query_as::<_, Document>`로 쿼리 결과 매핑 시 |
| `IntoResponse` (axum) | Rust 타입 → HTTP 응답 변환 | `AppError`, `AuthError`의 에러 응답 생성 시 |
| `FromRequestParts` (axum) | HTTP 요청 → Rust 타입 추출 | `AuthUser` 커스텀 Extractor |
| `Error` (thiserror) | 에러 타입 자동 구현 | `AppError` 열거형 |
| `Clone` | 값 복제 | `AppState` (Axum이 요청마다 복제) |
| `From<T>` (std) | 타입 변환 | `User` → `UserResponse`, `sqlx::Error` → `AppError` |

---

## 6. 확장 가이드

### 6.1 새 API 엔드포인트 추가하기

예시: `GET /api/v1/documents/:id/stats` (문서 통계 API)

**Step 1: 응답 모델 정의 (models/document.rs)**

```rust
#[derive(Debug, Serialize)]
pub struct DocumentStats {
    pub word_count: i64,
    pub char_count: i64,
    pub created_at: String,
    pub updated_at: String,
}
```

**Step 2: DB 쿼리 함수 추가 (db/documents.rs)**

```rust
pub async fn get_document_stats(pool: &SqlitePool, id: &str) -> Result<Option<DocumentStats>, AppError> {
    let stats = sqlx::query_as::<_, DocumentStats>(
        "SELECT word_count, char_count, created_at, updated_at FROM documents WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(stats)
}
```

**Step 3: 핸들러 함수 추가 (routes/documents.rs)**

```rust
pub async fn get_document_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DocumentStats>, AppError> {
    let stats = db::get_document_stats(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(stats))
}
```

**Step 4: 라우트 등록 (main.rs)**

```rust
// 기존 라우트 아래에 추가
.route("/documents/:id/stats", get(get_document_stats))
```

### 6.2 새 테이블 추가하기 (Migration)

**Step 1:** `backend/migrations/` 디렉토리에 새 SQL 파일 생성

파일명 규칙: `{번호}_{설명}.sql` (예: `003_bookmarks.sql`)

```sql
-- 003_bookmarks.sql
CREATE TABLE bookmarks (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(document_id, user_id)
);
```

**Step 2:** 서버를 재시작하면 자동으로 마이그레이션이 실행됩니다 (`main.rs:51-53`의 `sqlx::migrate!` 매크로).

**Step 3:** 해당 테이블에 대한 모델, DB 함수, 라우트를 각각 추가합니다.

### 6.3 인증이 필요한 라우트 만들기

핸들러 함수의 매개변수에 `AuthUser`를 추가하기만 하면 됩니다:

```rust
// 인증 불필요 — 누구나 호출 가능
pub async fn list_documents(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> { ... }

// 인증 필요 — AuthUser Extractor가 JWT를 자동 검증
pub async fn create_bookmark(
    State(state): State<AppState>,
    auth_user: AuthUser,          // ← 이 한 줄만 추가하면 인증 필수!
    Path(document_id): Path<String>,
) -> Result<Json<Bookmark>, AppError> {
    // auth_user.user_id로 현재 로그인한 사용자 식별
    let bookmark = db::create_bookmark(&state.pool, &auth_user.user_id, &document_id).await?;
    Ok(Json(bookmark))
}
```

인증 실패 시 `AuthError`가 자동으로 401 Unauthorized 응답을 반환합니다. 핸들러 코드에는 인증 로직을 작성할 필요가 없습니다.

---

## 7. 의존성 크레이트 가이드

`Cargo.toml`에 정의된 각 크레이트의 역할입니다.

### 웹 프레임워크

| 크레이트 | 버전 | 역할 |
|----------|------|------|
| **axum** | 0.8 | 웹 프레임워크. Router, Handler, Extractor 등 HTTP 서버의 핵심 기능 제공. `macros` 피처로 `#[debug_handler]` 등 디버그 매크로 활성화 |
| **tokio** | 1 | 비동기 런타임. `async/await`를 실행하는 엔진. `full` 피처로 네트워크, 파일 I/O, 타이머 등 모든 기능 활성화 |
| **tower-http** | 0.6 | HTTP 미들웨어 모음. `cors`(Cross-Origin), `trace`(요청 로깅), `fs`(정적 파일 서빙) 피처 사용 |

### 데이터베이스

| 크레이트 | 버전 | 역할 |
|----------|------|------|
| **sqlx** | 0.8 | 비동기 SQL 툴킷. `query_as`로 타입 안전한 쿼리 실행. `runtime-tokio`(tokio 런타임 사용), `sqlite`(SQLite 드라이버) 피처 활성화 |

### 직렬화 / 역직렬화

| 크레이트 | 버전 | 역할 |
|----------|------|------|
| **serde** | 1 | Rust의 직렬화/역직렬화 프레임워크. `derive` 피처로 `#[derive(Serialize, Deserialize)]` 매크로 사용 |
| **serde_json** | 1 | JSON ↔ Rust 변환. `json!()` 매크로로 JSON 값을 인라인 생성 가능 |

### 인증 / 보안

| 크레이트 | 버전 | 역할 |
|----------|------|------|
| **jsonwebtoken** | 9 | JWT 토큰 생성(`encode`)과 검증(`decode`). HS256 알고리즘 사용 |
| **argon2** | 0.5 | 비밀번호 해싱. Argon2id 알고리즘으로 안전하게 비밀번호 저장 |
| **sha2** | 0.10 | SHA-256 해싱. refresh_token을 DB에 저장할 때 해시 처리 |

### 유틸리티

| 크레이트 | 버전 | 역할 |
|----------|------|------|
| **uuid** | 1 | UUID 생성. `v7` 피처로 시간순 정렬 가능한 UUIDv7 생성 (ID로 사용) |
| **chrono** | 0.4 | 날짜/시간 처리. 토큰 만료 시간 계산, 타임스탬프 파싱. `serde` 피처로 JSON 직렬화 지원 |
| **slug** | 0.1 | 문자열을 URL-safe slug로 변환 (예: "내 첫 글" → "nae-ceos-geul") |
| **dotenvy** | 0.15 | `.env` 파일에서 환경변수 로딩 (개발 환경 설정용) |

### 에러 처리

| 크레이트 | 버전 | 역할 |
|----------|------|------|
| **anyhow** | 1 | `main` 함수의 간편한 에러 처리. `Result<()>` 반환 시 모든 에러 타입을 담을 수 있음 |
| **thiserror** | 2 | 커스텀 에러 타입 정의. `#[derive(Error)]`로 `std::error::Error` 자동 구현, `#[from]`으로 자동 변환 |

### 로깅

| 크레이트 | 버전 | 역할 |
|----------|------|------|
| **tracing** | 0.1 | 구조화된 로깅 프레임워크. `tracing::info!()`, `tracing::error!()` 등의 매크로 제공 |
| **tracing-subscriber** | 0.3 | tracing의 출력 설정. `env-filter` 피처로 `RUST_LOG` 환경변수를 통한 로그 레벨 제어 가능 |
