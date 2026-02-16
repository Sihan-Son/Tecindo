//! # Tecindo 웹 서버 진입점
//!
//! 이 파일은 Tecindo 애플리케이션의 **시작점(entry point)**입니다.
//! Rust 프로그램은 항상 `main()` 함수에서 실행이 시작됩니다.
//!
//! 이 파일이 수행하는 작업:
//! 1. 환경변수(.env) 로딩
//! 2. 로깅(tracing) 초기화
//! 3. SQLite 데이터베이스 연결 풀 생성
//! 4. 데이터베이스 마이그레이션 실행
//! 5. 파일 저장 디렉토리 생성
//! 6. API 라우터 설정
//! 7. HTTP 서버 시작

// ── 모듈 선언 ──
// `mod` 키워드는 다른 파일을 모듈로 가져옵니다.
// 예: `mod config;`는 같은 디렉토리의 `config.rs` 또는 `config/mod.rs`를 가져옵니다.
// Rust에서는 파일 시스템 구조가 곧 모듈 구조입니다.
mod config;
mod db;
mod error;
mod middleware;
mod models;
mod routes;
mod services;

// ── 외부 크레이트 및 모듈에서 필요한 항목 가져오기 ──
// `use` 키워드는 다른 모듈의 항목을 현재 스코프로 가져옵니다.
// Python의 `from X import Y`와 비슷합니다.
use anyhow::Result; // anyhow::Result: 어떤 에러 타입이든 담을 수 있는 범용 Result 타입
use axum::{
    // Axum: Rust의 비동기 웹 프레임워크. Express.js와 비슷한 역할
    routing::{get, patch, post, put, delete}, // HTTP 메서드별 라우팅 함수들
    Router,                                    // 라우터: URL 경로와 핸들러를 연결하는 구조체
};
use config::Config; // 우리가 만든 설정 모듈
use routes::{documents::AppState, *}; // `*`는 모듈의 모든 공개 항목을 가져옴 (glob import)
use sqlx::sqlite::SqlitePoolOptions; // SQLite 연결 풀 설정 옵션
use std::path::Path; // 파일 경로를 다루는 표준 라이브러리 타입
use tower_http::{
    // tower-http: HTTP 미들웨어 모음 크레이트
    cors::{Any, CorsLayer},              // CORS(Cross-Origin Resource Sharing) 설정
    services::{ServeDir, ServeFile},      // 정적 파일 서빙 서비스
    trace::TraceLayer,                    // HTTP 요청/응답 로깅 미들웨어
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt}; // 로깅 초기화 유틸리티

// #[tokio::main]: 비동기 런타임을 시작하는 **어트리뷰트 매크로**
// Rust의 main() 함수는 기본적으로 동기(sync)이므로,
// async/await를 사용하려면 비동기 런타임(Tokio)이 필요합니다.
// 이 매크로가 내부적으로 tokio 런타임을 생성하고 main을 그 안에서 실행합니다.
#[tokio::main]
async fn main() -> Result<()> {
    // ── 1단계: 환경변수 로딩 ──
    // .env 파일에서 환경변수를 읽어옵니다. (예: DATABASE_URL, JWT_SECRET 등)
    // .ok()는 Result를 Option으로 변환하여, .env 파일이 없어도 에러 없이 넘어갑니다.
    dotenvy::dotenv().ok();

    // ── 2단계: 로깅(tracing) 초기화 ──
    // tracing은 Rust 생태계의 표준 로깅 프레임워크입니다.
    // registry(): 로그 수집기를 만들고
    // .with(): 필터와 포맷터를 레이어처럼 쌓아올립니다 (데코레이터 패턴)
    tracing_subscriber::registry()
        .with(
            // EnvFilter: RUST_LOG 환경변수로 로그 레벨을 제어합니다.
            // 환경변수가 없으면 기본값으로 tecindo, tower_http, axum 모듈을 debug 레벨로 설정
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tecindo=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer()) // 로그를 터미널에 출력하는 포맷터 레이어
        .init(); // 전역 로거로 등록

    // ── 3단계: 설정 로딩 ──
    // Config::from_env()로 환경변수에서 설정을 읽어옵니다.
    // `?` 연산자: Result가 Err이면 즉시 함수에서 반환(에러 전파).
    //            Ok이면 내부 값을 꺼냅니다. try-catch 없이 에러를 처리하는 Rust의 방식입니다.
    let config = Config::from_env()?;
    // tracing::info!: 정보 수준의 로그를 출력하는 매크로
    // println!과 비슷하지만, 로그 레벨/타임스탬프/소스 위치 등 메타정보가 포함됩니다.
    tracing::info!("Starting Tecindo server on {}:{}", config.host, config.port);

    // ── 4단계: SQLite 연결 풀 생성 ──
    // 연결 풀(Connection Pool): 데이터베이스 연결을 미리 여러 개 만들어두고 재사용하는 패턴.
    // 매 요청마다 새 연결을 만들면 느리므로, 풀에서 빌려 쓰고 반환합니다.
    // .await: 비동기 작업이 완료될 때까지 기다립니다. (스레드를 블로킹하지 않음)
    let pool = SqlitePoolOptions::new()
        .max_connections(5) // 최대 5개의 동시 연결을 유지
        .connect(&config.database_url) // 데이터베이스에 연결 (비동기)
        .await?; // 연결 실패 시 에러 전파

    // ── 5단계: 데이터베이스 마이그레이션 실행 ──
    // 마이그레이션: 데이터베이스 스키마(테이블 구조)를 코드로 관리하는 방법
    // sqlx::migrate!는 컴파일 타임에 ./migrations 폴더의 SQL 파일들을 포함시키는 매크로
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool) // 아직 실행되지 않은 마이그레이션만 순서대로 실행
        .await?;

    // ── 6단계: 필요한 디렉토리 생성 ──
    // 문서 파일을 저장할 디렉토리가 없으면 생성합니다.
    // Path::new(): 문자열을 파일 경로 타입으로 변환
    let documents_path = Path::new(&config.documents_path);
    if !documents_path.exists() {
        // tokio::fs: 비동기 파일 시스템 작업. std::fs의 비동기 버전입니다.
        // create_dir_all: 중간 디렉토리까지 모두 생성 (mkdir -p와 같음)
        tokio::fs::create_dir_all(documents_path).await?;
        tracing::info!("Created documents directory: {}", config.documents_path);
    }

    // 업로드 파일 저장 디렉토리도 마찬가지로 생성
    let uploads_path = Path::new(&config.uploads_path);
    if !uploads_path.exists() {
        tokio::fs::create_dir_all(uploads_path).await?;
        tracing::info!("Created uploads directory: {}", config.uploads_path);
    }

    // ── 7단계: 애플리케이션 상태(State) 생성 ──
    // AppState: 모든 라우트 핸들러가 공유하는 데이터를 담는 구조체
    // Axum에서는 State를 통해 핸들러에 의존성을 주입합니다.
    // .clone(): 값을 복제합니다. pool과 String은 Clone 트레이트를 구현하므로 복제 가능.
    //           SqlitePool은 내부적으로 Arc(참조 카운트 스마트 포인터)를 사용하므로
    //           clone해도 실제 연결이 복제되지 않고, 같은 풀을 가리킵니다.
    let state = AppState {
        pool: pool.clone(),
        documents_path: config.documents_path.clone(),
        jwt_secret: config.jwt_secret.clone(),
    };

    // ── 8단계: API 라우터 설정 ──
    // Router::new(): 빈 라우터를 생성합니다.
    // .route(): URL 패턴과 핸들러 함수를 연결합니다.
    //           get(), post(), patch() 등은 HTTP 메서드를 지정합니다.

    // 인증 관련 라우트 (회원가입, 로그인, 토큰 갱신, 로그아웃, 내 정보)
    let auth_routes = Router::new()
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/refresh", post(routes::auth::refresh))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/me", get(routes::auth::me));

    // 모든 API 라우트를 하나로 합칩니다.
    let api_routes = Router::new()
        // .merge(): 다른 라우터의 라우트들을 현재 라우터에 합칩니다.
        .merge(auth_routes)
        // 문서(Document) CRUD API
        // .post()를 .route()에 체이닝하면 같은 경로에 여러 HTTP 메서드를 매핑할 수 있습니다.
        .route("/documents", get(list_documents).post(create_document))
        // :id는 URL 경로 파라미터 (Path<String>으로 핸들러에서 추출)
        .route("/documents/:id", get(get_document).patch(update_document).delete(delete_document))
        .route("/documents/:id/content", get(get_document_content).put(update_document_content))
        // 폴더(Folder) CRUD API
        .route("/folders", get(list_folders).post(create_folder))
        .route("/folders/:id", patch(update_folder).delete(delete_folder))
        // 태그(Tag) CRUD API
        .route("/tags", get(list_tags).post(create_tag))
        .route("/tags/:id", patch(update_tag).delete(delete_tag))
        // 문서-태그 관계 API
        .route("/documents/:id/tags", get(get_document_tags).post(add_tag_to_document))
        .route("/documents/:id/tags/:tag_id", delete(remove_tag_from_document))
        // 전문검색(FTS5) API
        .route("/search", get(search))
        // 글쓰기 세션 API
        .route("/documents/:id/sessions", get(list_document_sessions).post(create_writing_session))
        .route("/sessions/:id", patch(end_writing_session))
        // 헬스체크 API (서버 상태 확인용)
        .route("/health", get(health_check))
        // .with_state(): 이 라우터의 모든 핸들러에서 AppState를 사용할 수 있게 합니다.
        .with_state(state);

    // ── 9단계: CORS 미들웨어 설정 ──
    // CORS: 브라우저의 보안 정책. 다른 도메인에서의 API 호출을 허용/차단합니다.
    // 개발 환경에서는 Any(모두 허용)로 설정합니다.
    // 프로덕션에서는 특정 도메인만 허용해야 합니다.
    let cors = CorsLayer::new()
        .allow_origin(Any)  // 모든 출처(origin) 허용
        .allow_methods(Any) // 모든 HTTP 메서드 허용
        .allow_headers(Any); // 모든 헤더 허용

    // ── 10단계: 프론트엔드 정적 파일 서빙 설정 ──
    // 빌드된 프론트엔드 파일이 있으면 같은 서버에서 서빙합니다.
    // SPA(Single Page Application)이므로, 찾을 수 없는 경로는 index.html로 돌려보냅니다.
    let frontend_dist = Path::new("../frontend/dist");
    // if-else가 표현식(expression)으로 사용됩니다.
    // Rust에서는 if-else의 결과를 변수에 바로 대입할 수 있습니다.
    let app = if frontend_dist.exists() {
        tracing::info!("Serving frontend static files from ../frontend/dist");

        // ServeDir: 디렉토리의 파일을 HTTP로 서빙하는 서비스
        // not_found_service: 파일을 찾지 못하면 index.html을 반환 (SPA 라우팅 지원)
        let serve_dir = ServeDir::new("../frontend/dist")
            .not_found_service(ServeFile::new("../frontend/dist/index.html"));

        Router::new()
            // .nest(): API 라우트를 /api/v1 경로 아래에 중첩시킵니다.
            // 예: /documents → /api/v1/documents
            .nest("/api/v1", api_routes)
            // .fallback_service(): API 경로에 매칭되지 않는 모든 요청은 프론트엔드로 전달
            .fallback_service(serve_dir)
            // .layer(): 미들웨어를 추가합니다. 미들웨어는 요청/응답을 가로채서 처리합니다.
            .layer(cors)
            .layer(TraceLayer::new_for_http()) // HTTP 요청/응답 자동 로깅
    } else {
        // 프론트엔드 빌드가 없으면 API만 서빙합니다.
        tracing::warn!("Frontend dist directory not found, serving API only");

        Router::new()
            .nest("/api/v1", api_routes)
            .layer(cors)
            .layer(TraceLayer::new_for_http())
    };

    // ── 11단계: 서버 시작 ──
    // format!: 문자열 포맷팅 매크로. Python의 f-string과 비슷합니다.
    let addr = format!("{}:{}", config.host, config.port);
    // TcpListener: TCP 연결을 수신 대기하는 소켓
    // .bind(): 지정된 주소에 바인딩 (해당 포트에서 요청 대기 시작)
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    // axum::serve(): Axum 서버를 시작하고 요청을 처리합니다.
    // 이 줄에서 서버가 영원히 실행됩니다 (Ctrl+C로 종료할 때까지).
    axum::serve(listener, app).await?;

    // Ok(()): 성공을 나타내는 Result 값. ()는 "빈 값"(unit 타입)입니다.
    Ok(())
}
