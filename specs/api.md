# Tecindo REST API

Base URL: `/api/v1`

모든 응답은 JSON, snake_case 필드명을 사용한다.
에러 응답: `{ "error": { "code": string, "message": string } }`

### 인증 요구사항

Authentication, Health를 제외한 모든 엔드포인트는 JWT 인증이 필수다.

**Headers:** `Authorization: Bearer <access_token>`

인증 없이 접근 시 `401 Unauthorized` 응답. 각 사용자는 자신의 데이터만 접근 가능 (Multi-tenant 격리).

---

## Authentication

### POST /auth/register

회원가입.

**Request:**
```json
{ "username": "sihan", "password": "mypassword", "email": "sihan@example.com" }
```
- `username`: 필수, 3자 이상
- `password`: 필수, 8자 이상
- `email`: 선택, `@` 포함 필수

**Response:** `200`
```json
{
  "user": { "id": "...", "username": "sihan", "email": "sihan@example.com" },
  "access_token": "eyJ...",
  "refresh_token": "eyJ..."
}
```

**Errors:** `400` (validation), `409` (username/email conflict)

### POST /auth/login

로그인.

**Request:**
```json
{ "username": "sihan", "password": "mypassword" }
```

**Response:** AuthResponse (register와 동일)

**Errors:** `401` (invalid credentials)

### POST /auth/refresh

Access token 갱신.

**Request:**
```json
{ "refresh_token": "eyJ..." }
```

**Response:** AuthResponse (new token pair)

**Errors:** `401` (invalid/expired token)

### POST /auth/logout

로그아웃 (해당 사용자의 모든 refresh token 삭제).

**Headers:** `Authorization: Bearer <access_token>`

**Response:** `200` `{ "message": "Logged out successfully" }`

### GET /auth/me

현재 인증된 사용자 정보.

**Headers:** `Authorization: Bearer <access_token>`

**Response:** `200`
```json
{ "id": "...", "username": "sihan", "email": "sihan@example.com" }
```

---

## Documents

> 모든 Documents 엔드포인트는 `Authorization: Bearer <token>` 헤더 필수. 인증된 사용자의 문서만 반환/조작.

### GET /documents

문서 목록 조회. 고정 문서 우선, 수정일 내림차순 정렬. **인증된 사용자의 문서만 반환.**

**Query Parameters:**
- `tag_id` (선택): 특정 태그가 붙은 문서만 필터링

**Response:** `200`
```json
{
  "documents": [
    {
      "id": "019...",
      "folder_id": "019..." | null,
      "title": "나의 첫 글",
      "slug": "나의-첫-글",
      "file_path": "나의-첫-글-019abcde.md",
      "word_count": 150,
      "char_count": 500,
      "excerpt": "이 글은...",
      "is_pinned": 0,
      "is_archived": 0,
      "created_at": "2026-01-15T09:30:45.123Z",
      "updated_at": "2026-02-16T14:20:00.000Z"
    }
  ]
}
```

### POST /documents

새 문서 생성. 디스크에 빈 .md 파일을 생성하고 DB에 메타데이터를 저장한다.

**Request:**
```json
{ "title": "나의 첫 글", "folder_id": "019..." }
```
- `title`: 선택 (기본값: "Untitled", 같은 폴더 내 중복 시 "Untitled_2", "Untitled_3"...)
- `folder_id`: 선택

**Response:** `200` Document

### GET /documents/:id

단일 문서 조회.

**Response:** `200` Document

**Errors:** `404`

### PATCH /documents/:id

문서 메타데이터 수정 (부분 업데이트).

**Request:**
```json
{
  "title": "새 제목",
  "folder_id": "019..." | null,
  "is_pinned": true,
  "is_archived": false
}
```
- 모든 필드 선택적. 포함된 필드만 업데이트.
- `folder_id: null` → 루트로 이동
- `folder_id` 필드 누락 → 변경 안 함

**Response:** `200` Document

**Errors:** `404`

### DELETE /documents/:id

문서 삭제. DB 레코드와 디스크 .md 파일 모두 삭제.

**Response:** `204 No Content`

**Errors:** `404`

### GET /documents/:id/content

문서 마크다운 내용 조회.

**Response:** `200`
```json
{ "content": "# 제목\n\n본문..." }
```

**Errors:** `404`

### PUT /documents/:id/content

문서 내용 수정. 디스크 파일 덮어쓰기 + DB 통계 업데이트 + FTS 인덱스 갱신.

**Request:**
```json
{ "content": "# 새 내용\n\n..." }
```

**Response:** `204 No Content`

**Errors:** `404`

### GET /documents/:id/export/pdf

문서를 PDF로 내보내기. pandoc + XeLaTeX로 변환.

**Response:** `200`
- Content-Type: `application/pdf`
- Content-Disposition: `attachment; filename="slug.pdf"`

**Errors:** `404`, `500` (변환 실패 또는 60초 타임아웃)

---

## Folders

> 모든 Folders 엔드포인트는 `Authorization: Bearer <token>` 헤더 필수. 인증된 사용자의 폴더만 반환/조작.

### GET /folders

전체 폴더 목록 (sort_order → name 정렬). **인증된 사용자의 폴더만 반환.**

**Response:** `200`
```json
{
  "folders": [
    {
      "id": "019...",
      "parent_id": null,
      "name": "일기",
      "slug": "일기",
      "sort_order": 0,
      "created_at": "...",
      "updated_at": "..."
    }
  ]
}
```

### POST /folders

새 폴더 생성.

**Request:**
```json
{ "name": "일기", "parent_id": "019..." }
```
- `name`: 필수
- `parent_id`: 선택 (없으면 최상위)

**Response:** `200` Folder

### PATCH /folders/:id

폴더 수정.

**Request:**
```json
{ "name": "새 이름", "parent_id": "019...", "sort_order": 1 }
```

**Response:** `200` Folder

**Errors:** `404`

### DELETE /folders/:id

폴더 삭제. 내부 문서의 folder_id는 NULL로 설정 (ON DELETE SET NULL).

**Response:** `204 No Content`

**Errors:** `404`

---

## Tags

> 모든 Tags 엔드포인트는 `Authorization: Bearer <token>` 헤더 필수. 인증된 사용자의 태그만 반환/조작.

### GET /tags

전체 태그 목록. **인증된 사용자의 태그만 반환.**

**Response:** `200`
```json
{
  "tags": [
    { "id": "019...", "name": "프로그래밍", "color": "#3b82f6" }
  ]
}
```

### POST /tags

새 태그 생성.

**Request:**
```json
{ "name": "프로그래밍", "color": "#3b82f6" }
```
- `name`: 필수 (UNIQUE)
- `color`: 선택

**Response:** `200` Tag

### PATCH /tags/:id

태그 수정.

**Request:**
```json
{ "name": "새 이름", "color": "#ef4444" }
```

**Response:** `200` Tag

### DELETE /tags/:id

태그 삭제. document_tags 관계도 CASCADE 삭제.

**Response:** `204 No Content`

### GET /documents/:id/tags

특정 문서의 태그 목록.

**Response:** `200` `{ "tags": [...] }`

### POST /documents/:id/tags

문서에 태그 연결.

**Request:**
```json
{ "tag_id": "019..." }
```

**Response:** `201 Created`

### DELETE /documents/:id/tags/:tag_id

문서에서 태그 해제.

**Response:** `204 No Content`

---

## Document Versions

> 모든 Versions 엔드포인트는 `Authorization: Bearer <token>` 헤더 필수. 문서 소유자만 접근 가능.

### GET /documents/:id/versions

문서의 버전 히스토리 목록 (최신 순). content 필드 미포함.
버전은 `VERSION_INTERVAL_MINUTES` 간격으로 자동 생성되며, 5분 비활동 시 프론트엔드가 최종 스냅샷을 요청한다.

**Response:** `200`
```json
{
  "versions": [
    {
      "id": "019...",
      "document_id": "019...",
      "version_number": 5,
      "word_count": 200,
      "char_count": 600,
      "created_at": "2026-02-17T10:30:00.000Z"
    }
  ]
}
```

**Errors:** `404` (문서 없음 또는 권한 없음)

### GET /versions/:id

특정 버전의 전체 내용 조회.

**Response:** `200`
```json
{
  "id": "019...",
  "document_id": "019...",
  "version_number": 5,
  "content": "# 제목\n\n본문...",
  "word_count": 200,
  "char_count": 600,
  "created_at": "2026-02-17T10:30:00.000Z"
}
```

**Errors:** `404`

### POST /documents/:id/versions

비활동 시 프론트엔드가 호출하는 버전 스냅샷 생성. 마지막 버전 이후 변경이 있을 때만 생성.

**Response:** `201 Created` (생성됨) 또는 `204 No Content` (변경 없어 스킵)

**Errors:** `404`

---

## Search

> Search 엔드포인트는 `Authorization: Bearer <token>` 헤더 필수.

### GET /search?q=keyword

FTS5 전문검색. 관련도순 정렬, 최대 50건. **인증된 사용자의 문서만 검색.**

**Query Parameters:**
- `q`: 필수, 검색어 (FTS5 문법 지원: `hello*` 접두사 검색, 공백은 AND)

**Response:** `200`
```json
{
  "documents": [
    { "id": "...", "title": "...", "excerpt": "...", "rank": -1.5 }
  ]
}
```

**Errors:** `400` (빈 검색어)

---

## Writing Sessions

> Writing Sessions 엔드포인트는 `Authorization: Bearer <token>` 헤더 필수. 문서 소유자만 접근 가능.

### GET /documents/:id/sessions

문서의 글쓰기 세션 이력 (최신 순).

**Response:** `200`
```json
{
  "sessions": [
    {
      "id": "019...",
      "document_id": "019...",
      "device_name": "MacBook Pro",
      "started_at": "...",
      "ended_at": "...",
      "word_count_start": 100,
      "word_count_end": 250
    }
  ]
}
```

### POST /documents/:id/sessions

글쓰기 세션 시작.

**Request:**
```json
{ "device_name": "MacBook Pro", "word_count_start": 100 }
```

**Response:** `200` WritingSession

### PATCH /sessions/:id

글쓰기 세션 종료.

**Request:**
```json
{ "word_count_end": 250 }
```

**Response:** `200` WritingSession

---

## Health

### GET /health

서버 상태 확인.

**Response:** `200` `{ "status": "ok" }`
