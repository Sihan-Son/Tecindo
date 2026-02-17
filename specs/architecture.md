# Tecindo Architecture

## Overview

Tecindo는 개인용 셀프호스팅 글쓰기 앱으로, 단일 Docker 컨테이너에서 백엔드 + 프론트엔드를 함께 서빙하는 모노리스 아키텍처를 채택한다.

```
[Browser] ──HTTPS──▶ [Caddy] ──HTTP──▶ [Tecindo Container]
                                           ├── Axum HTTP Server (:3000)
                                           │   ├── /api/v1/* → REST API
                                           │   └── /* → Static Files (React SPA)
                                           ├── SQLite (metadata, search index)
                                           └── Filesystem (.md files)
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust (2021 edition), Axum 0.8, Tokio, sqlx 0.8 |
| Frontend | React 19, TypeScript 5, Vite 6, TipTap/ProseMirror |
| State Management | Zustand 5 |
| Database | SQLite (via sqlx, FTS5) |
| File Storage | Disk — .md files |
| Auth | JWT (access + refresh), Argon2id |
| PDF Export | pandoc + XeLaTeX |
| Reverse Proxy | Caddy (auto Let's Encrypt) |
| Deployment | Docker (ARM64), Raspberry Pi / NAS |

## Backend Architecture

### Module Structure

```
backend/src/
├── main.rs          # Entry point: server init, routing, static file serving
├── config.rs        # Environment variable loading (Config struct)
├── error.rs         # AppError enum with IntoResponse impl
├── db/
│   ├── mod.rs       # Module re-exports
│   ├── documents.rs # Document & Folder CRUD queries
│   ├── tags.rs      # Tag CRUD & document-tag relationship queries
│   ├── search.rs    # FTS5 full-text search queries
│   ├── sessions.rs  # Writing session queries
│   └── users.rs     # User & refresh token queries
├── models/
│   ├── mod.rs       # Module re-exports
│   ├── document.rs  # Document, Folder, Tag structs & request types
│   └── user.rs      # User, AuthResponse, login/register request types
├── routes/
│   ├── mod.rs       # Module re-exports (glob re-export all handlers)
│   ├── documents.rs # Document CRUD + PDF export handlers, AppState
│   ├── folders.rs   # Folder CRUD handlers
│   ├── tags.rs      # Tag CRUD + document-tag handlers
│   ├── search.rs    # Full-text search handler
│   ├── sessions.rs  # Writing session handlers
│   ├── auth.rs      # Register, login, refresh, logout, me
│   └── health.rs    # Health check endpoint
├── services/
│   ├── mod.rs
│   └── markdown.rs  # File I/O, word/char count, file path generation
└── middleware/
    └── auth.rs      # JWT creation/verification, AuthUser extractor
```

### Request Flow

```
HTTP Request
  → Axum Router (path matching)
    → Middleware (CORS, TraceLayer)
      → Handler Function
        → Extractor (State, Path, Query, Json, AuthUser)
          → DB Query (sqlx) / File I/O (tokio::fs)
            → Response (Json / StatusCode / impl IntoResponse)
```

### Error Handling

`AppError` enum은 모든 에러를 통합 관리한다:

| Variant | HTTP Status | Use Case |
|---------|------------|----------|
| NotFound | 404 | Resource not found |
| BadRequest(String) | 400 | Invalid input |
| Unauthorized(String) | 401 | Auth failure |
| Conflict(String) | 409 | Duplicate resource |
| Internal(String) | 500 | Server error (message hidden from client) |
| Database(sqlx::Error) | 500 | DB error (auto-converted via `#[from]`) |
| Io(io::Error) | 500 | File I/O error (auto-converted via `#[from]`) |

응답 형식: `{ "error": { "code": "not_found", "message": "Resource not found" } }`

### Shared State

`AppState` (in `routes/documents.rs`):

```rust
pub struct AppState {
    pub pool: SqlitePool,        // SQLite connection pool (Arc internally)
    pub documents_path: String,  // Markdown file root directory
    pub jwt_secret: String,      // JWT signing secret
}
```

## Frontend Architecture

### Component Tree

```
App (Router)
├── LoginPage
├── RegisterPage
└── ProtectedRoute
    └── Layout
        ├── Sidebar
        │   ├── Folder Tree (recursive)
        │   ├── Tag Filter (TagBadge)
        │   └── Document List (with tag badges)
        └── Editor
            ├── Title Input
            ├── TagSelector (inline hashtag input)
            ├── TipTap EditorContent
            └── Footer (char stats, export buttons)
```

### State Management (Zustand)

| Store | Purpose |
|-------|---------|
| `documentStore` | Documents CRUD, folders CRUD, current document, content |
| `tagStore` | Tags CRUD, document-tag relationships, findOrCreateTag |
| `uiStore` | UI state (sidebar visibility, etc.) |

### Data Flow

```
User Action → Zustand Action → API Client (fetch) → Backend
                                         ↓
                              Store Update (set) → React Re-render
```

## Storage Architecture

### Dual Storage Strategy

- **SQLite**: Metadata (title, word count, dates, tags, folders, users, sessions)
- **Filesystem**: Document content (.md files)

파일 경로 규칙: `{folder-slug}/{title-slug}-{uuid-prefix-8}.md`

### Full-Text Search (FTS5)

SQLite FTS5 가상 테이블로 제목 + 내용을 인덱싱한다.
토크나이저: `porter unicode61` (영어 어간 추출 + 유니코드 지원)

## Authentication Flow

```
Register/Login → Argon2id verify → JWT access token (short-lived)
                                 → JWT refresh token (7 days, hashed in DB)

API Request → Authorization: Bearer <access_token>
           → 401? → POST /auth/refresh → new token pair
           → still 401? → redirect to /login
```

## PDF Export Flow

```
GET /documents/:id/export/pdf
  → Read .md file from disk
  → Create temp .md with YAML frontmatter (title)
  → pandoc --pdf-engine=xelatex -V CJKmainfont=... -V geometry:margin=2.5cm
  → 60s timeout (tokio::time::timeout)
  → Return PDF bytes with Content-Disposition: attachment
  → Cleanup temp files
```

CJK 폰트는 `TECINDO_CJK_FONT` 환경변수로 설정 (Docker 기본값: `Noto Sans CJK KR`).
