# Tecindo

개인용 셀프호스팅 글쓰기 앱.

## 기술 스택

- **백엔드**: Rust (Axum), SQLite, JWT 인증
- **프론트엔드**: React 19, TypeScript, TipTap, Zustand, Vite
- **배포**: Docker, Caddy (리버스 프록시 + 자동 HTTPS)

## 요구사항

### 로컬 개발

- [Rust](https://rustup.rs/) 1.88+
- [Node.js](https://nodejs.org/) 22+
- SQLite3

### Docker 배포

- Docker, Docker Compose

## 로컬 개발

### 1. 환경변수 설정

```bash
cp .env.example .env
# .env 파일을 열어 JWT_SECRET을 변경
```

### 2. 백엔드 실행

```bash
cd backend
cargo run
```

서버가 `http://localhost:3000`에서 시작됩니다. DB 마이그레이션은 서버 시작 시 자동 실행됩니다.

### 3. 프론트엔드 개발 서버

```bash
cd frontend
npm install
npm run dev
```

Vite 개발 서버가 `http://localhost:5173`에서 시작됩니다.

### 4. 프론트엔드 프로덕션 빌드

```bash
cd frontend
npm run build
```

`frontend/dist/`에 빌드 결과물이 생성됩니다. 백엔드가 이 디렉토리를 감지하면 정적 파일로 서빙합니다.

## Docker 빌드 및 배포

### 빌드 + 실행

```bash
cp .env.example .env
# .env 파일에서 JWT_SECRET을 반드시 변경

docker compose up --build -d
```

`http://localhost:3000`으로 접속 가능합니다.

### Caddy를 통한 HTTPS 배포

`Caddyfile`에서 프로덕션 설정 블록의 주석을 해제하고 도메인을 설정하면 Let's Encrypt 인증서가 자동 발급됩니다.

## 환경변수

| 변수 | 설명 | 기본값 |
|------|------|--------|
| `DATABASE_URL` | SQLite DB 경로 | (필수) |
| `JWT_SECRET` | JWT 서명 비밀키 | (필수) |
| `DOCUMENTS_PATH` | 마크다운 문서 저장 경로 | `data/documents` |
| `UPLOADS_PATH` | 업로드 파일 저장 경로 | `data/uploads` |
| `HOST` | 서버 바인딩 주소 | `0.0.0.0` |
| `PORT` | 서버 포트 | `3000` |
| `MAX_DOCUMENT_VERSIONS` | 문서당 최대 버전 보관 수 | `50` |
| `VERSION_INTERVAL_MINUTES` | 버전 자동 생성 최소 간격 (분) | `60` |
| `RUST_LOG` | 로그 레벨 | `info,tecindo=debug` |

## 프로젝트 구조

```
writing/
├── backend/          # Rust (Axum) API 서버
│   ├── migrations/   # SQLite 마이그레이션
│   └── src/
├── frontend/         # React + TypeScript SPA
│   └── src/
├── specs/            # 프로젝트 상세 스펙 문서
├── data/             # 런타임 데이터 (gitignore)
├── docker-compose.yml
├── Caddyfile
└── .env.example
```
