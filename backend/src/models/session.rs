//! # 글쓰기 세션 모델 정의
//!
//! 사용자의 글쓰기 세션을 추적하기 위한 데이터 구조체들을 정의합니다.
//! 세션은 "언제, 어떤 기기에서, 얼마나 썼는지"를 기록하여
//! 글쓰기 습관을 분석하는 데 활용됩니다.
//!
//! ## 세션 흐름
//! 1. 에디터를 열면 `CreateSessionRequest`로 세션 시작
//! 2. 에디터를 닫거나 일정 시간 후 `EndSessionRequest`로 세션 종료
//! 3. 시작/종료 시점의 단어 수 차이로 작성량을 측정

use serde::{Deserialize, Serialize};

/// 글쓰기 세션 엔티티 — DB의 `writing_sessions` 테이블 한 행에 대응합니다.
///
/// 세션은 특정 문서에 대한 한 번의 글쓰기 활동을 나타냅니다.
/// 시작 시점과 종료 시점의 단어 수를 비교하여 작성량을 추적합니다.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WritingSession {
    /// 세션 고유 식별자 (UUIDv7)
    pub id: String,
    /// 이 세션이 속한 문서의 ID (외래키)
    pub document_id: String,
    /// 글쓰기에 사용한 기기 이름 (예: "MacBook Pro", "iPad")
    /// 선택 항목이므로 Option 타입
    pub device_name: Option<String>,
    /// 세션 시작 시각 (ISO 8601 형식: "2026-02-16T12:00:00.000Z")
    /// DB에서 자동 생성되므로 생성 요청 시에는 필요 없음
    pub started_at: String,
    /// 세션 종료 시각 — None이면 아직 진행 중인 세션
    /// Option을 사용하는 이유: 세션을 시작할 때는 종료 시각을 알 수 없기 때문
    pub ended_at: Option<String>,
    /// 세션 시작 시점의 문서 단어 수
    pub word_count_start: i64,
    /// 세션 종료 시점의 문서 단어 수 — None이면 아직 미종료
    /// (word_count_end - word_count_start)로 이번 세션에서 쓴 단어 수를 계산할 수 있음
    pub word_count_end: Option<i64>,
}

/// 세션 시작 요청 — `POST /api/v1/documents/:id/sessions`의 요청 본문에 해당합니다.
///
/// 문서 ID는 URL 경로에서 추출하므로 여기에는 포함되지 않습니다.
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    /// 기기 이름 (선택 — 어떤 기기에서 글을 쓰는지 기록용)
    pub device_name: Option<String>,
    /// 세션 시작 시점의 단어 수 (선택 — 없으면 0으로 처리)
    pub word_count_start: Option<i64>,
}

/// 세션 종료 요청 — `PATCH /api/v1/sessions/:id`의 요청 본문에 해당합니다.
///
/// 세션을 종료하면 서버가 자동으로 ended_at을 현재 시각으로 설정합니다.
#[derive(Debug, Deserialize)]
pub struct EndSessionRequest {
    /// 세션 종료 시점의 단어 수 (선택)
    pub word_count_end: Option<i64>,
}
