//! # 데이터 모델 모듈
//!
//! 애플리케이션에서 사용하는 데이터 구조체(struct)들을 정의합니다.
//! 각 하위 모듈은 특정 도메인의 데이터 타입을 담당합니다:
//! - `document`: 문서(Document)와 폴더(Folder) 관련 구조체
//! - `session`: 글쓰기 세션 관련 구조체
//! - `tag`: 태그 관련 구조체
//! - `user`: 사용자(User) 관련 구조체
//!
//! `pub use X::*;`는 하위 모듈의 모든 공개 항목을
//! 이 모듈에서 바로 접근할 수 있게 재공개(re-export)합니다.
//! 예: `crate::models::document::Document` 대신 `crate::models::Document`로 접근 가능

// pub mod: 하위 모듈을 공개(public)로 선언합니다.
// pub이 없으면 이 모듈 내부에서만 접근 가능합니다.
pub mod document;
pub mod session;
pub mod tag;
pub mod user;
pub mod version;

// pub use: 하위 모듈의 항목을 현재 모듈에서 재공개합니다.
// `*`(glob)는 모든 공개 항목을 의미합니다.
// 이렇게 하면 사용하는 쪽에서 `models::Document`처럼 짧게 쓸 수 있습니다.
pub use document::*;
pub use session::*;
pub use tag::*;
pub use user::*;
pub use version::*;
