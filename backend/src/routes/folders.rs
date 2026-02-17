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
    middleware::auth::AuthUser,
    models::*,
    routes::documents::AppState,
};
use axum::{
    extract::{Path, State},
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
    auth_user: AuthUser,
) -> Result<Json<Value>, AppError> {
    let folders = db::list_folders(&state.pool, &auth_user.user_id).await?;
    Ok(Json(json!({ "folders": folders })))
}

/// `POST /folders` — 새 폴더를 생성합니다.
///
/// 요청 본문: `{ "name": "폴더 이름", "parent_id": "부모 ID (선택)" }`
/// 이름으로부터 slug을 자동 생성합니다.
pub async fn create_folder(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<Folder>, AppError> {
    let slug = slug::slugify(&req.name);
    let folder = db::create_folder(&state.pool, req.name, req.parent_id, slug, &auth_user.user_id).await?;
    Ok(Json(folder))
}

/// `PATCH /folders/:id` — 폴더를 수정합니다 (부분 업데이트).
///
/// 요청 본문에 포함된 필드만 업데이트합니다.
/// 예: `{ "name": "새 이름" }` → 이름만 변경, 나머지는 그대로
pub async fn update_folder(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateFolderRequest>,
) -> Result<Json<Folder>, AppError> {
    let folder = db::update_folder(&state.pool, &id, &req, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(folder))
}

/// `DELETE /folders/:id` — 폴더를 삭제합니다.
///
/// 성공 시 HTTP 204 No Content를 반환합니다.
/// 해당 폴더에 속한 문서들은 DB 외래키 제약조건에 의해
/// folder_id가 NULL로 설정됩니다 (루트로 이동).
pub async fn delete_folder(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let deleted = db::delete_folder(&state.pool, &id, &auth_user.user_id).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
