//! # 태그 API 라우트 핸들러
//!
//! 태그 CRUD 및 문서-태그 관계 관리를 위한 HTTP 핸들러 함수들입니다.
//!
//! ## 엔드포인트 목록
//! | 메서드 | 경로 | 핸들러 | 설명 |
//! |--------|------|--------|------|
//! | GET | /api/v1/tags | `list_tags` | 전체 태그 목록 |
//! | POST | /api/v1/tags | `create_tag` | 새 태그 생성 |
//! | PATCH | /api/v1/tags/:id | `update_tag` | 태그 수정 |
//! | DELETE | /api/v1/tags/:id | `delete_tag` | 태그 삭제 |
//! | GET | /api/v1/documents/:id/tags | `get_document_tags` | 문서의 태그 목록 |
//! | POST | /api/v1/documents/:id/tags | `add_tag_to_document` | 문서에 태그 추가 |
//! | DELETE | /api/v1/documents/:id/tags/:tag_id | `remove_tag_from_document` | 문서에서 태그 제거 |
//!
//! ## Axum 핸들러 패턴
//! 각 함수는 Axum의 **추출자(Extractor)** 패턴을 따릅니다:
//! - `State(state)`: 애플리케이션 공유 상태 (DB 풀 등)
//! - `Path(id)`: URL 경로의 변수 (`:id` 부분)
//! - `Json(req)`: 요청 본문을 구조체로 파싱

// ── 의존성 가져오기 ──
use crate::{
    db,                          // 데이터베이스 쿼리 모듈
    error::AppError,             // 에러 타입 (자동으로 HTTP 에러 응답으로 변환됨)
    models::*,                   // 요청/응답 구조체들 (Tag, CreateTagRequest 등)
    routes::documents::AppState, // 애플리케이션 공유 상태 (DB 풀, 설정 등)
};
use axum::{
    extract::{Path, State}, // Axum 추출자: URL 파라미터, 앱 상태 추출
    http::StatusCode,       // HTTP 상태 코드 (204 No Content 등)
    Json,                   // JSON 요청/응답 처리
};
use serde_json::{json, Value}; // JSON 객체 생성용 매크로와 범용 JSON 타입

/// 전체 태그 목록을 조회합니다.
///
/// `GET /api/v1/tags` → `{ "tags": [...] }`
///
/// Axum에서 핸들러의 반환 타입이 `Result<Json<Value>, AppError>`이면:
/// - 성공(Ok): JSON 응답을 200 상태로 반환
/// - 실패(Err): AppError가 자동으로 적절한 HTTP 에러 응답으로 변환됨
pub async fn list_tags(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let tags = db::list_tags(&state.pool).await?;
    // json! 매크로: Rust 값을 JSON Value로 변환합니다
    // { "tags": [...] } 형태의 응답을 생성
    Ok(Json(json!({ "tags": tags })))
}

/// 새 태그를 생성합니다.
///
/// `POST /api/v1/tags` + `{ "name": "...", "color": "..." }`
///
/// `Json(req)`: 요청 본문(body)의 JSON을 `CreateTagRequest` 구조체로 자동 파싱합니다.
/// 파싱 실패 시(잘못된 JSON, 필수 필드 누락 등) Axum이 자동으로 400 에러를 반환합니다.
pub async fn create_tag(
    State(state): State<AppState>,
    Json(req): Json<CreateTagRequest>,
) -> Result<Json<Tag>, AppError> {
    let tag = db::create_tag(&state.pool, &req).await?;
    Ok(Json(tag))
}

/// 태그 정보를 수정합니다.
///
/// `PATCH /api/v1/tags/:id` + `{ "name": "...", "color": "..." }`
///
/// `Path(id)`: URL의 `:id` 부분을 String으로 추출합니다.
/// 예: `/api/v1/tags/abc-123` → id = "abc-123"
pub async fn update_tag(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTagRequest>,
) -> Result<Json<Tag>, AppError> {
    let tag = db::update_tag(&state.pool, &id, &req)
        .await?
        // ok_or(): Option<Tag>을 Result<Tag, AppError>로 변환
        // None(태그 없음)이면 NotFound 에러 → HTTP 404 응답
        .ok_or(AppError::NotFound)?;
    Ok(Json(tag))
}

/// 태그를 삭제합니다.
///
/// `DELETE /api/v1/tags/:id` → `204 No Content`
///
/// 삭제 성공 시 본문 없이 204 상태 코드만 반환합니다.
/// REST API에서 DELETE 성공 시 204를 반환하는 것이 일반적인 관례입니다.
pub async fn delete_tag(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let deleted = db::delete_tag(&state.pool, &id).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// 특정 문서에 연결된 태그 목록을 조회합니다.
///
/// `GET /api/v1/documents/:id/tags` → `{ "tags": [...] }`
///
/// 먼저 문서가 존재하는지 확인하고, 존재하면 해당 문서의 태그 목록을 반환합니다.
/// `let _ = ...`: 반환값(Document)은 사용하지 않고 버립니다 (존재 확인만 목적)
pub async fn get_document_tags(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    // 문서 존재 여부 확인 — 없으면 404 반환
    let _ = db::get_document(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    let tags = db::get_document_tags(&state.pool, &id).await?;
    Ok(Json(json!({ "tags": tags })))
}

/// 문서에 태그를 연결합니다.
///
/// `POST /api/v1/documents/:id/tags` + `{ "tag_id": "..." }` → `201 Created`
///
/// 문서와 태그가 모두 존재하는지 확인한 후 연결합니다.
/// 이미 연결되어 있으면 `INSERT OR IGNORE`로 중복을 무시합니다.
pub async fn add_tag_to_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AddTagToDocumentRequest>,
) -> Result<StatusCode, AppError> {
    // 문서 존재 여부 확인
    let _ = db::get_document(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 태그 존재 여부 확인 — 없는 태그를 연결하려는 것을 방지
    let _ = db::get_tag(&state.pool, &req.tag_id)
        .await?
        .ok_or(AppError::NotFound)?;

    db::add_tag_to_document(&state.pool, &id, &req.tag_id).await?;
    // 201 Created: 새 리소스(관계)가 생성되었음을 나타냅니다
    Ok(StatusCode::CREATED)
}

/// 문서에서 태그 연결을 해제합니다.
///
/// `DELETE /api/v1/documents/:id/tags/:tag_id` → `204 No Content`
///
/// `Path((doc_id, tag_id))`: URL에 경로 변수가 2개일 때 튜플로 추출합니다.
/// 순서는 URL에 나타나는 순서와 동일합니다:
/// `/documents/:id/tags/:tag_id` → (id, tag_id) → (doc_id, tag_id)
pub async fn remove_tag_from_document(
    State(state): State<AppState>,
    Path((doc_id, tag_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let removed = db::remove_tag_from_document(&state.pool, &doc_id, &tag_id).await?;
    if !removed {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
