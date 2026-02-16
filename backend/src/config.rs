//! # 애플리케이션 설정(Configuration) 모듈
//!
//! 환경변수에서 서버 설정값을 읽어오는 모듈입니다.
//! `.env` 파일이나 시스템 환경변수에서 값을 가져옵니다.
//!
//! 설정 항목:
//! - `DATABASE_URL`: SQLite 데이터베이스 경로
//! - `JWT_SECRET`: JWT 토큰 서명에 사용할 비밀키
//! - `DOCUMENTS_PATH`: 마크다운 문서 저장 디렉토리
//! - `UPLOADS_PATH`: 업로드 파일 저장 디렉토리
//! - `HOST`: 서버 바인딩 주소
//! - `PORT`: 서버 포트 번호

// std::env: Rust 표준 라이브러리의 환경변수 모듈
use std::env;

// #[derive(...)]: 자동으로 트레이트 구현을 생성하는 **derive 매크로**
// - Debug: {:?} 포맷으로 출력 가능 (디버깅용 문자열 표현)
// - Clone: .clone() 메서드로 값을 복제 가능
//
// Rust에서 트레이트(trait)는 "이 타입이 할 수 있는 행동"을 정의합니다.
// derive를 사용하면 컴파일러가 보일러플레이트 코드를 자동으로 생성합니다.
#[derive(Debug, Clone)]
/// 애플리케이션 전체 설정을 담는 구조체
///
/// 서버 시작 시 환경변수에서 한 번 읽어온 후,
/// 애플리케이션 전체에서 공유됩니다.
pub struct Config {
    /// SQLite 데이터베이스 파일 경로 (예: "sqlite:data/tecindo.db")
    pub database_url: String,
    /// JWT 토큰 서명/검증에 사용하는 비밀키
    pub jwt_secret: String,
    /// 마크다운 문서가 저장되는 디렉토리 경로
    pub documents_path: String,
    /// 업로드 파일이 저장되는 디렉토리 경로
    pub uploads_path: String,
    /// 서버가 바인딩할 호스트 주소 (기본값: "0.0.0.0")
    pub host: String,
    /// 서버 포트 번호 (기본값: 3000)
    /// u16: 0~65535 범위의 부호 없는 16비트 정수. 포트 번호에 딱 맞는 타입입니다.
    pub port: u16,
}

// impl: 구조체에 메서드를 추가하는 블록
// 다른 언어의 class 내부 메서드와 비슷합니다.
impl Config {
    /// 환경변수에서 설정값을 읽어 Config 인스턴스를 생성합니다.
    ///
    /// # 반환값
    /// - `Ok(Config)`: 필수 환경변수가 모두 있으면 설정 객체 반환
    /// - `Err(VarError)`: 필수 환경변수(DATABASE_URL, JWT_SECRET)가 없으면 에러
    ///
    /// # 에러
    /// `DATABASE_URL`과 `JWT_SECRET`은 필수이며, 없으면 에러가 발생합니다.
    /// 나머지 설정은 기본값이 있어 환경변수가 없어도 동작합니다.
    pub fn from_env() -> Result<Self, env::VarError> {
        // Ok(Self { ... }): 성공 시 Config 인스턴스를 Result::Ok로 감싸 반환
        // Self는 impl 블록의 대상 타입(Config)을 가리킵니다.
        Ok(Self {
            // env::var("KEY"): 환경변수를 읽습니다.
            // 반환 타입은 Result<String, VarError>이며,
            // `?`를 사용해 변수가 없으면 즉시 에러를 반환합니다.
            database_url: env::var("DATABASE_URL")?,  // 필수: 없으면 에러
            jwt_secret: env::var("JWT_SECRET")?,       // 필수: 없으면 에러

            // unwrap_or_else(|_| ...): Result가 Err일 때 실행할 클로저(익명 함수)를 지정합니다.
            // |_|: 클로저의 매개변수. `_`는 "이 값은 사용하지 않겠다"는 의미입니다.
            // .to_string(): &str(문자열 슬라이스)를 String(소유된 문자열)으로 변환
            documents_path: env::var("DOCUMENTS_PATH")
                .unwrap_or_else(|_| "data/documents".to_string()), // 선택: 기본값 제공
            uploads_path: env::var("UPLOADS_PATH")
                .unwrap_or_else(|_| "data/uploads".to_string()),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),

            // 포트 번호는 문자열 → 숫자 변환이 필요합니다.
            // .parse(): 문자열을 다른 타입으로 파싱. 여기서는 u16으로 변환합니다.
            // .unwrap_or(3000): 파싱 실패 시 기본값 3000 사용
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()        // "3000" → 3000u16
                .unwrap_or(3000), // 파싱 실패 시 기본값
        })
    }
}
