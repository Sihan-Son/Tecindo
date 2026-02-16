//! # 문서(Document) 라우트 핸들러
//!
//! 문서의 CRUD(생성/조회/수정/삭제)와 내용 읽기/쓰기를 처리하는
//! HTTP 핸들러 함수들입니다.
//!
//! ## 엔드포인트
//! - `GET    /api/v1/documents`             → 문서 목록 조회
//! - `POST   /api/v1/documents`             → 새 문서 생성
//! - `GET    /api/v1/documents/:id`         → 단일 문서 조회
//! - `PATCH  /api/v1/documents/:id`         → 문서 수정 (부분 업데이트)
//! - `DELETE /api/v1/documents/:id`         → 문서 삭제
//! - `GET    /api/v1/documents/:id/content` → 문서 내용(마크다운) 조회
//! - `PUT    /api/v1/documents/:id/content` → 문서 내용 수정
//!
//! ## Axum 핸들러 패턴
//! Axum 핸들러는 **Extractor(추출기)**를 매개변수로 받습니다.
//! Extractor는 HTTP 요청에서 데이터를 자동으로 추출합니다:
//! - `State(state)`: 앱 전역 상태 (DB 풀, 설정 등)
//! - `Path(id)`: URL 경로 파라미터 (예: /documents/:id에서 id)
//! - `Json(body)`: 요청 본문을 JSON으로 파싱하여 구조체로 변환
//!
//! 반환 타입이 `Result<T, AppError>`이면, Axum이 자동으로:
//! - `Ok(T)` → T를 HTTP 응답으로 변환 (IntoResponse 트레이트 사용)
//! - `Err(AppError)` → AppError를 에러 JSON 응답으로 변환

use crate::{
    db,            // 데이터베이스 접근 계층
    error::AppError,
    models::*,     // 데이터 모델 구조체들
    services,      // 비즈니스 로직 (파일 I/O 등)
};
use axum::{
    extract::{Path, State}, // Axum Extractor: 요청에서 데이터 추출
    http::StatusCode,        // HTTP 상태 코드 (200, 204, 404 등)
    Json,                    // JSON 요청/응답 래퍼
};
use serde_json::{json, Value}; // JSON 값 생성 유틸리티
use sqlx::SqlitePool;          // SQLite 연결 풀 타입

// #[derive(Clone)]: AppState가 Clone 트레이트를 구현하게 합니다.
// Axum의 State Extractor는 내부적으로 AppState를 clone하므로 필수입니다.
// SqlitePool은 Arc<내부상태>를 사용하므로 clone해도 실제 풀이 복제되지 않습니다.

/// 애플리케이션 공유 상태
///
/// 모든 요청 핸들러가 `State(state): State<AppState>`로 접근합니다.
/// Axum의 의존성 주입(Dependency Injection) 메커니즘입니다.
#[derive(Clone)]
pub struct AppState {
    /// SQLite 연결 풀 (내부적으로 Arc로 공유)
    pub pool: SqlitePool,
    /// 마크다운 문서 저장 디렉토리 경로
    pub documents_path: String,
    /// JWT 토큰 서명용 비밀키
    pub jwt_secret: String,
}

/// `GET /documents` — 전체 문서 목록을 조회합니다.
///
/// # Extractor
/// - `State(state)`: 구조 분해(destructuring) 패턴으로 AppState를 바로 추출합니다.
///   `State<AppState>`는 Axum이 HTTP 요청 처리 시 자동으로 주입합니다.
///
/// # 반환값
/// `{ "documents": [...] }` 형태의 JSON
pub async fn list_documents(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    // db::list_documents(): DB에서 문서 목록을 가져옵니다.
    // &state.pool: 풀의 참조를 전달 (소유권 이동 없이 빌려줌)
    let documents = db::list_documents(&state.pool).await?;
    // Json(): 값을 JSON HTTP 응답으로 변환하는 Axum 타입
    // json!(): serde_json의 JSON 생성 매크로
    Ok(Json(json!({ "documents": documents })))
}

/// `GET /documents/:id` — 단일 문서를 조회합니다.
///
/// # Extractor
/// - `Path(id)`: URL의 `:id` 부분을 String으로 추출합니다.
///   Path<String>은 `/documents/abc-123`에서 `"abc-123"`을 추출합니다.
pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Document>, AppError> {
    let document = db::get_document(&state.pool, &id)
        .await?
        // .ok_or(): Option이 None이면 지정한 에러를 반환합니다.
        // 문서를 찾지 못하면 404 NotFound 응답이 됩니다.
        .ok_or(AppError::NotFound)?;
    Ok(Json(document))
}

/// `POST /documents` — 새 문서를 생성합니다.
///
/// 빈 마크다운 파일을 디스크에 생성하고, DB에 메타데이터를 저장합니다.
///
/// # Extractor
/// - `Json(req)`: HTTP 요청 본문(body)을 JSON으로 파싱하여
///   `CreateDocumentRequest` 구조체로 변환합니다.
///   Axum이 Content-Type 확인과 파싱을 자동으로 처리합니다.
pub async fn create_document(
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<Document>, AppError> {
    // .as_deref(): Option<String>을 Option<&str>로 변환합니다.
    //   as_deref()은 String → &str 변환(deref)을 Option 안에서 수행합니다.
    // .unwrap_or("Untitled"): None이면 기본 제목 사용
    let title = req.title.as_deref().unwrap_or("Untitled");

    // 폴더에 문서를 넣는 경우, 폴더의 slug를 가져옵니다.
    // if let Some(folder_id) = &req.folder_id: folder_id가 있는 경우에만 실행
    let folder_slug = if let Some(folder_id) = &req.folder_id {
        let folder = db::get_folder(&state.pool, folder_id).await?;
        // .map(|f| f.slug): Option<Folder>의 내부 값을 변환
        //   Some(folder) → Some(folder.slug), None → None
        folder.map(|f| f.slug)
    } else {
        None
    };

    // 파일 경로 생성: 제목과 폴더로부터 "folder-slug/title-slug.md" 형태를 만듭니다.
    // .as_deref(): Option<String> → Option<&str> 변환
    let file_path = services::generate_file_path(title, folder_slug.as_deref());
    // slug::slugify(): 제목을 URL 친화적인 문자열로 변환
    let slug = slug::slugify(title);

    // 빈 마크다운 파일을 디스크에 생성합니다.
    services::write_markdown(&state.documents_path, &file_path, "").await?;

    // DB에 문서 메타데이터를 저장하고, 생성된 문서를 반환합니다.
    let document = db::create_document(&state.pool, &req, file_path, slug).await?;
    Ok(Json(document))
}

/// `PATCH /documents/:id` — 문서 메타데이터를 수정합니다.
///
/// 요청 본문에 포함된 필드만 업데이트합니다 (부분 업데이트).
/// 예: `{ "title": "새 제목" }`으로 제목만 변경 가능
pub async fn update_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<Document>, AppError> {
    let document = db::update_document(&state.pool, &id, &req)
        .await?
        .ok_or(AppError::NotFound)?; // 문서가 없으면 404
    Ok(Json(document))
}

/// `DELETE /documents/:id` — 문서를 삭제합니다.
///
/// DB 레코드와 디스크의 .md 파일을 모두 삭제합니다.
/// 성공 시 HTTP 204 No Content를 반환합니다 (본문 없음).
pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    // 먼저 문서 정보를 가져와 파일 경로를 확인합니다.
    let document = db::get_document(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    // DB에서 문서 레코드를 삭제합니다.
    let deleted = db::delete_document(&state.pool, &id).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }

    // 디스크의 .md 파일도 삭제합니다.
    // let _: 반환값을 무시합니다. 파일이 이미 없어도 에러를 발생시키지 않습니다.
    // tokio::fs::remove_file(): 비동기 파일 삭제
    let file_path = std::path::PathBuf::from(&state.documents_path).join(&document.file_path);
    let _ = tokio::fs::remove_file(file_path).await;

    // StatusCode::NO_CONTENT: HTTP 204 (성공했지만 반환할 본문 없음)
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /documents/:id/content` — 문서의 마크다운 내용을 조회합니다.
///
/// 디스크의 .md 파일을 읽어 JSON으로 반환합니다.
/// 응답: `{ "content": "# 제목\n\n본문..." }`
pub async fn get_document_content(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DocumentContent>, AppError> {
    // DB에서 문서 메타데이터를 가져와 파일 경로를 확인합니다.
    let document = db::get_document(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 디스크에서 마크다운 파일을 읽어옵니다.
    let content = services::read_markdown(&state.documents_path, &document.file_path).await?;
    // DocumentContent { content }: 필드명과 변수명이 같으면 축약 가능
    //   DocumentContent { content: content }와 동일합니다.
    Ok(Json(DocumentContent { content }))
}

/// `PUT /documents/:id/content` — 문서의 마크다운 내용을 수정합니다.
///
/// 디스크 파일을 덮어쓰고, DB의 단어 수/글자 수/미리보기를 업데이트합니다.
/// 또한 전문검색(FTS5) 인덱스도 갱신합니다.
/// 성공 시 HTTP 204 No Content를 반환합니다.
pub async fn update_document_content(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<DocumentContent>,
) -> Result<StatusCode, AppError> {
    let document = db::get_document(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    // FTS 인덱스 업데이트를 위해 이전 내용을 읽어둡니다.
    // .ok(): Result를 Option으로 변환 (에러를 무시하고 None으로 처리)
    let old_content = services::read_markdown(&state.documents_path, &document.file_path)
        .await
        .ok();

    // 새 내용을 디스크 파일에 저장합니다.
    services::write_markdown(&state.documents_path, &document.file_path, &req.content).await?;

    // 단어 수와 글자 수를 계산합니다.
    // as i64: usize(부호 없는 정수)를 i64(부호 있는 정수)로 캐스팅합니다.
    //   SQLite의 INTEGER는 i64에 매핑됩니다.
    let word_count = services::count_words(&req.content) as i64;
    let char_count = services::count_chars(&req.content) as i64;

    // 미리보기(excerpt): 내용의 처음 200자를 추출합니다.
    // 주의: 바이트 단위([..200])이므로 한글 등 멀티바이트 문자에서 패닉할 수 있습니다.
    let excerpt = if req.content.len() > 200 {
        Some(req.content[..200].to_string())
    } else {
        Some(req.content.clone()) // .clone(): 내용 전체를 복제
    };

    // DB의 문서 메타데이터(단어 수, 글자 수, 미리보기, 수정일)를 업데이트합니다.
    sqlx::query(
        r#"
        UPDATE documents
        SET word_count = ?, char_count = ?, excerpt = ?,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        WHERE id = ?
        "#,
        // ↑ SQL: 4개의 ?에 아래 bind로 순서대로 값을 대입합니다.
        //   strftime(): SQLite의 날짜/시간 포맷 함수
        //   'now': 현재 시각 (UTC)
    )
    .bind(word_count)
    .bind(char_count)
    .bind(excerpt)
    .bind(&id)
    .execute(&state.pool) // &state.pool: SqlitePool의 참조를 전달
    .await?;

    // FTS5(전문검색) 인덱스를 갱신합니다.
    // 검색 기능이 최신 내용을 반영할 수 있도록 합니다.
    // .as_deref(): Option<String> → Option<&str> 변환
    db::index_document(
        &state.pool,
        &id,
        &document.title,
        &req.content,
        Some(&document.title),
        old_content.as_deref(),
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
