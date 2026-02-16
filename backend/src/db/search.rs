//! # FTS5 전문검색(Full-Text Search) 모듈
//!
//! SQLite의 FTS5 확장을 사용한 전문검색 기능을 구현합니다.
//!
//! ## FTS5란?
//! FTS5(Full-Text Search 5)는 SQLite에 내장된 전문검색 엔진입니다.
//! 일반 `LIKE '%키워드%'`는 모든 행을 순차 탐색하지만,
//! FTS5는 역색인(inverted index)을 사용해 훨씬 빠르게 검색합니다.
//!
//! ## 외부 콘텐츠 테이블(External Content Table) 구조
//! 이 프로젝트의 FTS5 테이블은 `content='documents'` 설정으로 생성되어 있습니다.
//! 이는 FTS5가 검색 인덱스만 자체 관리하고, 원본 데이터는 `documents` 테이블에서
//! 읽어온다는 뜻입니다. 따라서:
//! - **검색(MATCH)**: FTS5 자체 인덱스를 사용 → 빠름
//! - **컬럼 값 읽기**: `documents` 테이블에서 가져옴
//! - **인덱스 관리**: INSERT/DELETE를 수동으로 해야 함 (자동 동기화 없음)
//!
//! ## 주의사항
//! `documents` 테이블에는 `content` 컬럼이 없으므로(파일에 저장),
//! `highlight()`나 `snippet()` 같은 FTS5 함수는 사용할 수 없습니다.
//! 대신 `documents` 테이블의 `excerpt` 필드를 미리보기로 사용합니다.

use crate::error::AppError;
use crate::models::Document;
use sqlx::SqlitePool;

/// 전문검색을 수행하여 매칭되는 문서 목록을 반환합니다.
///
/// FTS5의 MATCH 연산자로 검색하고, rank(관련도 점수)로 정렬합니다.
/// rank 값이 작을수록(음수) 더 관련도가 높은 문서입니다.
///
/// ## SQL 쿼리 설명
/// ```sql
/// FROM documents_fts                    -- FTS5 가상 테이블에서 검색
/// JOIN documents d ON d.rowid = ...     -- documents 테이블과 JOIN하여 메타데이터 조회
/// WHERE documents_fts MATCH ?           -- FTS5 전문검색 수행
/// ORDER BY documents_fts.rank           -- 관련도순 정렬 (BM25 알고리즘)
/// ```
///
/// ## 검색 문법 예시
/// - `"hello world"` → "hello"와 "world"가 모두 포함된 문서 (AND)
/// - `hello OR world` → 둘 중 하나라도 포함된 문서
/// - `hello*` → "hello"로 시작하는 단어가 포함된 문서 (접두사 검색)
pub async fn search_documents(pool: &SqlitePool, query: &str) -> Result<Vec<Document>, AppError> {
    let documents = sqlx::query_as::<_, Document>(
        r#"
        SELECT d.id, d.folder_id, d.title, d.slug, d.file_path,
               d.word_count, d.char_count, d.excerpt,
               d.is_pinned, d.is_archived, d.created_at, d.updated_at
        FROM documents_fts
        JOIN documents d ON d.rowid = documents_fts.rowid
        WHERE documents_fts MATCH ?
        ORDER BY documents_fts.rank
        LIMIT 50
        "#,
    )
    .bind(query)
    .fetch_all(pool)
    .await?;

    Ok(documents)
}

/// 문서의 제목과 본문을 FTS5 인덱스에 등록/갱신합니다.
///
/// 문서 내용이 변경될 때마다 호출되어 검색 인덱스를 최신 상태로 유지합니다.
///
/// ## 동작 방식
/// 1. 문서의 `rowid`(SQLite 내부 정수 ID)를 조회합니다
/// 2. 이전 내용(old_title, old_content)이 있으면 FTS5 삭제 명령으로 기존 인덱스 제거
/// 3. 새 내용(title, content)을 FTS5에 INSERT
///
/// ## FTS5 외부 콘텐츠 테이블의 삭제 방식
/// 일반적인 `DELETE FROM`이 아니라 특수한 삭제 명령을 사용해야 합니다:
/// ```sql
/// INSERT INTO fts테이블(fts테이블, rowid, 컬럼들) VALUES('delete', rowid, 이전값들)
/// ```
/// 이 방식은 FTS5가 역색인에서 해당 토큰들을 정확히 제거하기 위해
/// **이전 값과 정확히 동일한 내용**을 전달받아야 합니다.
///
/// ## 에러 처리
/// 삭제와 삽입 모두 `let _ =`로 에러를 무시합니다(best-effort).
/// 이유: FTS 인덱스 업데이트 실패가 문서 저장 자체를 실패시키면 안 되기 때문입니다.
/// 인덱스가 꼬이면 검색이 부정확해질 수 있지만, 문서 데이터는 안전합니다.
pub async fn index_document(
    pool: &SqlitePool,
    id: &str,
    title: &str,
    content: &str,
    old_title: Option<&str>,
    old_content: Option<&str>,
) -> Result<(), AppError> {
    // rowid 조회: SQLite는 TEXT PRIMARY KEY를 쓰더라도 내부적으로 정수 rowid를 자동 부여합니다.
    // FTS5 외부 콘텐츠 테이블은 이 rowid로 원본 테이블과 매핑됩니다.
    // query_as::<_, (i64,)>: 결과를 i64 하나짜리 튜플로 매핑합니다
    let row: Option<(i64,)> = sqlx::query_as("SELECT rowid FROM documents WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    // let Some(...) = ... else { return }: 패턴 매칭 + 조기 반환
    // rowid가 없으면 (= 문서가 DB에 없으면) 아무것도 하지 않고 종료
    let Some((rowid,)) = row else {
        return Ok(());
    };

    // 이전 내용이 있을 때만 기존 인덱스를 삭제합니다
    // (첫 번째 저장 시에는 old 값이 None이므로 삭제를 건너뜁니다)
    if let (Some(old_t), Some(old_c)) = (old_title, old_content) {
        // FTS5 외부 콘텐츠 테이블의 특수 삭제 명령
        // 첫 번째 컬럼에 'delete' 문자열을 넣으면 "삭제 모드"로 동작합니다
        let _ = sqlx::query(
            "INSERT INTO documents_fts(documents_fts, rowid, title, content) VALUES('delete', ?, ?, ?)",
        )
        .bind(rowid)
        .bind(old_t)
        .bind(old_c)
        .execute(pool)
        .await;
        // let _ = ...: 결과를 의도적으로 무시합니다 (삭제 실패해도 계속 진행)
    }

    // 새 내용으로 FTS5 인덱스에 추가합니다
    // rowid를 명시적으로 지정하여 documents 테이블의 해당 행과 매핑합니다
    let _ = sqlx::query("INSERT INTO documents_fts(rowid, title, content) VALUES(?, ?, ?)")
        .bind(rowid)
        .bind(title)
        .bind(content)
        .execute(pool)
        .await;

    Ok(())
}
