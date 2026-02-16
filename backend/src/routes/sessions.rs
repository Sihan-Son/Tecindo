//! # 글쓰기 세션 API 라우트 핸들러
//!
//! 글쓰기 세션의 시작, 종료, 조회를 위한 HTTP 핸들러 함수들입니다.
//!
//! ## 엔드포인트 목록
//! | 메서드 | 경로 | 핸들러 | 설명 |
//! |--------|------|--------|------|
//! | GET | /api/v1/documents/:id/sessions | `list_document_sessions` | 문서의 세션 목록 |
//! | POST | /api/v1/documents/:id/sessions | `create_writing_session` | 새 세션 시작 |
//! | PATCH | /api/v1/sessions/:id | `end_writing_session` | 세션 종료 |
//!
//! ## 세션 사용 흐름
//! ```text
//! 1. 에디터 진입 → POST /documents/:id/sessions (세션 시작)
//! 2. 글쓰기 진행 중...
//! 3. 에디터 나감 → PATCH /sessions/:id (세션 종료, 최종 단어 수 기록)
//! 4. 통계 확인 → GET /documents/:id/sessions (세션 이력 조회)
//! ```

use crate::{
    db,
    error::AppError,
    models::*,                   // WritingSession, CreateSessionRequest, EndSessionRequest
    routes::documents::AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

/// 특정 문서의 글쓰기 세션 목록을 조회합니다.
///
/// `GET /api/v1/documents/:id/sessions` → `{ "sessions": [...] }`
///
/// 최신 세션이 먼저 오도록 started_at 내림차순으로 정렬됩니다.
/// 문서의 작성 이력을 시간순으로 추적하는 데 사용합니다.
pub async fn list_document_sessions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    // 문서 존재 여부를 먼저 확인합니다
    // 존재하지 않는 문서의 세션을 조회하면 빈 배열 대신 404를 반환하기 위함
    let _ = db::get_document(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    let sessions = db::list_sessions_for_document(&state.pool, &id).await?;
    Ok(Json(json!({ "sessions": sessions })))
}

/// 새 글쓰기 세션을 시작합니다.
///
/// `POST /api/v1/documents/:id/sessions`
/// + `{ "device_name": "MacBook Pro", "word_count_start": 150 }`
///
/// ## 매개변수 처리
/// - `device_name`: 선택 항목 — 요청에 없으면 None으로 처리
/// - `word_count_start`: 선택 항목 — 요청에 없으면 0으로 처리
///
/// `as_deref()`: `Option<String>`을 `Option<&str>`로 변환합니다.
/// String은 소유권이 있는 타입이고, &str은 빌린 참조 타입입니다.
/// DB 함수가 `Option<&str>`을 받으므로 이 변환이 필요합니다.
///
/// `unwrap_or(0)`: Option이 None이면 기본값 0을 사용합니다.
pub async fn create_writing_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<WritingSession>, AppError> {
    // 문서 존재 여부 확인
    let _ = db::get_document(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    let session = db::create_session(
        &state.pool,
        &id,
        req.device_name.as_deref(), // Option<String> → Option<&str> 변환
        req.word_count_start.unwrap_or(0), // None이면 기본값 0
    )
    .await?;

    Ok(Json(session))
}

/// 진행 중인 글쓰기 세션을 종료합니다.
///
/// `PATCH /api/v1/sessions/:id` + `{ "word_count_end": 250 }`
///
/// 세션 종료 시 서버가 자동으로:
/// 1. `ended_at`을 현재 UTC 시각으로 설정
/// 2. `word_count_end`를 요청값으로 설정
///
/// 이후 프론트엔드에서 (word_count_end - word_count_start)로
/// 이 세션에서 작성한 단어 수를 계산할 수 있습니다.
pub async fn end_writing_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<EndSessionRequest>,
) -> Result<Json<WritingSession>, AppError> {
    let session = db::end_session(&state.pool, &id, req.word_count_end)
        .await?
        .ok_or(AppError::NotFound)?; // 세션이 없으면 404 반환
    Ok(Json(session))
}
