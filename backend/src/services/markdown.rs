//! # 마크다운 파일 I/O 서비스
//!
//! 마크다운(.md) 파일의 읽기/쓰기와 텍스트 통계 관련 유틸리티 함수들을 제공합니다.
//!
//! 이 모듈의 함수들:
//! - `read_markdown()`: 디스크에서 .md 파일을 읽어 문자열로 반환
//! - `write_markdown()`: 문자열을 .md 파일로 디스크에 저장
//! - `count_words()`: 텍스트의 단어 수 계산
//! - `count_chars()`: 텍스트의 문자 수 계산
//! - `generate_file_path()`: 제목으로부터 파일 경로 생성

use crate::error::AppError;
// Path: 파일 경로를 나타내는 불변 참조 타입 (&str과 비슷한 역할)
// PathBuf: 소유된 파일 경로 타입 (String과 비슷한 역할)
//   Path : PathBuf = &str : String
use std::path::{Path, PathBuf};
// tokio::fs: 비동기 파일 시스템 모듈
// 일반 std::fs는 동기(블로킹)이므로, 비동기 서버에서는 tokio::fs를 사용해야 합니다.
// 그렇지 않으면 파일 I/O 중에 다른 요청을 처리할 수 없습니다.
use tokio::fs;

/// 디스크에서 마크다운 파일을 읽어 문자열로 반환합니다.
///
/// # 매개변수
/// - `documents_path`: 문서 저장 루트 디렉토리 (예: "data/documents")
/// - `file_path`: 루트 디렉토리 기준 상대 경로 (예: "my-folder/my-doc.md")
///
/// # 반환값
/// - `Ok(String)`: 파일 내용
/// - `Err(AppError::Io)`: 파일을 찾을 수 없거나 읽을 수 없는 경우
pub async fn read_markdown(documents_path: &str, file_path: &str) -> Result<String, AppError> {
    // PathBuf::from(): 문자열을 경로 타입으로 변환
    // .join(): 두 경로를 합칩니다 (OS에 맞는 경로 구분자 사용)
    // 예: "data/documents" + "folder/doc.md" → "data/documents/folder/doc.md"
    let full_path = PathBuf::from(documents_path).join(file_path);
    // fs::read_to_string(): 파일 전체를 UTF-8 문자열로 읽습니다 (비동기)
    // &full_path: PathBuf의 참조를 전달 (&를 통해 소유권 이동 없이 빌려줌)
    let content = fs::read_to_string(&full_path).await?;
    Ok(content)
}

/// 마크다운 내용을 디스크 파일에 저장합니다.
///
/// 부모 디렉토리가 없으면 자동으로 생성합니다.
///
/// # 매개변수
/// - `documents_path`: 문서 저장 루트 디렉토리
/// - `file_path`: 상대 파일 경로
/// - `content`: 저장할 마크다운 내용
pub async fn write_markdown(
    documents_path: &str,
    file_path: &str,
    content: &str,
) -> Result<(), AppError> {
    let full_path = PathBuf::from(documents_path).join(file_path);

    // 부모 디렉토리가 존재하는지 확인하고 없으면 생성합니다.
    // .parent(): 파일 경로에서 디렉토리 부분만 추출
    // 예: "data/docs/folder/doc.md".parent() → "data/docs/folder"
    // if let Some(parent) = ...: parent()가 Some을 반환하면 실행
    if let Some(parent) = full_path.parent() {
        // create_dir_all: 중간 디렉토리까지 모두 생성 (이미 있으면 무시)
        fs::create_dir_all(parent).await?;
    }

    // fs::write(): 파일에 내용을 씁니다 (파일이 있으면 덮어쓰기, 없으면 새로 생성)
    fs::write(&full_path, content).await?;
    Ok(())
}

/// 텍스트의 단어 수를 계산합니다.
///
/// 공백(스페이스, 탭, 줄바꿈)으로 분리하여 단어를 셉니다.
///
/// # 매개변수
/// - `text`: 단어 수를 셀 텍스트 (&str: 문자열의 불변 참조)
///
/// # 반환값
/// - `usize`: 단어 수 (부호 없는 정수, 양수만 가능)
pub fn count_words(text: &str) -> usize {
    // .split_whitespace(): 공백으로 문자열을 분리하여 이터레이터를 반환
    // .count(): 이터레이터의 항목 수를 셉니다
    // 이 함수는 async가 아닙니다 — CPU 작업만 하므로 비동기가 필요 없습니다.
    text.split_whitespace().count()
}

/// 텍스트의 문자 수를 계산합니다.
///
/// 유니코드 문자 단위로 셉니다 (한글 1자 = 1문자).
/// `.len()`은 바이트 수를 반환하므로 한글 등에는 부적합합니다.
pub fn count_chars(text: &str) -> usize {
    // .chars(): 유니코드 문자(char) 단위 이터레이터
    // .count(): 문자 수를 셉니다
    // 참고: text.len()은 바이트 수를 반환합니다 (한글 1자 = 3바이트)
    text.chars().count()
}

/// 문서 제목과 폴더 정보로 파일 저장 경로를 생성합니다.
///
/// # 매개변수
/// - `title`: 문서 제목
/// - `folder_slug`: 폴더의 slug (None이면 루트에 저장)
///
/// # 반환값
/// 파일 경로 문자열 (예: "my-folder/my-title.md" 또는 "my-title.md")
///
/// # 예시
/// ```
/// generate_file_path("나의 첫 글", Some("일기")) → "일기/나의-첫-글.md"
/// generate_file_path("나의 첫 글", None) → "나의-첫-글.md"
/// ```
pub fn generate_file_path(title: &str, folder_slug: Option<&str>) -> String {
    // slug::slugify(): 제목을 URL 친화적인 문자열로 변환합니다.
    // 예: "Hello World!" → "hello-world", "나의 글" → "나의-글"
    let slug = slug::slugify(title);
    // if let Some(folder) = ...: Option이 Some이면 값을 추출하여 folder에 대입
    if let Some(folder) = folder_slug {
        // format!: 포맷 문자열 매크로. Python의 f-string과 비슷합니다.
        format!("{}/{}.md", folder, slug)
    } else {
        format!("{}.md", slug)
    }
}
