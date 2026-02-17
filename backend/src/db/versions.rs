use crate::models::{DocumentVersion, DocumentVersionSummary};
use sqlx::SqlitePool;

pub async fn create_version(
    pool: &SqlitePool,
    document_id: &str,
    content: &str,
    word_count: i64,
    char_count: i64,
) -> Result<(), sqlx::Error> {
    let id = uuid::Uuid::now_v7().to_string();
    let next_version: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_number), 0) + 1 FROM document_versions WHERE document_id = ?",
    )
    .bind(document_id)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO document_versions (id, document_id, version_number, content, word_count, char_count)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(document_id)
    .bind(next_version)
    .bind(content)
    .bind(word_count)
    .bind(char_count)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_versions(
    pool: &SqlitePool,
    document_id: &str,
) -> Result<Vec<DocumentVersionSummary>, sqlx::Error> {
    let versions = sqlx::query_as::<_, DocumentVersionSummary>(
        r#"
        SELECT id, document_id, version_number, word_count, char_count, created_at
        FROM document_versions
        WHERE document_id = ?
        ORDER BY version_number DESC
        "#,
    )
    .bind(document_id)
    .fetch_all(pool)
    .await?;

    Ok(versions)
}

pub async fn get_version(
    pool: &SqlitePool,
    version_id: &str,
) -> Result<Option<DocumentVersion>, sqlx::Error> {
    let version = sqlx::query_as::<_, DocumentVersion>(
        r#"
        SELECT id, document_id, version_number, content, word_count, char_count, created_at
        FROM document_versions
        WHERE id = ?
        "#,
    )
    .bind(version_id)
    .fetch_optional(pool)
    .await?;

    Ok(version)
}

/// 설정된 간격(분) 이내에 버전이 생성된 적 있는지 확인.
/// 없으면 true (새 버전을 만들어야 함).
pub async fn should_create_version(
    pool: &SqlitePool,
    document_id: &str,
    interval_minutes: u32,
) -> Result<bool, sqlx::Error> {
    let recent_exists: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT 1 FROM document_versions
        WHERE document_id = ?
        AND created_at > strftime('%Y-%m-%dT%H:%M:%fZ', 'now', ? || ' minutes')
        LIMIT 1
        "#,
    )
    .bind(document_id)
    .bind(format!("-{}", interval_minutes))
    .fetch_optional(pool)
    .await?;

    Ok(recent_exists.is_none())
}

/// 문서의 updated_at이 마지막 버전의 created_at보다 최신인지 확인.
/// true면 버전으로 기록되지 않은 변경이 있다는 뜻.
pub async fn needs_version_snapshot(
    pool: &SqlitePool,
    document_id: &str,
) -> Result<bool, sqlx::Error> {
    let result: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT 1 FROM documents d
        WHERE d.id = ?
        AND d.updated_at > COALESCE(
            (SELECT MAX(created_at) FROM document_versions WHERE document_id = ?),
            '1970-01-01T00:00:00.000Z'
        )
        "#,
    )
    .bind(document_id)
    .bind(document_id)
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

pub async fn prune_versions(
    pool: &SqlitePool,
    document_id: &str,
    max_versions: u32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM document_versions
        WHERE document_id = ? AND id NOT IN (
            SELECT id FROM document_versions
            WHERE document_id = ?
            ORDER BY version_number DESC
            LIMIT ?
        )
        "#,
    )
    .bind(document_id)
    .bind(document_id)
    .bind(max_versions)
    .execute(pool)
    .await?;

    Ok(())
}
