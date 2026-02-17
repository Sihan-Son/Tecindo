//! # 문서 및 폴더 데이터베이스 쿼리 모듈
//!
//! 이 파일에는 `documents`와 `folders` 테이블에 대한
//! CRUD(Create, Read, Update, Delete) 쿼리 함수들이 정의되어 있습니다.
//!
//! 모든 함수는 `async`이며 `SqlitePool`을 받아 데이터베이스와 상호작용합니다.
//! 에러 발생 시 `AppError`를 반환합니다.

use crate::error::AppError;
use crate::models::*;
// SqlitePool: SQLite 연결 풀. 여러 비동기 작업이 동시에 DB에 접근할 수 있게 합니다.
// &SqlitePool로 받으면 소유권을 가져가지 않고 빌려서(borrow) 사용합니다.
use sqlx::SqlitePool;

/// 모든 문서를 조회합니다.
///
/// 고정(pinned)된 문서가 먼저 표시되고, 그 다음 수정일 기준 내림차순 정렬합니다.
///
/// # 매개변수
/// - `pool`: SQLite 연결 풀의 참조(&). 소유권을 가져가지 않고 빌려 씁니다.
///
/// # 반환값
/// - `Result<Vec<Document>, AppError>`: 성공 시 문서 목록, 실패 시 에러
///   Vec<T>: 가변 길이 배열 (다른 언어의 ArrayList, List 등에 해당)
pub async fn list_documents(pool: &SqlitePool, user_id: &str) -> Result<Vec<Document>, AppError> {
    let docs = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, folder_id, title, slug, file_path, word_count, char_count,
               excerpt, is_pinned, is_archived, created_at, updated_at
        FROM documents
        WHERE user_id = ?
        ORDER BY is_pinned DESC, updated_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(docs)
}

/// ID로 단일 문서를 조회합니다.
///
/// # 매개변수
/// - `pool`: DB 연결 풀
/// - `id`: 조회할 문서의 UUIDv7 식별자 (&str: 문자열 슬라이스, 즉 문자열의 참조)
///
/// # 반환값
/// - `Ok(Some(Document))`: 문서를 찾은 경우
/// - `Ok(None)`: 해당 ID의 문서가 없는 경우
/// - `Err(AppError)`: DB 에러 발생 시
pub async fn get_document(pool: &SqlitePool, id: &str, user_id: &str) -> Result<Option<Document>, AppError> {
    let doc = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, folder_id, title, slug, file_path, word_count, char_count,
               excerpt, is_pinned, is_archived, created_at, updated_at
        FROM documents
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(doc)
}

/// 같은 폴더에서 "Untitled" 접두사를 가진 문서 제목들을 조회합니다.
pub async fn list_untitled_titles(
    pool: &SqlitePool,
    folder_id: Option<&str>,
    user_id: &str,
) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> = if let Some(fid) = folder_id {
        sqlx::query_as(
            "SELECT title FROM documents WHERE folder_id = ? AND user_id = ? AND title LIKE 'Untitled%'",
        )
        .bind(fid)
        .bind(user_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT title FROM documents WHERE folder_id IS NULL AND user_id = ? AND title LIKE 'Untitled%'",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
    };
    Ok(rows.into_iter().map(|(t,)| t).collect())
}

/// 새 문서를 생성합니다.
///
/// 문서 레코드를 DB에 삽입하고, 생성된 문서를 다시 조회하여 반환합니다.
///
/// # 매개변수
/// - `pool`: DB 연결 풀
/// - `req`: 문서 생성 요청 데이터 (제목, 폴더 ID)
/// - `file_path`: 마크다운 파일이 저장될 경로
/// - `slug`: URL 친화적인 문서 식별자
pub async fn create_document(
    pool: &SqlitePool,
    id: &str,
    req: &CreateDocumentRequest,
    file_path: String,
    slug: String,
    user_id: &str,
) -> Result<Document, AppError> {
    let title = req.title.clone().unwrap_or_else(|| "Untitled".to_string());

    sqlx::query(
        r#"
        INSERT INTO documents (id, folder_id, title, slug, file_path, user_id)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&req.folder_id)
    .bind(&title)
    .bind(&slug)
    .bind(&file_path)
    .bind(user_id)
    .execute(pool)
    .await?;

    get_document(pool, id, user_id)
        .await?
        .ok_or(AppError::Internal("Failed to retrieve created document".to_string()))
}

/// 문서를 수정합니다 (부분 업데이트 - PATCH 방식).
///
/// 요청에 포함된 필드만 업데이트하고, 나머지는 그대로 유지합니다.
/// 동적으로 SQL UPDATE 쿼리를 구성합니다.
///
/// # 매개변수
/// - `pool`: DB 연결 풀
/// - `id`: 수정할 문서의 ID
/// - `req`: 수정할 필드들 (None인 필드는 변경하지 않음)
///
/// # 반환값
/// - `Ok(Some(Document))`: 수정 성공
/// - `Ok(None)`: 해당 ID의 문서가 없음
pub async fn update_document(
    pool: &SqlitePool,
    id: &str,
    req: &UpdateDocumentRequest,
    user_id: &str,
) -> Result<Option<Document>, AppError> {
    let doc = get_document(pool, id, user_id).await?;
    if doc.is_none() {
        return Ok(None);
    }

    let mut query = String::from("UPDATE documents SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");
    let mut bindings = Vec::new();

    if let Some(title) = &req.title {
        query.push_str(", title = ?");
        bindings.push(title.as_str());
    }

    if let Some(folder_id_opt) = &req.folder_id {
        match folder_id_opt {
            Some(folder_id) => {
                query.push_str(", folder_id = ?");
                bindings.push(folder_id.as_str());
            }
            None => {
                query.push_str(", folder_id = NULL");
            }
        }
    }

    if let Some(is_pinned) = req.is_pinned {
        query.push_str(", is_pinned = ?");
        bindings.push(if is_pinned { "1" } else { "0" });
    }

    if let Some(is_archived) = req.is_archived {
        query.push_str(", is_archived = ?");
        bindings.push(if is_archived { "1" } else { "0" });
    }

    query.push_str(" WHERE id = ? AND user_id = ?");
    bindings.push(id);
    bindings.push(user_id);

    let mut query_builder = sqlx::query(&query);
    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }

    query_builder.execute(pool).await?;

    get_document(pool, id, user_id).await
}

/// 문서를 삭제합니다.
///
/// # 매개변수
/// - `pool`: DB 연결 풀
/// - `id`: 삭제할 문서의 ID
///
/// # 반환값
/// - `Ok(true)`: 삭제 성공 (1행 이상 영향)
/// - `Ok(false)`: 해당 ID의 문서가 없음 (0행 영향)
pub async fn delete_document(pool: &SqlitePool, id: &str, user_id: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM documents WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// 특정 태그가 붙은 문서만 조회합니다.
///
/// `document_tags` 중간 테이블을 JOIN하여 해당 태그를 가진 문서 목록을 반환합니다.
pub async fn list_documents_by_tag(pool: &SqlitePool, tag_id: &str, user_id: &str) -> Result<Vec<Document>, AppError> {
    let docs = sqlx::query_as::<_, Document>(
        r#"
        SELECT d.id, d.folder_id, d.title, d.slug, d.file_path, d.word_count, d.char_count,
               d.excerpt, d.is_pinned, d.is_archived, d.created_at, d.updated_at
        FROM documents d
        JOIN document_tags dt ON dt.document_id = d.id
        WHERE dt.tag_id = ? AND d.user_id = ?
        ORDER BY d.is_pinned DESC, d.updated_at DESC
        "#,
    )
    .bind(tag_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(docs)
}

// ── 폴더(Folder) 관련 쿼리 ──

/// 모든 폴더를 조회합니다 (정렬 순서 → 이름순).
pub async fn list_folders(pool: &SqlitePool, user_id: &str) -> Result<Vec<Folder>, AppError> {
    let folders = sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, parent_id, name, slug, sort_order, created_at, updated_at
        FROM folders
        WHERE user_id = ?
        ORDER BY sort_order, name
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(folders)
}

/// 새 폴더를 생성합니다.
///
/// # 매개변수
/// - `name`: 폴더 이름
/// - `parent_id`: 부모 폴더 ID (None이면 최상위 폴더)
/// - `slug`: URL 친화적인 이름
pub async fn create_folder(
    pool: &SqlitePool,
    name: String,
    parent_id: Option<String>,
    slug: String,
    user_id: &str,
) -> Result<Folder, AppError> {
    let id = uuid::Uuid::now_v7().to_string();

    sqlx::query(
        r#"
        INSERT INTO folders (id, parent_id, name, slug, user_id)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&parent_id)
    .bind(&name)
    .bind(&slug)
    .bind(user_id)
    .execute(pool)
    .await?;

    get_folder(pool, &id, user_id)
        .await?
        .ok_or(AppError::Internal("Failed to retrieve created folder".to_string()))
}

/// ID로 단일 폴더를 조회합니다.
pub async fn get_folder(pool: &SqlitePool, id: &str, user_id: &str) -> Result<Option<Folder>, AppError> {
    let folder = sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, parent_id, name, slug, sort_order, created_at, updated_at
        FROM folders
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(folder)
}

/// 폴더를 수정합니다 (부분 업데이트).
///
/// 동적 쿼리 구성이 복잡하여, 각 필드를 개별 UPDATE 문으로 처리합니다.
/// (성능보다 코드 단순성을 우선한 접근)
pub async fn update_folder(
    pool: &SqlitePool,
    id: &str,
    req: &UpdateFolderRequest,
    user_id: &str,
) -> Result<Option<Folder>, AppError> {
    let folder = get_folder(pool, id, user_id).await?;
    if folder.is_none() {
        return Ok(None);
    }

    if let Some(name) = &req.name {
        sqlx::query("UPDATE folders SET name = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ? AND user_id = ?")
            .bind(name)
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    if let Some(parent_id) = &req.parent_id {
        sqlx::query("UPDATE folders SET parent_id = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ? AND user_id = ?")
            .bind(parent_id)
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    if let Some(sort_order) = req.sort_order {
        sqlx::query("UPDATE folders SET sort_order = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ? AND user_id = ?")
            .bind(sort_order)
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    get_folder(pool, id, user_id).await
}

/// 폴더를 삭제합니다.
///
/// DB 스키마에서 `ON DELETE SET NULL`이 설정되어 있으므로,
/// 폴더가 삭제되면 그 안의 문서들의 folder_id가 NULL로 설정됩니다.
pub async fn delete_folder(pool: &SqlitePool, id: &str, user_id: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM folders WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
