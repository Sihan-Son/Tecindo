# Tecindo Database Schema

Engine: SQLite (via sqlx)
Migrations: `backend/migrations/` (sqlx::migrate! macro)

---

## Tables

### folders

문서를 분류하는 폴더. `parent_id` self-reference로 트리 구조.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PK | UUIDv7 |
| parent_id | TEXT | FK → folders(id) ON DELETE SET NULL | 부모 폴더 (NULL = 최상위) |
| user_id | TEXT | FK → users(id), IDX | 소유 사용자 (migration 003) |
| name | TEXT | NOT NULL | 폴더 이름 |
| slug | TEXT | NOT NULL | URL-friendly 이름 |
| sort_order | INTEGER | NOT NULL DEFAULT 0 | 정렬 순서 |
| created_at | TEXT | NOT NULL DEFAULT now | ISO 8601 UTC |
| updated_at | TEXT | NOT NULL DEFAULT now | ISO 8601 UTC |

### documents

마크다운 문서 메타데이터. 실제 내용은 디스크 .md 파일에 저장.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PK | UUIDv7 |
| folder_id | TEXT | FK → folders(id) ON DELETE SET NULL | 소속 폴더 |
| user_id | TEXT | FK → users(id), IDX | 소유 사용자 (migration 003) |
| title | TEXT | NOT NULL DEFAULT 'Untitled' | 문서 제목 |
| slug | TEXT | NOT NULL | URL-friendly 제목 |
| file_path | TEXT | NOT NULL UNIQUE | .md 파일 상대 경로 |
| word_count | INTEGER | NOT NULL DEFAULT 0 | 단어 수 |
| char_count | INTEGER | NOT NULL DEFAULT 0 | 문자 수 |
| excerpt | TEXT | | 내용 미리보기 (처음 200자) |
| is_pinned | INTEGER | NOT NULL DEFAULT 0 | 고정 여부 (0/1) |
| is_archived | INTEGER | NOT NULL DEFAULT 0 | 아카이브 여부 |
| created_at | TEXT | NOT NULL DEFAULT now | ISO 8601 UTC |
| updated_at | TEXT | NOT NULL DEFAULT now | ISO 8601 UTC |

### tags

문서에 붙이는 태그. 프론트엔드에서 인라인 입력 시 자동 생성.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PK | UUIDv7 |
| user_id | TEXT | FK → users(id), IDX | 소유 사용자 (migration 003) |
| name | TEXT | NOT NULL, UNIQUE(user_id, name) | 태그 이름 (사용자별 유니크) |
| color | TEXT | | 색상 코드 (예: #3b82f6) |

### document_tags

문서-태그 다대다 관계 (junction table).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| document_id | TEXT | NOT NULL, FK → documents(id) CASCADE | 문서 ID |
| tag_id | TEXT | NOT NULL, FK → tags(id) CASCADE | 태그 ID |
| | | PK (document_id, tag_id) | 복합 기본키 |

### documents_fts (FTS5 virtual table)

전문검색 인덱스. `documents` 테이블의 title + content를 인덱싱.

- Tokenizer: `porter unicode61`
- External content mode: `content='documents'`

### writing_sessions

글쓰기 세션 기록. 시작/종료 시점의 단어 수를 추적.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PK | UUIDv7 |
| document_id | TEXT | NOT NULL, FK → documents(id) CASCADE | 문서 ID |
| device_name | TEXT | | 기기 이름 |
| started_at | TEXT | NOT NULL DEFAULT now | 세션 시작 |
| ended_at | TEXT | | 세션 종료 (진행 중이면 NULL) |
| word_count_start | INTEGER | NOT NULL DEFAULT 0 | 시작 시 단어 수 |
| word_count_end | INTEGER | | 종료 시 단어 수 |

### document_versions (migration 004)

문서 저장 시 자동 생성되는 스냅샷. `MAX_DOCUMENT_VERSIONS` 환경변수로 문서당 최대 보관 수 설정 (기본값 50).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PK | UUIDv7 |
| document_id | TEXT | NOT NULL, FK → documents(id) CASCADE | 문서 ID |
| version_number | INTEGER | NOT NULL, UNIQUE(document_id, version_number) | 버전 번호 (1부터 순차 증가) |
| content | TEXT | NOT NULL | 마크다운 전체 내용 |
| word_count | INTEGER | NOT NULL DEFAULT 0 | 단어 수 |
| char_count | INTEGER | NOT NULL DEFAULT 0 | 문자 수 |
| created_at | TEXT | NOT NULL DEFAULT now | ISO 8601 UTC |

**Indexes:**
- `idx_document_versions_doc` ON document_versions(document_id, version_number DESC)

### settings

키-값 설정 저장소.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| key | TEXT | PK | 설정 키 |
| value | TEXT | NOT NULL | 설정 값 |

### users (migration 002)

사용자 계정.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PK | UUIDv7 |
| username | TEXT | NOT NULL UNIQUE | 사용자명 (3자 이상) |
| email | TEXT | UNIQUE | 이메일 (선택) |
| password_hash | TEXT | NOT NULL | Argon2id 해시 |
| created_at | TEXT | NOT NULL DEFAULT now | |
| updated_at | TEXT | NOT NULL DEFAULT now | |

### refresh_tokens (migration 002)

JWT refresh token 해시 저장.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PK | UUIDv7 |
| user_id | TEXT | NOT NULL, FK → users(id) CASCADE | 사용자 ID |
| token_hash | TEXT | NOT NULL UNIQUE | SHA-256 해시 |
| expires_at | TEXT | NOT NULL | 만료 시각 (7일) |
| created_at | TEXT | NOT NULL DEFAULT now | |

**Indexes:**
- `idx_refresh_tokens_user_id` ON refresh_tokens(user_id)
- `idx_refresh_tokens_token_hash` ON refresh_tokens(token_hash)
- `idx_documents_user_id` ON documents(user_id) (migration 003)
- `idx_folders_user_id` ON folders(user_id) (migration 003)
- `idx_tags_user_id` ON tags(user_id) (migration 003)
- `idx_tags_user_name` ON tags(user_id, name) UNIQUE (migration 003)

---

## Conventions

- **ID format**: UUIDv7 (시간 기반, 정렬 가능)
- **Timestamps**: ISO 8601 TEXT (`strftime('%Y-%m-%dT%H:%M:%fZ', 'now')`)
- **Boolean**: INTEGER 0/1 (SQLite에 BOOLEAN 타입 없음)
- **Cascade rules**:
  - 폴더 삭제 → 하위 문서 folder_id = NULL
  - 문서 삭제 → writing_sessions, document_tags, document_versions CASCADE 삭제
  - 태그 삭제 → document_tags CASCADE 삭제
  - 사용자 삭제 → refresh_tokens CASCADE 삭제
