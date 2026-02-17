//! # 문서(Document) 라우트 핸들러
//!
//! 문서의 CRUD(생성/조회/수정/삭제)와 내용 읽기/쓰기를 처리하는
//! HTTP 핸들러 함수들입니다.
//!
//! ## 엔드포인트
//! - `GET    /api/v1/documents`             → 문서 목록 조회
//! - `POST   /api/v1/documents`             → 새 문서 생성
//! - `GET    /api/v1/documents/:id`         → 단일 문서 조회
//! - `PATCH  /api/v1/documents/:id`         → 문서 수정 (부분 업데이트)
//! - `DELETE /api/v1/documents/:id`         → 문서 삭제
//! - `GET    /api/v1/documents/:id/content` → 문서 내용(마크다운) 조회
//! - `PUT    /api/v1/documents/:id/content` → 문서 내용 수정
//!
//! ## Axum 핸들러 패턴
//! Axum 핸들러는 **Extractor(추출기)**를 매개변수로 받습니다.
//! Extractor는 HTTP 요청에서 데이터를 자동으로 추출합니다:
//! - `State(state)`: 앱 전역 상태 (DB 풀, 설정 등)
//! - `Path(id)`: URL 경로 파라미터 (예: /documents/:id에서 id)
//! - `Json(body)`: 요청 본문을 JSON으로 파싱하여 구조체로 변환
//!
//! 반환 타입이 `Result<T, AppError>`이면, Axum이 자동으로:
//! - `Ok(T)` → T를 HTTP 응답으로 변환 (IntoResponse 트레이트 사용)
//! - `Err(AppError)` → AppError를 에러 JSON 응답으로 변환

use crate::{
    db,
    error::AppError,
    middleware::auth::AuthUser,
    models::*,
    services,
};
use axum::{
    extract::{Path, Query, State}, // Axum Extractor: 요청에서 데이터 추출
    http::{StatusCode, header, HeaderMap},
    response::IntoResponse,
    Json,                    // JSON 요청/응답 래퍼
};
use serde::Deserialize;
use serde_json::{json, Value}; // JSON 값 생성 유틸리티
use sqlx::SqlitePool;          // SQLite 연결 풀 타입

// #[derive(Clone)]: AppState가 Clone 트레이트를 구현하게 합니다.
// Axum의 State Extractor는 내부적으로 AppState를 clone하므로 필수입니다.
// SqlitePool은 Arc<내부상태>를 사용하므로 clone해도 실제 풀이 복제되지 않습니다.

/// 애플리케이션 공유 상태
///
/// 모든 요청 핸들러가 `State(state): State<AppState>`로 접근합니다.
/// Axum의 의존성 주입(Dependency Injection) 메커니즘입니다.
#[derive(Clone)]
pub struct AppState {
    /// SQLite 연결 풀 (내부적으로 Arc로 공유)
    pub pool: SqlitePool,
    /// 마크다운 문서 저장 디렉토리 경로
    pub documents_path: String,
    /// JWT 토큰 서명용 비밀키
    pub jwt_secret: String,
    /// 문서당 최대 버전 보관 수
    pub max_document_versions: u32,
    /// 버전 생성 최소 간격 (분)
    pub version_interval_minutes: u32,
}

/// 문서 목록 조회용 쿼리 파라미터
#[derive(Deserialize)]
pub struct ListDocumentsQuery {
    /// 특정 태그가 붙은 문서만 필터링
    pub tag_id: Option<String>,
}

/// `GET /documents` — 전체 문서 목록을 조회합니다.
///
/// # 쿼리 파라미터
/// - `tag_id` (선택): 특정 태그가 붙은 문서만 반환
///
/// # 반환값
/// `{ "documents": [...] }` 형태의 JSON
pub async fn list_documents(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<Value>, AppError> {
    let documents = if let Some(tag_id) = &query.tag_id {
        db::list_documents_by_tag(&state.pool, tag_id, &auth_user.user_id).await?
    } else {
        db::list_documents(&state.pool, &auth_user.user_id).await?
    };
    Ok(Json(json!({ "documents": documents })))
}

/// `GET /documents/:id` — 단일 문서를 조회합니다.
///
/// # Extractor
/// - `Path(id)`: URL의 `:id` 부분을 String으로 추출합니다.
///   Path<String>은 `/documents/abc-123`에서 `"abc-123"`을 추출합니다.
pub async fn get_document(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Document>, AppError> {
    let document = db::get_document(&state.pool, &id, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(document))
}

/// `POST /documents` — 새 문서를 생성합니다.
///
/// 빈 마크다운 파일을 디스크에 생성하고, DB에 메타데이터를 저장합니다.
///
/// # Extractor
/// - `Json(req)`: HTTP 요청 본문(body)을 JSON으로 파싱하여
///   `CreateDocumentRequest` 구조체로 변환합니다.
///   Axum이 Content-Type 확인과 파싱을 자동으로 처리합니다.
pub async fn create_document(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<Document>, AppError> {
    let id = uuid::Uuid::now_v7().to_string();

    let title = if let Some(t) = req.title.as_deref() {
        t.to_string()
    } else {
        let existing = db::list_untitled_titles(
            &state.pool,
            req.folder_id.as_deref(),
            &auth_user.user_id,
        ).await?;
        generate_untitled_name(&existing)
    };

    let folder_slug = if let Some(folder_id) = &req.folder_id {
        let folder = db::get_folder(&state.pool, folder_id, &auth_user.user_id).await?;
        folder.map(|f| f.slug)
    } else {
        None
    };

    let file_path = services::generate_file_path(&title, folder_slug.as_deref(), &id);
    let slug = slug::slugify(&title);

    services::write_markdown(&state.documents_path, &file_path, "").await?;

    let req_with_title = CreateDocumentRequest {
        title: Some(title),
        folder_id: req.folder_id,
    };
    let document = db::create_document(&state.pool, &id, &req_with_title, file_path, slug, &auth_user.user_id).await?;
    Ok(Json(document))
}

/// 같은 폴더 내 기존 Untitled 제목들을 보고 다음 고유 이름을 생성합니다.
/// 예: [] → "Untitled", ["Untitled"] → "Untitled_2", ["Untitled", "Untitled_2"] → "Untitled_3"
fn generate_untitled_name(existing: &[String]) -> String {
    if !existing.iter().any(|t| t == "Untitled") {
        return "Untitled".to_string();
    }
    let mut max_n = 1u32;
    for title in existing {
        if let Some(suffix) = title.strip_prefix("Untitled_") {
            if let Ok(n) = suffix.parse::<u32>() {
                if n >= max_n {
                    max_n = n + 1;
                }
            }
        }
    }
    if max_n == 1 { max_n = 2; }
    format!("Untitled_{}", max_n)
}

/// `PATCH /documents/:id` — 문서 메타데이터를 수정합니다.
///
/// 요청 본문에 포함된 필드만 업데이트합니다 (부분 업데이트).
/// 예: `{ "title": "새 제목" }`으로 제목만 변경 가능
pub async fn update_document(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<Document>, AppError> {
    // folder_id가 지정된 경우, 해당 폴더가 현재 사용자 소유인지 검증
    if let Some(Some(folder_id)) = &req.folder_id {
        db::get_folder(&state.pool, folder_id, &auth_user.user_id)
            .await?
            .ok_or(AppError::NotFound)?;
    }

    let document = db::update_document(&state.pool, &id, &req, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(document))
}

/// `DELETE /documents/:id` — 문서를 삭제합니다.
///
/// DB 레코드와 디스크의 .md 파일을 모두 삭제합니다.
/// 성공 시 HTTP 204 No Content를 반환합니다 (본문 없음).
pub async fn delete_document(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let document = db::get_document(&state.pool, &id, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let deleted = db::delete_document(&state.pool, &id, &auth_user.user_id).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }

    let file_path = std::path::PathBuf::from(&state.documents_path).join(&document.file_path);
    let _ = tokio::fs::remove_file(file_path).await;

    Ok(StatusCode::NO_CONTENT)
}

/// `GET /documents/:id/content` — 문서의 마크다운 내용을 조회합니다.
///
/// 디스크의 .md 파일을 읽어 JSON으로 반환합니다.
/// 응답: `{ "content": "# 제목\n\n본문..." }`
pub async fn get_document_content(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<DocumentContent>, AppError> {
    let document = db::get_document(&state.pool, &id, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let content = services::read_markdown(&state.documents_path, &document.file_path).await?;
    Ok(Json(DocumentContent { content }))
}

/// `PUT /documents/:id/content` — 문서의 마크다운 내용을 수정합니다.
///
/// 디스크 파일을 덮어쓰고, DB의 단어 수/글자 수/미리보기를 업데이트합니다.
/// 또한 전문검색(FTS5) 인덱스도 갱신합니다.
/// 성공 시 HTTP 204 No Content를 반환합니다.
pub async fn update_document_content(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<DocumentContent>,
) -> Result<StatusCode, AppError> {
    let document = db::get_document(&state.pool, &id, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // FTS 인덱스 업데이트를 위해 이전 내용을 읽어둡니다.
    // .ok(): Result를 Option으로 변환 (에러를 무시하고 None으로 처리)
    let old_content = services::read_markdown(&state.documents_path, &document.file_path)
        .await
        .ok();

    // 새 내용을 디스크 파일에 저장합니다.
    services::write_markdown(&state.documents_path, &document.file_path, &req.content).await?;

    // 단어 수와 글자 수를 계산합니다.
    // as i64: usize(부호 없는 정수)를 i64(부호 있는 정수)로 캐스팅합니다.
    //   SQLite의 INTEGER는 i64에 매핑됩니다.
    let word_count = services::count_words(&req.content) as i64;
    let char_count = services::count_chars(&req.content) as i64;

    // 미리보기(excerpt): 내용의 처음 200자를 추출합니다.
    // chars().take()를 사용하여 유니코드 문자 단위로 안전하게 자릅니다.
    let excerpt = if req.content.is_empty() {
        None
    } else if req.content.chars().count() > 200 {
        Some(req.content.chars().take(200).collect::<String>())
    } else {
        Some(req.content.clone())
    };

    // DB의 문서 메타데이터(단어 수, 글자 수, 미리보기, 수정일)를 업데이트합니다.
    sqlx::query(
        r#"
        UPDATE documents
        SET word_count = ?, char_count = ?, excerpt = ?,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(word_count)
    .bind(char_count)
    .bind(excerpt)
    .bind(&id)
    .bind(&auth_user.user_id)
    .execute(&state.pool)
    .await?;

    // 설정된 간격이 지났을 때만 버전 스냅샷 저장 (best-effort)
    if db::should_create_version(&state.pool, &id, state.version_interval_minutes)
        .await
        .unwrap_or(true)
    {
        let _ = db::create_version(&state.pool, &id, &req.content, word_count, char_count).await;
        let _ = db::prune_versions(&state.pool, &id, state.max_document_versions).await;
    }

    // FTS5(전문검색) 인덱스를 갱신합니다.
    // 검색 기능이 최신 내용을 반영할 수 있도록 합니다.
    // .as_deref(): Option<String> → Option<&str> 변환
    db::index_document(
        &state.pool,
        &id,
        &document.title,
        &req.content,
        Some(&document.title),
        old_content.as_deref(),
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// `GET /documents/:id/export/pdf` — 문서를 pandoc으로 PDF 변환 후 다운로드합니다.
pub async fn export_document_pdf(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let document = db::get_document(&state.pool, &id, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let content = services::read_markdown(&state.documents_path, &document.file_path).await?;

    // 요청별 고유 임시파일 (동시 요청 충돌 방지)
    let req_id = uuid::Uuid::now_v7();
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join(format!("tecindo-{}.md", req_id));
    let output_path = temp_dir.join(format!("tecindo-{}.pdf", req_id));

    let full_content = format!(
        "---\ntitle: \"{}\"\n---\n\n{}",
        document.title.replace('\\', "\\\\").replace('"', "\\\""),
        content
    );
    tokio::fs::write(&input_path, full_content.as_bytes()).await?;

    // CJK 폰트: 환경변수 TECINDO_CJK_FONT로 설정 가능
    let cjk_font = std::env::var("TECINDO_CJK_FONT")
        .unwrap_or_else(|_| "Apple SD Gothic Neo".to_string());

    // 60초 timeout
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        tokio::process::Command::new("pandoc")
            .arg(&input_path)
            .arg("-o")
            .arg(&output_path)
            .arg("--pdf-engine=xelatex")
            .arg("-V")
            .arg(format!("CJKmainfont={}", cjk_font))
            .arg("-V")
            .arg("geometry:margin=2.5cm")
            .output(),
    )
    .await;

    let _ = tokio::fs::remove_file(&input_path).await;

    let output = match result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            let _ = tokio::fs::remove_file(&output_path).await;
            return Err(AppError::Internal(format!("pandoc 실행 실패: {}", e)));
        }
        Err(_) => {
            let _ = tokio::fs::remove_file(&output_path).await;
            return Err(AppError::Internal("PDF 변환 시간 초과 (60초)".to_string()));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = tokio::fs::remove_file(&output_path).await;
        return Err(AppError::Internal(format!("PDF 변환 실패: {}", stderr)));
    }

    let pdf_bytes = tokio::fs::read(&output_path).await?;
    let _ = tokio::fs::remove_file(&output_path).await;

    let slug = slug::slugify(&document.title);
    let filename = if slug.is_empty() { "document".to_string() } else { slug };

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}.pdf\"", filename).parse().unwrap(),
    );

    Ok((headers, pdf_bytes))
}
