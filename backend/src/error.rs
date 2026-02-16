//! # 에러 처리 모듈
//!
//! 애플리케이션에서 발생할 수 있는 모든 에러 타입을 정의합니다.
//! Rust에서는 예외(exception) 대신 `Result<T, E>` 타입으로 에러를 처리합니다.
//!
//! 이 모듈의 핵심:
//! - `AppError` 열거형(enum): 모든 에러 종류를 하나의 타입으로 통합
//! - `IntoResponse` 구현: 에러를 HTTP 응답으로 자동 변환

use axum::{
    http::StatusCode,                     // HTTP 상태 코드 (200, 404, 500 등)
    response::{IntoResponse, Response},   // Axum의 응답 변환 트레이트
    Json,                                 // JSON 응답 래퍼
};
use serde_json::json; // json! 매크로: JSON 객체를 간편하게 생성
use thiserror::Error; // thiserror: 커스텀 에러 타입을 쉽게 만들어주는 매크로 크레이트

// #[derive(Debug, Error)]: 두 가지 derive 매크로를 적용합니다.
// - Debug: 디버깅용 출력 ({:?})
// - Error (thiserror): std::error::Error 트레이트를 자동 구현.
//   #[error("...")] 어트리뷰트로 Display 트레이트(사람이 읽을 에러 메시지)도 자동 생성합니다.
//
// enum(열거형): 여러 가지 가능한 값 중 하나를 나타내는 타입.
// 다른 언어의 union type이나 sealed class와 비슷합니다.
// match 문으로 모든 경우를 빠짐없이 처리해야 합니다 (exhaustive matching).

/// 애플리케이션에서 발생할 수 있는 모든 에러 종류
///
/// 각 에러 variant는 적절한 HTTP 상태 코드와 메시지로 변환됩니다.
/// 핸들러에서 `Result<T, AppError>`를 반환하면,
/// Axum이 자동으로 `IntoResponse`를 호출하여 HTTP 응답으로 변환합니다.
#[derive(Debug, Error)]
pub enum AppError {
    // #[error("...")]: 이 variant의 Display 메시지를 정의합니다.
    // .to_string()이나 println!("{}", err)로 출력할 때 이 메시지가 사용됩니다.

    /// 요청한 리소스를 찾을 수 없음 (HTTP 404)
    #[error("Resource not found")]
    NotFound,

    /// 잘못된 요청 (HTTP 400)
    /// String을 포함하여 구체적인 에러 메시지를 전달합니다.
    /// {0}은 첫 번째 필드(String)를 참조하는 포맷 문법입니다.
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// 서버 내부 오류 (HTTP 500)
    #[error("Internal error: {0}")]
    Internal(String),

    /// 데이터베이스 오류 (HTTP 500)
    /// #[from]: sqlx::Error를 AppError로 자동 변환하는 From 트레이트를 구현합니다.
    /// 이를 통해 sqlx 함수에서 반환된 에러에 `?` 연산자를 사용하면
    /// 자동으로 AppError::Database로 변환됩니다.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// 파일 입출력 오류 (HTTP 500)
    /// #[from]: std::io::Error → AppError::Io 자동 변환
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 인증 실패 (HTTP 401)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// 리소스 충돌 (HTTP 409)
    #[error("Conflict: {0}")]
    Conflict(String),
}

// impl IntoResponse for AppError:
// Axum의 IntoResponse 트레이트를 AppError에 구현합니다.
// 이를 통해 핸들러가 Err(AppError)를 반환하면,
// Axum이 자동으로 이 메서드를 호출하여 적절한 HTTP 응답을 생성합니다.
//
// 트레이트 구현(impl Trait for Type)은 Rust의 핵심 패턴입니다.
// 다른 언어의 인터페이스 구현(implements)과 비슷하지만,
// 기존 타입에 새 행동을 추가할 수 있다는 점이 다릅니다.
impl IntoResponse for AppError {
    /// AppError를 HTTP 응답으로 변환합니다.
    ///
    /// 각 에러 종류에 따라 적절한 HTTP 상태 코드와 JSON 에러 메시지를 생성합니다.
    /// 내부 에러(Database, IO, Internal)는 실제 에러 내용을 로그에만 기록하고,
    /// 클라이언트에는 일반적인 메시지만 반환합니다 (보안을 위해).
    fn into_response(self) -> Response {
        // match: 패턴 매칭. enum의 각 variant에 대해 다른 처리를 합니다.
        // 모든 variant를 빠짐없이 처리해야 합니다 (exhaustive).
        // (status, code, message) 튜플을 반환합니다.
        let (status, code, message) = match self {
            // Self::NotFound → (404, "not_found", "Resource not found")
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found", self.to_string()),

            // ref: 패턴 매칭에서 값을 이동(move)하지 않고 참조만 빌려옵니다.
            // Rust의 소유권(ownership) 시스템 때문에 필요합니다.
            // self가 이미 match에서 사용 중이므로, 내부 값은 참조로 접근합니다.
            AppError::BadRequest(ref msg) => {
                (StatusCode::BAD_REQUEST, "bad_request", msg.clone())
            }
            AppError::Internal(ref msg) => {
                // 내부 에러는 로그에 기록 (서버 관리자용)
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    // 클라이언트에는 일반적인 메시지만 반환 (보안: 내부 구현 노출 방지)
                    "An internal error occurred".to_string(),
                )
            }
            AppError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "A database error occurred".to_string(),
                )
            }
            AppError::Io(ref e) => {
                tracing::error!("IO error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "io_error",
                    "An IO error occurred".to_string(),
                )
            }
            AppError::Unauthorized(ref msg) => {
                (StatusCode::UNAUTHORIZED, "unauthorized", msg.clone())
            }
            AppError::Conflict(ref msg) => {
                (StatusCode::CONFLICT, "conflict", msg.clone())
            }
        };

        // JSON 응답 본문을 생성합니다.
        // json! 매크로: Rust 코드로 JSON 구조를 직관적으로 작성할 수 있게 합니다.
        // 결과: { "error": { "code": "not_found", "message": "Resource not found" } }
        let body = Json(json!({
            "error": {
                "code": code,
                "message": message
            }
        }));

        // (StatusCode, Json<Value>)를 Response로 변환합니다.
        // Axum은 튜플 (상태코드, 본문)을 자동으로 HTTP 응답으로 변환합니다.
        // .into_response(): IntoResponse 트레이트의 메서드를 호출
        (status, body).into_response()
    }
}
