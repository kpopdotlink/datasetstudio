-- Dataset Studio 초기 스키마
-- Version: 0001

-- 소스 (폴더, 파일, 직접입력)
CREATE TABLE IF NOT EXISTS sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT NOT NULL CHECK (type IN ('folder', 'file', 'manual')),
    path_or_key TEXT NOT NULL UNIQUE,
    display_name TEXT,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    meta_json TEXT
);

-- 문서
CREATE TABLE IF NOT EXISTS documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    original_path TEXT,
    checksum TEXT NOT NULL,
    encoding TEXT DEFAULT 'UTF-8',
    status TEXT DEFAULT 'unprocessed' CHECK (status IN ('unprocessed', 'processing', 'chunked', 'error')),
    byte_size INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_documents_source_id ON documents(source_id);
CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_documents_checksum ON documents(source_id, checksum);

-- 청크 버전 (파라미터 세트)
CREATE TABLE IF NOT EXISTS chunk_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    params_json TEXT NOT NULL,
    tokenizer_id TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 청크
CREATE TABLE IF NOT EXISTS chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_version_id INTEGER NOT NULL REFERENCES chunk_versions(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    start_offset INTEGER NOT NULL,
    end_offset INTEGER NOT NULL,
    text_cached TEXT,
    char_count INTEGER NOT NULL,
    token_est INTEGER NOT NULL,
    overlap_prev INTEGER DEFAULT 0,
    overlap_next INTEGER DEFAULT 0,
    review_status TEXT DEFAULT 'pending' CHECK (review_status IN ('pending', 'approved', 'skipped', 'rejected')),
    is_hard_cut INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id);
CREATE INDEX IF NOT EXISTS idx_chunks_review_status ON chunks(review_status);
CREATE INDEX IF NOT EXISTS idx_chunks_version ON chunks(chunk_version_id);

-- 청크 편집 (오버레이)
CREATE TABLE IF NOT EXISTS chunk_edits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id INTEGER NOT NULL REFERENCES chunks(id) ON DELETE CASCADE,
    edited_text TEXT NOT NULL,
    edited_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    edit_meta_json TEXT
);

CREATE INDEX IF NOT EXISTS idx_chunk_edits_chunk_id ON chunk_edits(chunk_id);

-- 태그
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(key, value)
);

-- 청크-태그 연결
CREATE TABLE IF NOT EXISTS chunk_tags (
    chunk_id INTEGER NOT NULL REFERENCES chunks(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (chunk_id, tag_id)
);

-- 프리셋
CREATE TABLE IF NOT EXISTS presets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    schema_version TEXT NOT NULL DEFAULT '1.0',
    mapping_json TEXT NOT NULL,
    validators_json TEXT,
    is_builtin INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Export 실행 기록
CREATE TABLE IF NOT EXISTS export_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    preset_id INTEGER REFERENCES presets(id) ON DELETE SET NULL,
    preset_name TEXT NOT NULL,
    params_json TEXT NOT NULL,
    filters_json TEXT NOT NULL,
    stats_json TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Export 파일
CREATE TABLE IF NOT EXISTS export_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    export_run_id INTEGER NOT NULL REFERENCES export_runs(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    line_count INTEGER NOT NULL,
    sha256 TEXT
);

-- 잡 (백그라운드 작업)
CREATE TABLE IF NOT EXISTS jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_type TEXT NOT NULL CHECK (job_type IN ('scan', 'chunk', 'export')),
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    progress REAL DEFAULT 0.0,
    payload_json TEXT,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- FTS 인덱스 (청크 검색용)
CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
    text_cached,
    content='chunks',
    content_rowid='id'
);

-- FTS 트리거
CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON chunks BEGIN
    INSERT INTO chunks_fts(rowid, text_cached) VALUES (new.id, new.text_cached);
END;

CREATE TRIGGER IF NOT EXISTS chunks_ad AFTER DELETE ON chunks BEGIN
    INSERT INTO chunks_fts(chunks_fts, rowid, text_cached) VALUES ('delete', old.id, old.text_cached);
END;

CREATE TRIGGER IF NOT EXISTS chunks_au AFTER UPDATE ON chunks BEGIN
    INSERT INTO chunks_fts(chunks_fts, rowid, text_cached) VALUES ('delete', old.id, old.text_cached);
    INSERT INTO chunks_fts(rowid, text_cached) VALUES (new.id, new.text_cached);
END;
