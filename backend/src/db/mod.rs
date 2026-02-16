//! # 데이터베이스 접근 계층 (Data Access Layer)
//!
//! 데이터베이스와 직접 상호작용하는 함수들을 모아둔 모듈입니다.
//! 라우트 핸들러(routes/)에서 이 모듈의 함수를 호출하여 DB 작업을 수행합니다.
//!
//! 각 하위 모듈:
//! - `documents`: 문서와 폴더의 CRUD(생성/조회/수정/삭제) 쿼리
//! - `search`: 전문검색(FTS5) 인덱스 관리 쿼리
//! - `sessions`: 글쓰기 세션 관련 쿼리
//! - `tags`: 태그 CRUD 및 문서-태그 관계 쿼리
//! - `users`: 사용자 인증 관련 쿼리

pub mod documents;
pub mod search;
pub mod sessions;
pub mod tags;
pub mod users;

// 하위 모듈의 모든 공개 함수를 재공개(re-export)하여
// `crate::db::list_documents`처럼 바로 접근할 수 있게 합니다.
pub use documents::*;
pub use search::*;
pub use sessions::*;
pub use tags::*;
