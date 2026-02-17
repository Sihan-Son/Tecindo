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
pub async fn list_documents(pool: &SqlitePool) -> Result<Vec<Document>, AppError> {
    // sqlx::query_as::<_, Document>():
    //   SQL 쿼리를 실행하고 결과를 Document 구조체로 자동 변환합니다.
    //   <_, Document>에서 _는 데이터베이스 타입(컴파일러가 추론), Document는 결과 타입입니다.
    //   Document에 #[derive(sqlx::FromRow)]가 있어서 자동 변환이 가능합니다.
    //
    // r#"..."#: Raw 문자열 리터럴.
    //   이스케이프 문자(\n, \" 등)를 처리하지 않아 SQL을 그대로 쓸 수 있습니다.
    let docs = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, folder_id, title, slug, file_path, word_count, char_count,
               excerpt, is_pinned, is_archived, created_at, updated_at
        FROM documents
        ORDER BY is_pinned DESC, updated_at DESC
        "#,
        // ↑ SQL 설명:
        //   ORDER BY is_pinned DESC → 고정된 문서(1)가 먼저
        //   updated_at DESC → 최근 수정된 문서가 먼저
    )
    // .fetch_all(pool): 모든 결과 행을 가져옵니다 (Vec<Document> 반환)
    .fetch_all(pool)
    // .await: 비동기 작업 완료 대기
    // ?: 에러 발생 시 AppError로 자동 변환 후 함수에서 반환 (에러 전파)
    //    sqlx::Error → AppError::Database 변환은 error.rs의 #[from]이 처리합니다.
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
pub async fn get_document(pool: &SqlitePool, id: &str) -> Result<Option<Document>, AppError> {
    let doc = sqlx::query_as::<_, Document>(
        r#"
        SELECT id, folder_id, title, slug, file_path, word_count, char_count,
               excerpt, is_pinned, is_archived, created_at, updated_at
        FROM documents
        WHERE id = ?
        "#,
        // ↑ SQL의 `?`는 파라미터 바인딩 자리표시자입니다.
        //   아래 .bind(id)로 실제 값을 안전하게 대입합니다.
        //   이 방식은 SQL 인젝션 공격을 방지합니다.
    )
    // .bind(id): SQL의 ? 자리에 id 값을 안전하게 바인딩합니다.
    .bind(id)
    // .fetch_optional(): 결과가 0행이면 None, 1행이면 Some(Document)을 반환합니다.
    //   .fetch_one()은 결과가 없으면 에러를 반환하지만,
    //   .fetch_optional()은 Option으로 감싸서 안전하게 처리합니다.
    .fetch_optional(pool)
    .await?;

    Ok(doc)
}

/// 같은 폴더에서 "Untitled" 접두사를 가진 문서 제목들을 조회합니다.
pub async fn list_untitled_titles(
    pool: &SqlitePool,
    folder_id: Option<&str>,
) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> = if let Some(fid) = folder_id {
        sqlx::query_as(
            "SELECT title FROM documents WHERE folder_id = ? AND title LIKE 'Untitled%'",
        )
        .bind(fid)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT title FROM documents WHERE folder_id IS NULL AND title LIKE 'Untitled%'",
        )
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
) -> Result<Document, AppError> {
    let title = req.title.clone().unwrap_or_else(|| "Untitled".to_string());

    // sqlx::query(): 결과를 구조체로 변환하지 않는 단순 실행 쿼리
    // (query_as와 달리 반환 타입 지정 불필요)
    sqlx::query(
        r#"
        INSERT INTO documents (id, folder_id, title, slug, file_path)
        VALUES (?, ?, ?, ?, ?)
        "#,
        // ↑ SQL: documents 테이블에 새 행을 삽입합니다.
        //   나머지 컬럼(word_count, created_at 등)은 DEFAULT 값이 사용됩니다.
    )
    // 각 ?에 순서대로 값을 바인딩합니다.
    .bind(id)
    .bind(&req.folder_id)
    .bind(&title)
    .bind(&slug)
    .bind(&file_path)
    .execute(pool)
    .await?;

    get_document(pool, id)
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
) -> Result<Option<Document>, AppError> {
    // 먼저 문서가 존재하는지 확인
    let doc = get_document(pool, id).await?;
    if doc.is_none() {
        return Ok(None); // 문서가 없으면 None 반환 (라우트 핸들러에서 404로 변환)
    }

    // ── 동적 쿼리 구성 ──
    // PATCH 요청이므로, 클라이언트가 보낸 필드만 SQL에 포함해야 합니다.
    // String::from(): &str에서 소유된 String을 생성합니다.
    // mut: 변수를 가변(mutable)으로 선언. Rust에서 변수는 기본적으로 불변(immutable)입니다.
    let mut query = String::from("UPDATE documents SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");
    // Vec<&str>: 문자열 슬라이스(&str)의 가변 길이 배열
    // 나중에 SQL의 ? 자리에 순서대로 바인딩할 값들을 모아둡니다.
    let mut bindings = Vec::new();

    // if let Some(x) = ...: Option이 Some인 경우에만 실행하는 패턴 매칭
    // &req.title: 참조로 접근 (소유권을 가져가지 않음)
    if let Some(title) = &req.title {
        // .push_str(): String에 문자열을 추가 (append)
        query.push_str(", title = ?");
        // .as_str(): String을 &str로 변환
        bindings.push(title.as_str());
    }

    // .is_some(): Option이 Some인지 확인 (bool 반환)
    if req.folder_id.is_some() {
        query.push_str(", folder_id = ?");
        // .as_ref(): Option<String>을 Option<&String>으로 변환 (소유권 이동 방지)
        // .map(|s| s.as_str()): Option 안의 String을 &str로 변환
        // .unwrap_or(""): None이면 빈 문자열 사용
        bindings.push(req.folder_id.as_ref().map(|s| s.as_str()).unwrap_or(""));
    }

    if let Some(is_pinned) = req.is_pinned {
        query.push_str(", is_pinned = ?");
        // SQLite에는 BOOLEAN 타입이 없어 INTEGER 0/1로 처리합니다.
        // if 표현식: Rust에서 if-else는 값을 반환하는 표현식(expression)입니다.
        bindings.push(if is_pinned { "1" } else { "0" });
    }

    if let Some(is_archived) = req.is_archived {
        query.push_str(", is_archived = ?");
        bindings.push(if is_archived { "1" } else { "0" });
    }

    // WHERE 절 추가 (특정 문서만 수정)
    query.push_str(" WHERE id = ?");
    bindings.push(id);

    // ── 동적 쿼리 실행 ──
    // sqlx::query(&query): 동적으로 구성한 SQL 문자열로 쿼리를 생성합니다.
    let mut query_builder = sqlx::query(&query);
    // for 루프: bindings의 각 값을 순서대로 ? 자리에 바인딩합니다.
    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }

    query_builder.execute(pool).await?;

    // 수정된 문서를 다시 조회하여 반환 (최신 updated_at 값 포함)
    get_document(pool, id).await
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
pub async fn delete_document(pool: &SqlitePool, id: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM documents WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    // .rows_affected(): 쿼리에 의해 영향받은 행 수를 반환합니다.
    // 삭제된 행이 있으면 true, 없으면 false
    Ok(result.rows_affected() > 0)
}

// ── 폴더(Folder) 관련 쿼리 ──

/// 모든 폴더를 조회합니다 (정렬 순서 → 이름순).
pub async fn list_folders(pool: &SqlitePool) -> Result<Vec<Folder>, AppError> {
    let folders = sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, parent_id, name, slug, sort_order, created_at, updated_at
        FROM folders
        ORDER BY sort_order, name
        "#,
        // ↑ SQL: sort_order가 같으면 이름(name) 알파벳순으로 정렬
    )
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
) -> Result<Folder, AppError> {
    let id = uuid::Uuid::now_v7().to_string();

    sqlx::query(
        r#"
        INSERT INTO folders (id, parent_id, name, slug)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&parent_id) // Option<String>: Some이면 값을, None이면 SQL NULL을 바인딩합니다.
    .bind(&name)
    .bind(&slug)
    .execute(pool)
    .await?;

    // 생성된 폴더를 다시 조회하여 반환
    get_folder(pool, &id)
        .await?
        .ok_or(AppError::Internal("Failed to retrieve created folder".to_string()))
}

/// ID로 단일 폴더를 조회합니다.
pub async fn get_folder(pool: &SqlitePool, id: &str) -> Result<Option<Folder>, AppError> {
    let folder = sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, parent_id, name, slug, sort_order, created_at, updated_at
        FROM folders
        WHERE id = ?
        "#,
    )
    .bind(id)
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
) -> Result<Option<Folder>, AppError> {
    // 폴더 존재 여부 확인
    let folder = get_folder(pool, id).await?;
    if folder.is_none() {
        return Ok(None);
    }

    // ── 각 필드를 개별 쿼리로 업데이트 ──
    // 각 필드마다 별도의 UPDATE 문을 실행합니다.
    // 트랜잭션이 없어 원자성은 보장되지 않지만, 단순하고 안전한 접근입니다.
    if let Some(name) = &req.name {
        sqlx::query("UPDATE folders SET name = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(parent_id) = &req.parent_id {
        sqlx::query("UPDATE folders SET parent_id = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?")
            .bind(parent_id)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(sort_order) = req.sort_order {
        sqlx::query("UPDATE folders SET sort_order = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?")
            .bind(sort_order)
            .bind(id)
            .execute(pool)
            .await?;
    }

    // 수정된 폴더를 다시 조회하여 반환
    get_folder(pool, id).await
}

/// 폴더를 삭제합니다.
///
/// DB 스키마에서 `ON DELETE SET NULL`이 설정되어 있으므로,
/// 폴더가 삭제되면 그 안의 문서들의 folder_id가 NULL로 설정됩니다.
pub async fn delete_folder(pool: &SqlitePool, id: &str) -> Result<bool, AppError> {
    let result = sqlx::query("DELETE FROM folders WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
