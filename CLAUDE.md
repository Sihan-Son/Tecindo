# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# currentDate
Today's date is 2026-02-16.

# Tecindo — 개인용 셀프호스팅 글쓰기 앱

## 기술 스택

| 영역 | 선택 |
|------|------|
| 백엔드 | Axum 0.8 (Rust), Tokio, sqlx (SQLite), yrs, jsonwebtoken, argon2, uuid v7, tower-http, serde, tracing |
| 프론트엔드 | React 19, Vite 6, TypeScript, TipTap (ProseMirror), Yjs, y-websocket, y-indexeddb, Zustand, react-router-dom 7, vite-plugin-pwa |
| DB | SQLite (sqlx) — 메타데이터/검색 인덱스 |
| 파일 저장 | 디스크 .md 파일 |
| 인증 | JWT (access + refresh token), Argon2id |
| HTTPS | Caddy (자동 Let's Encrypt) |
| 배포 | Docker (ARM64), Raspberry Pi 4 |

## 프로젝트 구조

```
writing/
├── docker-compose.yml
├── Caddyfile
├── .env.example
├── backend/
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── migrations/
│   │   └── 001_initial.sql
│   └── src/
│       ├── main.rs          # Axum 서버 + 라우팅 + 정적파일 서빙
│       ├── config.rs         # 환경변수 로딩
│       ├── error.rs          # 에러 타입
│       ├── db/
│       │   ├── mod.rs
│       │   └── documents.rs  # 문서 CRUD 쿼리
│       ├── models/
│       │   ├── mod.rs
│       │   └── document.rs   # Document, Folder 구조체
│       ├── routes/
│       │   ├── mod.rs
│       │   ├── documents.rs  # 문서 CRUD 핸들러
│       │   ├── folders.rs    # 폴더 핸들러
│       │   └── health.rs     # 헬스체크
│       └── services/
│           ├── mod.rs
│           └── markdown.rs   # .md 파일 I/O
├── frontend/
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── components/
│       │   ├── Editor/
│       │   │   └── Editor.tsx       # TipTap 에디터
│       │   ├── Sidebar/
│       │   │   └── Sidebar.tsx      # 문서 목록
│       │   └── Layout/
│       │       └── Layout.tsx       # 반응형 레이아웃
│       ├── hooks/
│       │   └── useDocuments.ts
│       ├── stores/
│       │   ├── documentStore.ts     # Zustand 문서 상태
│       │   └── uiStore.ts          # UI 상태
│       ├── api/
│       │   └── client.ts           # REST API 클라이언트
│       ├── lib/
│       │   └── types.ts            # 공유 타입
│       └── styles/
│           └── editor.css          # 타이포그래피
└── data/                           # 볼륨 마운트 (gitignore)
    ├── documents/
    ├── uploads/
    └── tecindo.db
```

## DB 스키마 (SQLite)

```sql
-- folders
CREATE TABLE folders (
    id TEXT PRIMARY KEY,  -- UUIDv7
    parent_id TEXT REFERENCES folders(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- documents
CREATE TABLE documents (
    id TEXT PRIMARY KEY,  -- UUIDv7
    folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL,
    title TEXT NOT NULL DEFAULT 'Untitled',
    slug TEXT NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    word_count INTEGER NOT NULL DEFAULT 0,
    char_count INTEGER NOT NULL DEFAULT 0,
    excerpt TEXT,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- tags
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT
);

CREATE TABLE document_tags (
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, tag_id)
);

-- FTS5
CREATE VIRTUAL TABLE documents_fts USING fts5(
    title, content,
    content='documents',
    content_rowid='rowid',
    tokenize='porter unicode61'
);

-- writing sessions
CREATE TABLE writing_sessions (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    device_name TEXT,
    started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ended_at TEXT,
    word_count_start INTEGER NOT NULL DEFAULT 0,
    word_count_end INTEGER
);

-- settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

## API 설계 (REST /api/v1/)

### Documents
- `GET /documents` → `{ documents: Document[] }`
- `POST /documents` → `{ title?, folder_id? }` → `Document`
- `GET /documents/:id` → `Document`
- `PATCH /documents/:id` → `{ title?, folder_id?, is_pinned?, is_archived? }` → `Document`
- `DELETE /documents/:id` → `204`
- `GET /documents/:id/content` → `{ content: string }` (마크다운)
- `PUT /documents/:id/content` → `{ content: string }` → `204`

### Folders
- `GET /folders` → `{ folders: Folder[] }`
- `POST /folders` → `{ name, parent_id? }` → `Folder`
- `PATCH /folders/:id` → `{ name?, parent_id?, sort_order? }` → `Folder`
- `DELETE /folders/:id` → `204`

### Health
- `GET /health` → `{ status: "ok" }`

## 코딩 규칙

- Rust: 2021 edition, async/await, anyhow for errors in main, thiserror for library errors
- Frontend: functional components only, named exports, absolute imports via `@/`
- 모든 API 응답은 JSON, snake_case 필드명
- 에러 응답: `{ error: { code: string, message: string } }`
