//! # 라우트 핸들러 모듈
//!
//! HTTP 요청을 처리하는 핸들러 함수들을 모아둔 모듈입니다.
//! Axum에서 핸들러는 HTTP 요청을 받아 응답을 반환하는 async 함수입니다.
//!
//! 각 하위 모듈:
//! - `auth`: 인증 관련 (회원가입, 로그인, 토큰 갱신, 로그아웃)
//! - `documents`: 문서 CRUD 핸들러
//! - `folders`: 폴더 CRUD 핸들러
//! - `health`: 서버 상태 확인 (헬스체크)
//! - `search`: 전문검색(FTS5) 핸들러
//! - `sessions`: 글쓰기 세션 핸들러
//! - `tags`: 태그 CRUD 및 문서-태그 관계 핸들러

pub mod auth;
pub mod documents;
pub mod folders;
pub mod health;
pub mod search;
pub mod sessions;
pub mod tags;
pub mod versions;

// 각 모듈의 핸들러 함수들을 재공개하여
// main.rs에서 `routes::list_documents`처럼 바로 접근 가능하게 합니다.
pub use documents::*;
pub use folders::*;
pub use health::*;
pub use search::*;
pub use sessions::*;
pub use tags::*;
pub use versions::*;
