//! # 태그 모델 정의
//!
//! 태그(Tag) 시스템에서 사용하는 데이터 구조체들을 정의합니다.
//! 태그는 문서를 분류하고 검색하기 위한 라벨입니다.
//!
//! ## 구조체 역할
//! - `Tag`: 데이터베이스에 저장된 태그를 표현 (응답용)
//! - `CreateTagRequest`: 새 태그 생성 시 클라이언트가 보내는 JSON 본문
//! - `UpdateTagRequest`: 태그 수정 시 클라이언트가 보내는 JSON 본문
//! - `AddTagToDocumentRequest`: 문서에 태그를 연결할 때 클라이언트가 보내는 JSON 본문

use serde::{Deserialize, Serialize};

/// 태그 엔티티 — DB의 `tags` 테이블 한 행(row)에 대응합니다.
///
/// # derive 매크로 설명
/// - `Serialize`: 이 구조체를 JSON으로 변환할 수 있게 합니다 (API 응답 시 사용)
/// - `Deserialize`: JSON을 이 구조체로 변환할 수 있게 합니다
/// - `sqlx::FromRow`: SQL 쿼리 결과(행)를 이 구조체로 자동 매핑합니다
/// - `Clone`: 값을 복제할 수 있게 합니다 (.clone() 메서드 제공)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    /// 태그 고유 식별자 (UUIDv7 형식 문자열)
    pub id: String,
    /// 태그 이름 (예: "일기", "에세이", "아이디어")
    pub name: String,
    /// 태그 색상 코드 (예: "#FF5733"), 없을 수도 있으므로 Option 타입
    /// Option<T>: 값이 있으면 Some(값), 없으면 None — Rust의 null 안전 처리 방식
    pub color: Option<String>,
}

/// 태그 생성 요청 — `POST /api/v1/tags`의 요청 본문(body)에 해당합니다.
///
/// Serialize를 빼고 Deserialize만 derive한 이유:
/// 이 구조체는 클라이언트 → 서버 방향으로만 사용되므로
/// JSON 파싱(Deserialize)만 필요하고, JSON 생성(Serialize)은 불필요합니다.
#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    /// 생성할 태그 이름 (필수)
    pub name: String,
    /// 태그 색상 코드 (선택)
    pub color: Option<String>,
}

/// 태그 수정 요청 — `PATCH /api/v1/tags/:id`의 요청 본문에 해당합니다.
///
/// 모든 필드가 Option인 이유: PATCH는 부분 업데이트(partial update)를 의미합니다.
/// 클라이언트가 변경하고 싶은 필드만 보내면 되므로, 빠진 필드는 None으로 처리됩니다.
/// 예: `{ "name": "새이름" }` → name만 변경, color는 None이므로 그대로 유지
#[derive(Debug, Deserialize)]
pub struct UpdateTagRequest {
    /// 변경할 태그 이름 (None이면 변경하지 않음)
    pub name: Option<String>,
    /// 변경할 태그 색상 (None이면 변경하지 않음)
    pub color: Option<String>,
}

/// 문서에 태그 추가 요청 — `POST /api/v1/documents/:id/tags`의 요청 본문에 해당합니다.
///
/// 문서 ID는 URL 경로 파라미터(:id)에서 가져오고,
/// 어떤 태그를 연결할지는 이 요청 본문의 tag_id로 지정합니다.
#[derive(Debug, Deserialize)]
pub struct AddTagToDocumentRequest {
    /// 문서에 연결할 태그의 ID
    pub tag_id: String,
}
