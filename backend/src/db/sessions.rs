//! # 글쓰기 세션 데이터베이스 쿼리 모듈
//!
//! 글쓰기 세션의 생성, 종료, 조회를 담당하는 SQL 쿼리 함수들입니다.
//!
//! ## 세션 라이프사이클
//! ```text
//! [시작] create_session() → 진행 중(ended_at = NULL) → end_session() → [종료]
//! ```
//!
//! 세션을 통해 다음을 추적할 수 있습니다:
//! - 언제 글을 쓰기 시작하고 끝냈는지
//! - 어떤 기기에서 작성했는지
//! - 한 세션에서 몇 단어를 썼는지 (word_count_end - word_count_start)

use crate::error::AppError;
use crate::models::WritingSession;
use sqlx::SqlitePool;

/// 새 글쓰기 세션을 시작합니다.
///
/// 문서를 편집하기 시작할 때 호출됩니다.
/// `started_at`은 DB의 DEFAULT 값으로 자동 설정되므로 서버 시간이 기록됩니다.
///
/// ## 매개변수
/// - `document_id`: 편집할 문서의 ID
/// - `device_name`: 사용 중인 기기 이름 (예: "MacBook Pro") — None이면 미기록
/// - `word_count_start`: 세션 시작 시점의 문서 단어 수
pub async fn create_session(
    pool: &SqlitePool,
    document_id: &str,
    device_name: Option<&str>,
    word_count_start: i64,
) -> Result<WritingSession, AppError> {
    // UUIDv7으로 세션 ID를 생성합니다
    let id = uuid::Uuid::now_v7().to_string();

    // r#"..."#: Rust의 원시 문자열 리터럴 (raw string literal)
    // 이스케이프 처리 없이 줄바꿈과 따옴표를 그대로 쓸 수 있어 SQL 작성에 편리합니다
    sqlx::query(
        r#"
        INSERT INTO writing_sessions (id, document_id, device_name, word_count_start)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(document_id)
    .bind(device_name) // Option<&str>: None이면 SQL NULL
    .bind(word_count_start)
    .execute(pool)
    .await?; // ?: 에러 발생 시 이 함수에서 즉시 반환 (에러 전파)

    // 생성 직후 조회하여 DB가 채운 기본값(started_at 등)이 포함된 완전한 객체를 반환
    get_session(pool, &id)
        .await?
        .ok_or(AppError::Internal(
            "Failed to retrieve created session".to_string(),
        ))
}

/// ID로 세션 하나를 조회합니다.
///
/// 세션이 존재하면 `Some(WritingSession)`, 없으면 `None`을 반환합니다.
pub async fn get_session(pool: &SqlitePool, id: &str) -> Result<Option<WritingSession>, AppError> {
    let session = sqlx::query_as::<_, WritingSession>(
        r#"
        SELECT id, document_id, device_name, started_at, ended_at,
               word_count_start, word_count_end
        FROM writing_sessions
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool) // 0행이면 None, 1행이면 Some
    .await?;

    Ok(session)
}

/// 진행 중인 글쓰기 세션을 종료합니다.
///
/// `ended_at`을 현재 시각으로 설정하고, 종료 시점의 단어 수를 기록합니다.
/// SQL의 `strftime('%Y-%m-%dT%H:%M:%fZ', 'now')`로 서버 측 UTC 시각을 기록합니다.
///
/// ## 반환값
/// - `Ok(Some(WritingSession))`: 종료 성공, 업데이트된 세션 반환
/// - `Ok(None)`: 해당 ID의 세션이 존재하지 않음
pub async fn end_session(
    pool: &SqlitePool,
    id: &str,
    word_count_end: Option<i64>,
) -> Result<Option<WritingSession>, AppError> {
    // 먼저 세션이 존재하는지 확인합니다
    let session = get_session(pool, id).await?;
    if session.is_none() {
        return Ok(None); // 존재하지 않으면 None 반환 → 라우트에서 404 처리
    }

    // 세션 종료: ended_at에 현재 시각, word_count_end에 최종 단어 수를 기록
    sqlx::query(
        r#"
        UPDATE writing_sessions
        SET ended_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
            word_count_end = ?
        WHERE id = ?
        "#,
    )
    .bind(word_count_end)
    .bind(id)
    .execute(pool)
    .await?;

    // 업데이트된 세션을 다시 조회하여 반환 (ended_at이 채워진 상태)
    get_session(pool, id).await
}

/// 특정 문서의 모든 글쓰기 세션을 최신순으로 조회합니다.
///
/// 가장 최근 세션이 먼저 오도록 `started_at DESC`로 정렬합니다.
/// 이를 통해 문서의 작성 이력과 작성 패턴을 확인할 수 있습니다.
pub async fn list_sessions_for_document(
    pool: &SqlitePool,
    document_id: &str,
) -> Result<Vec<WritingSession>, AppError> {
    let sessions = sqlx::query_as::<_, WritingSession>(
        r#"
        SELECT id, document_id, device_name, started_at, ended_at,
               word_count_start, word_count_end
        FROM writing_sessions
        WHERE document_id = ?
        ORDER BY started_at DESC
        "#,
    )
    .bind(document_id)
    .fetch_all(pool) // 모든 행을 Vec으로 반환 (0개여도 빈 Vec)
    .await?;

    Ok(sessions)
}
