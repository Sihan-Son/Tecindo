//! # 폴더(Folder) 라우트 핸들러
//!
//! 폴더의 CRUD(생성/조회/수정/삭제)를 처리하는 HTTP 핸들러 함수들입니다.
//!
//! ## 엔드포인트
//! - `GET    /api/v1/folders`     → 전체 폴더 목록 조회
//! - `POST   /api/v1/folders`     → 새 폴더 생성
//! - `PATCH  /api/v1/folders/:id` → 폴더 수정 (부분 업데이트)
//! - `DELETE /api/v1/folders/:id` → 폴더 삭제
//!
//! 폴더는 `parent_id`를 통해 트리(계층) 구조를 형성합니다.
//! 폴더를 삭제하면 DB 스키마의 `ON DELETE SET NULL`에 의해
//! 해당 폴더에 속한 문서들이 자동으로 루트로 이동합니다.

use crate::{
    db,
    error::AppError,
    models::*,
    routes::documents::AppState, // AppState는 documents 모듈에 정의되어 있습니다.
};
use axum::{
    extract::{Path, State}, // Path: URL 파라미터 추출, State: 앱 상태 추출
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

/// `GET /folders` — 전체 폴더 목록을 조회합니다.
///
/// 정렬 순서(sort_order) → 이름순으로 정렬하여 반환합니다.
/// 응답: `{ "folders": [...] }`
pub async fn list_folders(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let folders = db::list_folders(&state.pool).await?;
    Ok(Json(json!({ "folders": folders })))
}

/// `POST /folders` — 새 폴더를 생성합니다.
///
/// 요청 본문: `{ "name": "폴더 이름", "parent_id": "부모 ID (선택)" }`
/// 이름으로부터 slug을 자동 생성합니다.
pub async fn create_folder(
    State(state): State<AppState>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<Folder>, AppError> {
    // slug::slugify(): 폴더 이름을 URL 친화적인 형태로 변환
    // &req.name: 참조로 전달하여 소유권을 유지합니다.
    let slug = slug::slugify(&req.name);
    // req.name, req.parent_id: 여기서 소유권이 이동(move)됩니다.
    // 이후 req의 이 필드들은 사용할 수 없습니다.
    let folder = db::create_folder(&state.pool, req.name, req.parent_id, slug).await?;
    Ok(Json(folder))
}

/// `PATCH /folders/:id` — 폴더를 수정합니다 (부분 업데이트).
///
/// 요청 본문에 포함된 필드만 업데이트합니다.
/// 예: `{ "name": "새 이름" }` → 이름만 변경, 나머지는 그대로
pub async fn update_folder(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateFolderRequest>,
) -> Result<Json<Folder>, AppError> {
    let folder = db::update_folder(&state.pool, &id, &req)
        .await?
        .ok_or(AppError::NotFound)?; // 폴더가 없으면 404 응답
    Ok(Json(folder))
}

/// `DELETE /folders/:id` — 폴더를 삭제합니다.
///
/// 성공 시 HTTP 204 No Content를 반환합니다.
/// 해당 폴더에 속한 문서들은 DB 외래키 제약조건에 의해
/// folder_id가 NULL로 설정됩니다 (루트로 이동).
pub async fn delete_folder(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let deleted = db::delete_folder(&state.pool, &id).await?;
    if !deleted {
        return Err(AppError::NotFound); // 삭제할 폴더가 없으면 404
    }
    Ok(StatusCode::NO_CONTENT) // 204: 성공했지만 반환할 본문 없음
}
