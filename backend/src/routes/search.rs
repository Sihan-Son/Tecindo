//! # 전문검색 API 라우트 핸들러
//!
//! FTS5 기반 전문검색 엔드포인트를 제공합니다.
//!
//! ## 엔드포인트
//! | 메서드 | 경로 | 설명 |
//! |--------|------|------|
//! | GET | /api/v1/search?q=키워드 | 문서 전문검색 |
//!
//! ## 사용 예시
//! ```
//! GET /api/v1/search?q=프로그래밍
//! GET /api/v1/search?q=rust+async    ← 공백은 AND 검색
//! GET /api/v1/search?q=hello*        ← 접두사 검색 (hello로 시작하는 단어)
//! ```

use crate::{
    db,
    error::AppError,
    routes::documents::AppState,
};
use axum::{
    extract::{Query, State}, // Query: URL 쿼리 파라미터(?key=value)를 추출하는 추출자
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

/// URL 쿼리 파라미터를 파싱하는 구조체입니다.
///
/// `GET /api/v1/search?q=키워드`에서 `q=키워드` 부분을 추출합니다.
/// Axum의 `Query` 추출자가 URL의 쿼리 문자열을 이 구조체로 자동 변환합니다.
///
/// Deserialize만 필요한 이유: 클라이언트 → 서버 방향(URL → 구조체)으로만 변환하므로
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// 검색 키워드 (FTS5 검색 문법 사용 가능)
    pub q: String,
}

/// 문서 전문검색을 수행합니다.
///
/// `GET /api/v1/search?q=키워드` → `{ "documents": [...] }`
///
/// ## 추출자 설명
/// `Query(query): Query<SearchQuery>`: URL 쿼리 파라미터를 SearchQuery로 파싱합니다.
/// 예: `?q=hello` → SearchQuery { q: "hello".to_string() }
///
/// ## 에러 처리
/// - 빈 검색어: 400 Bad Request 반환
/// - FTS5 문법 에러: SQLite 에러가 전파되어 500 반환
/// - 정상: 관련도순으로 정렬된 문서 목록 반환 (최대 50건)
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Value>, AppError> {
    // 빈 검색어 방지: 공백만 있는 경우도 trim()으로 걸러냅니다
    if query.q.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Search query cannot be empty".to_string(),
        ));
    }

    // FTS5 전문검색 실행 — 관련도순으로 정렬된 결과를 반환합니다
    let documents = db::search_documents(&state.pool, &query.q).await?;
    Ok(Json(json!({ "documents": documents })))
}
