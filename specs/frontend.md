# Tecindo Frontend

## Stack

- React 19, TypeScript 5, Vite 6
- TipTap (ProseMirror) + tiptap-markdown
- Zustand 5 (state management)
- react-router-dom 7
- vite-plugin-pwa

---

## Routing

```
/login       → LoginPage
/register    → RegisterPage
/            → ProtectedRoute → Layout (no document selected)
/doc/:id     → ProtectedRoute → Layout (document loaded)
```

`ProtectedRoute`는 access token 유무로 인증 상태를 확인한다.

---

## Component Structure

### Layout (`components/Layout/Layout.tsx`)

사이드바 + 에디터의 2컬럼 레이아웃. 반응형.

### Sidebar (`components/Sidebar/Sidebar.tsx`)

| Section | Description |
|---------|-------------|
| Header | "Tecindo" 타이틀, 새 폴더 / 새 문서 버튼 |
| Search | 문서 제목 검색 (클라이언트 필터) |
| Folders | 트리 구조 폴더 목록 (재귀 렌더링), 접기/펼치기, 더블클릭 이름 편집 |
| Tag Filter | 태그 배지 목록, 클릭으로 필터링 토글 |
| Document List | 문서 아이템 (제목, excerpt, 태그 배지, 수정일, 단어 수), 컨텍스트 메뉴 (이동, 삭제) |

**States:**
- `currentFolderId` — 현재 선택된 폴더 (null = 전체)
- `selectedTagId` — 현재 선택된 태그 필터 (null = 전체)
- `searchQuery` — 검색어 (클라이언트 필터)
- `expandedFolders` — 펼쳐진 폴더 ID Set
- `menuDocId` — 컨텍스트 메뉴가 열린 문서 ID

**Korean IME 처리:**
- `e.nativeEvent.isComposing` 체크로 한글 입력 중 Enter 이벤트 무시
- `creatingFolderRef` ref guard로 onBlur + onKeyDown 중복 실행 방지

### Editor (`components/Editor/Editor.tsx`)

| Section | Description |
|---------|-------------|
| Title | `<input>` — 타이틀 수정, onBlur 시 서버 저장 |
| TagSelector | 인라인 해시태그 입력 |
| EditorContent | TipTap WYSIWYG 에디터 (마크다운 저장) |
| Footer | 작성일, 글자 통계 (한/영/기타/단어), .md/.pdf 내보내기 버튼 |

**Auto-save:** `onUpdate`에서 1초 debounce 후 서버 저장 (`saveTimeoutRef`).

**Content Loading:** 문서 전환 시 `fetchDocumentContent`로 서버에서 내용 로드.
`skipUpdateRef`로 로딩 중 onUpdate 트리거 방지.

**Export:**
- `.md` — 프론트엔드에서 Blob 생성 후 다운로드
- `.pdf` — `GET /api/v1/documents/:id/export/pdf` 호출

**Character Stats:**
- 총 글자 수 (공백 제외)
- 한글 (`[\u3131-\u318E\uAC00-\uD7A3]`)
- 영문 (`[a-zA-Z]`)
- 기타 (총 - 한 - 영)
- 단어 수 (공백 분리)

### Tags Components

#### TagSelector (`components/Tags/TagSelector.tsx`)

인스타그램 해시태그 스타일 인라인 태그 입력 컴포넌트.

- `#` 접두사 + 태그 배지들 + 입력란
- Enter/쉼표로 태그 추가 (없으면 자동 생성)
- Backspace로 마지막 태그 제거
- 자동완성 드롭다운 (기존 태그 중 매칭)
- `findOrCreateTag` — 이름으로 검색, 없으면 해시 기반 자동 색상 할당하여 생성

#### TagBadge (`components/Tags/TagBadge.tsx`)

태그 배지 렌더링. 색상 원 + 이름. 선택적 `onRemove` 콜백.

#### TagManager (`components/Tags/TagManager.tsx`)

태그 생성/삭제 관리 패널 (현재 UI에 직접 마운트되지 않음).

### Auth Components

#### LoginPage (`components/Auth/LoginPage.tsx`)

사용자명 + 비밀번호 로그인 폼. 토큰을 localStorage에 저장.

#### RegisterPage (`components/Auth/RegisterPage.tsx`)

사용자명 + 이메일(선택) + 비밀번호 회원가입 폼.

#### ProtectedRoute (`components/Auth/ProtectedRoute.tsx`)

access token 존재 여부 확인. 없으면 /login으로 리다이렉트.

---

## State Stores (Zustand)

### documentStore (`stores/documentStore.ts`)

| State | Type | Description |
|-------|------|-------------|
| documents | Document[] | 문서 목록 |
| currentDocument | Document \| null | 현재 선택된 문서 |
| currentContent | string | 현재 문서 내용 |
| folders | Folder[] | 폴더 목록 |
| currentFolderId | string \| null | 현재 선택된 폴더 |
| loading | boolean | 로딩 상태 |
| error | string \| null | 에러 메시지 |

| Action | Description |
|--------|-------------|
| loadDocuments(tagId?) | 문서 목록 로드 (태그 필터 가능) |
| loadDocument(id) | 단일 문서 로드 → currentDocument 설정 |
| createDocument(data?) | 새 문서 생성 → 목록에 추가 |
| updateDocument(id, data) | 문서 메타 수정 |
| deleteDocument(id) | 문서 삭제 → 목록에서 제거 |
| saveContent(id, content) | 문서 내용 저장 (auto-save) |
| loadFolders() | 폴더 목록 로드 |
| createFolder(data) | 폴더 생성 |
| updateFolder(id, data) | 폴더 수정 |
| deleteFolder(id) | 폴더 삭제 |

### tagStore (`stores/tagStore.ts`)

| State | Type | Description |
|-------|------|-------------|
| tags | Tag[] | 전체 태그 목록 |
| documentTags | Record<string, Tag[]> | 문서별 태그 맵 |

| Action | Description |
|--------|-------------|
| loadTags() | 전체 태그 로드 |
| createTag(data) | 태그 생성 |
| deleteTag(id) | 태그 삭제 |
| loadDocumentTags(docId) | 문서의 태그 로드 |
| addTagToDocument(docId, tagId) | 문서에 태그 추가 |
| removeTagFromDocument(docId, tagId) | 문서에서 태그 제거 |
| findOrCreateTag(name) | 이름으로 태그 검색, 없으면 자동 생성 (해시 기반 색상) |

**Auto-color assignment:**
```
TAG_COLORS = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#3b82f6', '#8b5cf6', '#ec4899', '#6b7280']
color = TAG_COLORS[hash(name) % 8]
```

---

## API Client (`api/client.ts`, `api/tags.ts`)

### client.ts

| Function | HTTP | Path |
|----------|------|------|
| fetchDocuments(tagId?) | GET | /documents(?tag_id=) |
| fetchDocument(id) | GET | /documents/:id |
| createDocument(data?) | POST | /documents |
| updateDocument(id, data) | PATCH | /documents/:id |
| deleteDocument(id) | DELETE | /documents/:id |
| fetchDocumentContent(id) | GET | /documents/:id/content |
| updateDocumentContent(id, content) | PUT | /documents/:id/content |
| fetchFolders() | GET | /folders |
| createFolder(data) | POST | /folders |
| updateFolder(id, data) | PATCH | /folders/:id |
| deleteFolder(id) | DELETE | /folders/:id |

`authFetch` 래퍼: Authorization 헤더 자동 추가, 401 시 refresh token으로 재시도.

### tags.ts

모든 함수가 `authFetch`를 사용.

| Function | HTTP | Path |
|----------|------|------|
| fetchTags() | GET | /tags |
| createTag(data) | POST | /tags |
| updateTag(id, data) | PATCH | /tags/:id |
| deleteTag(id) | DELETE | /tags/:id |
| fetchDocumentTags(docId) | GET | /documents/:id/tags |
| addTagToDocument(docId, tagId) | POST | /documents/:id/tags |
| removeTagFromDocument(docId, tagId) | DELETE | /documents/:id/tags/:tagId |

---

## Types (`lib/types.ts`)

```typescript
interface Document {
  id: string;
  folder_id: string | null;
  title: string;
  slug: string;
  file_path: string;
  word_count: number;
  char_count: number;
  excerpt: string | null;
  is_pinned: boolean;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
}

interface Folder {
  id: string;
  parent_id: string | null;
  name: string;
  slug: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface Tag {
  id: string;
  name: string;
  color: string | null;
}

interface SearchResult {
  id: string;
  title: string;
  excerpt: string;
  rank: number;
}
```
