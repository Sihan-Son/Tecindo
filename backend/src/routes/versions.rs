use crate::{
    db,
    error::AppError,
    middleware::auth::AuthUser,
    services,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

use super::documents::AppState;

pub async fn list_document_versions(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    // 소유권 확인
    db::get_document(&state.pool, &id, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let versions = db::list_versions(&state.pool, &id).await?;
    Ok(Json(json!({ "versions": versions })))
}

pub async fn get_version_content(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let version = db::get_version(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 문서 소유권 확인
    db::get_document(&state.pool, &version.document_id, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(json!(version)))
}

/// `POST /documents/:id/versions` — 비활동 시 프론트엔드가 호출하는 스냅샷 생성 엔드포인트.
/// 마지막 버전 이후 변경이 있을 때만 새 버전을 생성한다.
pub async fn create_version_snapshot(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let doc = db::get_document(&state.pool, &id, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 마지막 버전 이후 변경이 없으면 스킵
    if !db::needs_version_snapshot(&state.pool, &id).await? {
        return Ok(StatusCode::NO_CONTENT);
    }

    let content = services::read_markdown(&state.documents_path, &doc.file_path).await?;
    let word_count = services::count_words(&content) as i64;
    let char_count = services::count_chars(&content) as i64;

    db::create_version(&state.pool, &id, &content, word_count, char_count).await?;
    db::prune_versions(&state.pool, &id, state.max_document_versions).await?;

    Ok(StatusCode::CREATED)
}
