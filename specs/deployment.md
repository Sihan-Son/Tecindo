# Tecindo Deployment Guide

## Target Environment

- ARM64 Linux (Raspberry Pi 4, NAS)
- Docker container
- Caddy reverse proxy (auto HTTPS)

---

## Quick Start

### 1. Clone & Configure

```bash
git clone <repo-url> && cd writing
cp .env.example .env
# Edit .env: change JWT_SECRET
```

### 2. Build & Run

```bash
docker compose up -d --build
```

서버가 `http://localhost:3000`에서 시작된다.

### 3. Production (with Caddy)

Caddyfile에서 도메인을 설정:

```
your-domain.com {
    reverse_proxy tecindo:3000
    encode gzip
    header {
        X-Frame-Options "SAMEORIGIN"
        X-Content-Type-Options "nosniff"
        Referrer-Policy "strict-origin-when-cross-origin"
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
    }
}
```

---

## Docker Image

### Multi-stage Build (`backend/Dockerfile`)

| Stage | Base Image | Purpose |
|-------|-----------|---------|
| frontend | node:22-slim | React 앱 빌드 (`npm run build`) |
| chef | rust:1.88-slim-bookworm | cargo-chef 설치 |
| planner | chef | 의존성 분석 (`cargo chef prepare`) |
| builder | chef | 의존성 빌드 + 앱 빌드 (캐시 분리) |
| runtime | debian:bookworm-slim | 최소 프로덕션 이미지 |

### Runtime Dependencies

| Package | Purpose |
|---------|---------|
| libsqlite3-0 | SQLite 런타임 |
| ca-certificates | HTTPS 인증서 |
| pandoc | Markdown → LaTeX 변환 |
| texlive-xetex | XeLaTeX PDF 엔진 |
| texlive-latex-recommended | 기본 LaTeX 패키지 |
| texlive-fonts-recommended | 추가 폰트 (Zapf Dingbats 등) |
| texlive-lang-chinese | CJK 언어 지원 |
| lmodern | Latin Modern 폰트 |
| fonts-noto-cjk | Noto Sans CJK 폰트 (한글) |

---

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| DATABASE_URL | Yes | — | SQLite DB 경로 (예: `sqlite:./data/tecindo.db`) |
| JWT_SECRET | Yes | — | JWT 서명 비밀키 (반드시 변경) |
| DOCUMENTS_PATH | No | `data/documents` | .md 파일 저장 경로 |
| UPLOADS_PATH | No | `data/uploads` | 업로드 파일 경로 |
| HOST | No | `0.0.0.0` | 서버 바인딩 주소 |
| PORT | No | `3000` | 서버 포트 |
| RUST_LOG | No | `info` | 로그 레벨 |
| TECINDO_CJK_FONT | No | `Noto Sans CJK KR` | PDF CJK 폰트 이름 |

---

## Data Persistence

Docker volume: `./data:/app/data`

```
data/
├── tecindo.db       # SQLite database
├── documents/       # Markdown files
└── uploads/         # Uploaded files
```

**Backup:** `data/` 디렉토리 전체를 백업하면 된다.

---

## docker-compose.yml

```yaml
services:
  tecindo:
    build:
      context: .
      dockerfile: backend/Dockerfile
    ports:
      - "3000:3000"
    volumes:
      - ./data:/app/data
    env_file:
      - .env
    mem_limit: 512m
    restart: unless-stopped
```

---

## Development

### Backend (Rust)

```bash
cd backend
cargo run     # 개발 서버 (hot-reload 없음)
cargo build --release  # 릴리스 빌드
```

환경변수는 `.env` 파일에서 자동 로딩 (dotenvy).

### Frontend (React)

```bash
cd frontend
npm install
npm run dev   # Vite dev server (HMR)
npm run build # Production build → dist/
```

Vite dev server는 `/api` 요청을 백엔드로 프록시해야 한다.

### Local PDF Export

로컬 macOS에서 PDF 내보내기를 테스트하려면:

```bash
brew install pandoc
brew install --cask mactex  # 또는 basictex
```

`TECINDO_CJK_FONT` 환경변수를 맥 폰트로 설정:

```
TECINDO_CJK_FONT="Apple SD Gothic Neo"
```

---

## Troubleshooting

### PDF export fails with font error

Docker에서 CJK 폰트 관련 에러가 발생하면:
1. `TECINDO_CJK_FONT` 환경변수 확인 (Docker 기본: `Noto Sans CJK KR`)
2. `fonts-noto-cjk` 패키지가 설치되어 있는지 확인
3. 컨테이너 내에서 `fc-list | grep -i noto`로 폰트 확인

### Database locked

SQLite는 동시 쓰기에 제한이 있다. `max_connections(5)`로 설정되어 있으나,
쓰기 충돌이 잦으면 WAL 모드 활성화 고려:

```sql
PRAGMA journal_mode=WAL;
```

### Memory limit

Docker에서 `mem_limit: 512m` 설정. PDF 변환이 메모리를 많이 사용하므로,
대량 문서 변환 시 메모리 부족이 발생할 수 있다.
