-- documents, folders, tags 테이블에 user_id 컬럼 추가
-- 사용자별 데이터 격리(Multi-tenant)를 위한 마이그레이션

ALTER TABLE documents ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE folders ADD COLUMN user_id TEXT REFERENCES users(id);
ALTER TABLE tags ADD COLUMN user_id TEXT REFERENCES users(id);

-- tags의 name UNIQUE 제약조건을 user_id별로 변경
-- SQLite는 ALTER TABLE로 제약조건 변경이 불가하므로 인덱스로 대체
CREATE UNIQUE INDEX idx_tags_user_name ON tags(user_id, name);

CREATE INDEX idx_documents_user_id ON documents(user_id);
CREATE INDEX idx_folders_user_id ON folders(user_id);
CREATE INDEX idx_tags_user_id ON tags(user_id);
