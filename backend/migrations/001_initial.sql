-- ============================================================================
-- Tecindo 초기 데이터베이스 마이그레이션
-- ============================================================================
-- 이 파일은 SQLite 데이터베이스의 초기 스키마(테이블 구조)를 정의합니다.
-- sqlx::migrate! 매크로에 의해 서버 시작 시 자동으로 실행됩니다.
-- 한 번 실행된 마이그레이션은 다시 실행되지 않습니다 (sqlx가 추적).
--
-- SQLite 특징:
-- - 타입: TEXT, INTEGER, REAL, BLOB, NULL (5가지만 존재)
-- - BOOLEAN이 없어 INTEGER 0/1로 대체
-- - DATETIME이 없어 TEXT로 ISO 8601 형식 문자열을 저장
-- - 외래키(FOREIGN KEY)는 기본 비활성. PRAGMA foreign_keys=ON 필요
-- ============================================================================

-- ── 폴더(folders) 테이블 ──
-- 문서를 분류하는 폴더를 저장합니다.
-- parent_id를 통해 폴더 안에 폴더를 넣을 수 있습니다 (트리 구조).
CREATE TABLE folders (
    -- PRIMARY KEY: 이 컬럼이 테이블의 고유 식별자임을 선언합니다.
    -- 같은 값이 두 번 들어갈 수 없습니다.
    id TEXT PRIMARY KEY,  -- UUIDv7: 시간 기반 고유 식별자

    -- REFERENCES: 외래키(Foreign Key) 제약조건.
    -- 이 값은 반드시 folders 테이블의 id 컬럼에 존재하는 값이어야 합니다.
    -- ON DELETE SET NULL: 부모 폴더가 삭제되면 이 값을 NULL로 설정합니다.
    --   (자식 폴더가 최상위 폴더로 이동)
    parent_id TEXT REFERENCES folders(id) ON DELETE SET NULL,

    -- NOT NULL: 이 컬럼에는 NULL 값을 넣을 수 없습니다 (필수 입력).
    name TEXT NOT NULL,    -- 폴더 이름 (예: "일기", "소설")

    slug TEXT NOT NULL,    -- URL 친화적 이름 (예: "diary", "novel")

    -- DEFAULT: 값을 지정하지 않으면 사용할 기본값
    sort_order INTEGER NOT NULL DEFAULT 0,  -- 정렬 순서 (0이 가장 위)

    -- strftime(): SQLite의 날짜/시간 포맷 함수
    -- '%Y-%m-%dT%H:%M:%fZ': ISO 8601 형식 (예: "2024-01-15T09:30:45.123Z")
    -- 'now': 현재 UTC 시각
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),  -- 생성 일시
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))   -- 수정 일시
);

-- ── 문서(documents) 테이블 ──
-- 마크다운 문서의 메타데이터를 저장합니다.
-- 실제 내용은 디스크의 .md 파일에 저장되고, DB에는 메타정보만 기록합니다.
CREATE TABLE documents (
    id TEXT PRIMARY KEY,  -- UUIDv7

    -- 소속 폴더 ID. 외래키로 folders 테이블 참조.
    -- ON DELETE SET NULL: 폴더가 삭제되면 문서가 루트(폴더 없음)로 이동
    folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL,

    title TEXT NOT NULL DEFAULT 'Untitled',  -- 문서 제목 (기본값: 'Untitled')
    slug TEXT NOT NULL,                       -- URL 친화적 제목

    -- UNIQUE: 이 컬럼의 값은 테이블 내에서 유일해야 합니다.
    -- 같은 파일 경로의 문서가 두 개 존재할 수 없습니다.
    file_path TEXT NOT NULL UNIQUE,  -- 디스크의 .md 파일 상대 경로

    word_count INTEGER NOT NULL DEFAULT 0,   -- 단어 수 (내용 저장 시 자동 계산)
    char_count INTEGER NOT NULL DEFAULT 0,   -- 문자 수

    excerpt TEXT,  -- 내용 미리보기 (처음 200자). NULL 허용 (아직 내용이 없을 수 있음)

    -- SQLite에는 BOOLEAN 타입이 없어 INTEGER 0(false)/1(true)로 대체합니다.
    is_pinned INTEGER NOT NULL DEFAULT 0,    -- 고정 여부 (상단 고정)
    is_archived INTEGER NOT NULL DEFAULT 0,  -- 아카이브 여부 (목록에서 숨김)

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- ── 태그(tags) 테이블 ──
-- 문서에 붙일 수 있는 라벨/카테고리입니다.
-- 한 문서에 여러 태그, 한 태그에 여러 문서 → 다대다(N:M) 관계
CREATE TABLE tags (
    id TEXT PRIMARY KEY,           -- UUIDv7
    name TEXT NOT NULL UNIQUE,     -- 태그 이름 (중복 불가)
    color TEXT                     -- 태그 색상 코드 (예: "#FF5733"). NULL 허용
);

-- ── 문서-태그 관계(document_tags) 테이블 ──
-- 다대다(N:M) 관계를 구현하는 연결 테이블(junction table)입니다.
-- 한 문서에 여러 태그를, 한 태그를 여러 문서에 붙일 수 있습니다.
CREATE TABLE document_tags (
    -- ON DELETE CASCADE: 문서가 삭제되면 이 관계 레코드도 자동 삭제됩니다.
    --   (SET NULL과 달리, CASCADE는 레코드 자체를 삭제합니다)
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,

    -- 복합 기본키(Composite Primary Key): 두 컬럼의 조합이 고유해야 합니다.
    -- 같은 문서에 같은 태그를 두 번 붙일 수 없습니다.
    PRIMARY KEY (document_id, tag_id)
);

-- ── 전문검색(FTS5) 가상 테이블 ──
-- SQLite의 FTS5(Full-Text Search 5) 확장을 사용한 전문검색 인덱스입니다.
-- 일반 LIKE '%검색어%'보다 훨씬 빠르고, 형태소 분석/랭킹 등 고급 기능을 제공합니다.
-- VIRTUAL TABLE: 실제 데이터를 저장하지 않고, 인덱스만 관리하는 특수 테이블
CREATE VIRTUAL TABLE documents_fts USING fts5(
    title,                        -- 검색 대상 컬럼: 제목
    content,                      -- 검색 대상 컬럼: 내용
    content='documents',          -- 원본 데이터가 있는 테이블 (contentless가 아닌 external content 모드)
    content_rowid='rowid',        -- documents 테이블의 행 식별자
    tokenize='porter unicode61'   -- 토크나이저: porter(영어 어간 추출) + unicode61(유니코드 지원)
    -- porter: "running" → "run" 등 영어 단어를 기본형으로 변환하여 검색 정확도 향상
    -- unicode61: 유니코드 문자를 올바르게 처리 (한글 등)
);

-- ── 글쓰기 세션(writing_sessions) 테이블 ──
-- 사용자의 글쓰기 세션(시작~종료)을 기록합니다.
-- 글쓰기 통계(얼마나 쓰는지, 언제 쓰는지 등)에 활용됩니다.
CREATE TABLE writing_sessions (
    id TEXT PRIMARY KEY,             -- UUIDv7

    -- 어떤 문서에 대한 세션인지.
    -- ON DELETE CASCADE: 문서가 삭제되면 관련 세션도 모두 삭제
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,

    device_name TEXT,                -- 기기 이름 (예: "맥북", "아이패드"). NULL 허용

    started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),  -- 세션 시작 시각
    ended_at TEXT,                   -- 세션 종료 시각 (진행 중이면 NULL)

    word_count_start INTEGER NOT NULL DEFAULT 0,  -- 세션 시작 시 단어 수
    word_count_end INTEGER           -- 세션 종료 시 단어 수 (진행 중이면 NULL)
    -- 세션 중 작성한 단어 수 = word_count_end - word_count_start
);

-- ── 설정(settings) 테이블 ──
-- 애플리케이션 설정을 키-값(key-value) 쌍으로 저장합니다.
-- 예: { key: "theme", value: "dark" }
CREATE TABLE settings (
    key TEXT PRIMARY KEY,   -- 설정 키 (예: "theme", "language", "font_size")
    value TEXT NOT NULL      -- 설정 값 (JSON 문자열도 가능)
);
