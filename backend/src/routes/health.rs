//! # 헬스체크(Health Check) 핸들러
//!
//! 서버가 정상적으로 동작하는지 확인하는 엔드포인트입니다.
//!
//! ## 엔드포인트
//! - `GET /api/v1/health` → `{ "status": "ok" }`
//!
//! 주로 다음 용도로 사용됩니다:
//! - 로드밸런서(Caddy)의 서버 상태 확인
//! - 컨테이너 오케스트레이터(Docker)의 헬스체크
//! - 모니터링 시스템의 가동 시간 추적

use axum::Json;                // JSON 응답 래퍼
use serde_json::{json, Value}; // JSON 생성 유틸리티

/// `GET /health` — 서버 상태를 확인합니다.
///
/// 이 핸들러는 Extractor 없이 작동하는 가장 단순한 형태입니다.
/// State, Path 등 아무것도 필요 없고, 고정된 JSON만 반환합니다.
///
/// 반환 타입이 `Json<Value>`이므로 Axum이 자동으로:
/// - Content-Type: application/json 헤더를 설정하고
/// - HTTP 200 OK 상태 코드를 반환합니다.
///
/// 참고: `Result`를 사용하지 않으므로 이 핸들러는 실패하지 않습니다.
pub async fn health_check() -> Json<Value> {
    // json! 매크로로 JSON 객체를 생성합니다.
    // 결과: {"status": "ok"}
    Json(json!({
        "status": "ok"
    }))
}
