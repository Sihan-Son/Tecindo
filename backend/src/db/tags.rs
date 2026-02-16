//! # 태그 데이터베이스 쿼리 모듈
//!
//! 태그 CRUD 및 문서-태그 관계를 관리하는 SQL 쿼리 함수들입니다.
//! 모든 함수는 `SqlitePool` 참조를 받아 비동기로 실행됩니다.
//!
//! ## 테이블 구조
//! - `tags`: 태그 엔티티 (id, name, color)
//! - `document_tags`: 문서와 태그의 다대다(N:M) 관계 테이블

use crate::error::AppError;
use crate::models::*;
use sqlx::SqlitePool;

/// 모든 태그를 이름순으로 조회합니다.
///
/// `sqlx::query_as::<_, Tag>(sql)` 설명:
/// - `query_as`는 SQL 결과를 지정한 구조체(Tag)로 자동 변환합니다
/// - `<_, Tag>`에서 `_`는 DB 드라이버(SQLite)를 컴파일러가 추론하게 하고,
///   `Tag`는 결과를 매핑할 대상 구조체입니다
/// - `fetch_all`은 모든 행을 Vec으로 반환합니다
pub async fn list_tags(pool: &SqlitePool) -> Result<Vec<Tag>, AppError> {
    let tags = sqlx::query_as::<_, Tag>(
        "SELECT id, name, color FROM tags ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    Ok(tags)
}

/// ID로 태그 하나를 조회합니다.
///
/// `fetch_optional`은 결과가 0행이면 None, 1행이면 Some(Tag)을 반환합니다.
/// `fetch_one`을 쓰면 0행일 때 에러가 발생하므로, 존재 여부가 불확실한 경우
/// `fetch_optional`이 더 안전합니다.
pub async fn get_tag(pool: &SqlitePool, id: &str) -> Result<Option<Tag>, AppError> {
    let tag = sqlx::query_as::<_, Tag>(
        "SELECT id, name, color FROM tags WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(tag)
}

/// 새 태그를 생성하고 생성된 태그를 반환합니다.
///
/// ## 처리 흐름
/// 1. UUIDv7으로 고유 ID 생성 — v7은 타임스탬프 기반이라 시간순 정렬이 가능합니다
/// 2. INSERT 쿼리로 DB에 저장
/// 3. 방금 생성한 태그를 다시 조회하여 반환 (DB의 기본값이 적용된 완전한 데이터)
///
/// `.bind()`는 SQL의 `?` 플레이스홀더에 값을 바인딩합니다.
/// 직접 문자열을 SQL에 넣지 않고 바인딩을 쓰는 이유: SQL 인젝션 방지
pub async fn create_tag(pool: &SqlitePool, req: &CreateTagRequest) -> Result<Tag, AppError> {
    // UUIDv7: 시간 기반 UUID로, 생성 순서대로 정렬됩니다
    let id = uuid::Uuid::now_v7().to_string();

    sqlx::query("INSERT INTO tags (id, name, color) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&req.name)
        .bind(&req.color) // Option<String>도 bind 가능 — None이면 SQL NULL로 처리됨
        .execute(pool)
        .await?;

    // 생성 직후 조회하여 완전한 Tag 객체를 반환합니다
    // ok_or(): Option을 Result로 변환 — None이면 지정한 에러를 반환
    get_tag(pool, &id)
        .await?
        .ok_or(AppError::Internal("Failed to retrieve created tag".to_string()))
}

/// 태그 정보를 부분 업데이트합니다.
///
/// PATCH 방식: 변경 요청에 포함된 필드만 업데이트합니다.
/// 예를 들어 name만 보내면 color는 그대로 유지됩니다.
///
/// ## 반환값
/// - `Ok(Some(Tag))`: 업데이트 성공, 변경된 태그 반환
/// - `Ok(None)`: 해당 ID의 태그가 존재하지 않음
/// - `Err(...)`: DB 에러 발생
pub async fn update_tag(
    pool: &SqlitePool,
    id: &str,
    req: &UpdateTagRequest,
) -> Result<Option<Tag>, AppError> {
    // 먼저 태그 존재 여부를 확인합니다
    let tag = get_tag(pool, id).await?;
    if tag.is_none() {
        return Ok(None); // 404 처리를 라우트 핸들러에 위임
    }

    // if let Some(값) = Option: Option이 Some일 때만 내부 블록을 실행하는 패턴 매칭
    // 각 필드를 개별 쿼리로 업데이트합니다 (간결함을 위해 동적 쿼리 빌딩 대신 사용)
    if let Some(name) = &req.name {
        sqlx::query("UPDATE tags SET name = ? WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(color) = &req.color {
        sqlx::query("UPDATE tags SET color = ? WHERE id = ?")
            .bind(color)
            .bind(id)
            .execute(pool)
            .await?;
    }

    // 업데이트 후 최신 상태를 조회하여 반환
    get_tag(pool, id).await
}

/// ID로 태그를 삭제합니다.
///
/// `document_tags` 테이블에 `ON DELETE CASCADE`가 설정되어 있으므로,
/// 태그를 삭제하면 해당 태그와 문서의 관계도 자동으로 삭제됩니다.
///
/// ## 반환값
/// - `true`: 삭제 성공 (1행 이상 삭제됨)
/// - `false`: 해당 ID의 태그가 존재하지 않아 삭제된 행이 없음
pub async fn delete_tag(pool: &SqlitePool, id: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    // rows_affected(): 이 쿼리로 영향받은 행 수를 반환
    Ok(result.rows_affected() > 0)
}

/// 문서에 태그를 연결합니다 (다대다 관계 추가).
///
/// `INSERT OR IGNORE`: 이미 동일한 (document_id, tag_id) 조합이 존재하면
/// 에러를 발생시키지 않고 무시합니다. 이를 통해 중복 연결을 방지합니다.
/// (document_tags 테이블의 PRIMARY KEY가 복합키이므로 중복 시 충돌 발생)
pub async fn add_tag_to_document(
    pool: &SqlitePool,
    document_id: &str,
    tag_id: &str,
) -> Result<(), AppError> {
    sqlx::query("INSERT OR IGNORE INTO document_tags (document_id, tag_id) VALUES (?, ?)")
        .bind(document_id)
        .bind(tag_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// 문서에서 태그 연결을 해제합니다.
///
/// ## 반환값
/// - `true`: 연결 해제 성공
/// - `false`: 해당 문서-태그 관계가 존재하지 않음
pub async fn remove_tag_from_document(
    pool: &SqlitePool,
    document_id: &str,
    tag_id: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM document_tags WHERE document_id = ? AND tag_id = ?")
        .bind(document_id)
        .bind(tag_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// 특정 문서에 연결된 모든 태그를 조회합니다.
///
/// `document_tags` 중간 테이블을 JOIN하여 문서에 속한 태그 목록을 가져옵니다.
/// 다대다 관계에서 중간 테이블 JOIN은 관계형 DB의 기본적인 패턴입니다.
///
/// ```sql
/// tags ←── document_tags ──→ documents
///  (1)         (N:M)           (1)
/// ```
pub async fn get_document_tags(pool: &SqlitePool, document_id: &str) -> Result<Vec<Tag>, AppError> {
    let tags = sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.id, t.name, t.color
        FROM tags t
        JOIN document_tags dt ON dt.tag_id = t.id
        WHERE dt.document_id = ?
        ORDER BY t.name
        "#,
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;

    Ok(tags)
}
