//! 데이터베이스 스키마 정의
//!
//! 참고: 실제 스키마는 migrations/0001_initial.sql에 정의되어 있습니다.
//! 이 파일은 레거시 호환성을 위해 유지됩니다.

#[allow(dead_code)]
/// 스키마 생성 SQL (deprecated - migrations 폴더 사용)
pub const SCHEMA_SQL: &str = r#"
-- 마이그레이션 추적
CREATE TABLE IF NOT EXISTS _migrations (
    id INTEGER PRIMARY KEY,
    version TEXT NOT NULL UNIQUE,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 소스 (폴더/파일/수동입력)
CREATE TABLE IF NOT EXISTS sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT NOT NULL CHECK(type IN ('folder', 'file', 'manual')),
    path_or_key TEXT NOT NULL,
    display_name TEXT,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    meta_json TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_path ON sources(path_or_key);

-- 문서
CREATE TABLE IF NOT EXISTS documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    original_path TEXT,
    checksum TEXT NOT NULL,
    encoding TEXT NOT NULL DEFAULT 'UTF-8',
    status TEXT NOT NULL DEFAULT 'unprocessed'
        CHECK(status IN ('unprocessed', 'chunked', 'in_review', 'approved', 'exported', 'needs_attention')),
    byte_size INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_documents_source ON documents(source_id);
CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(status);

-- 문서 스냅샷 (원본 보존)
CREATE TABLE IF NOT EXISTS document_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    snapshot_path TEXT NOT NULL,
    snapshot_checksum TEXT NOT NULL,
    normalized_checksum TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_snapshots_doc ON document_snapshots(document_id);

-- 태그
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    UNIQUE(key, value)
);

-- 문서-태그 연결
CREATE TABLE IF NOT EXISTS document_tags (
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, tag_id)
);

-- 청크 버전 (재현성 확보)
CREATE TABLE IF NOT EXISTS chunk_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    params_json TEXT NOT NULL,
    tokenizer_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
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
    char_count INTEGER NOT NULL DEFAULT 0,
    token_est INTEGER NOT NULL DEFAULT 0,
    overlap_prev INTEGER NOT NULL DEFAULT 0,
    overlap_next INTEGER NOT NULL DEFAULT 0,
    review_status TEXT NOT NULL DEFAULT 'pending'
        CHECK(review_status IN ('pending', 'approved', 'skipped', 'rejected')),
    is_hard_cut INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_chunks_doc_version ON chunks(document_id, chunk_version_id);
CREATE INDEX IF NOT EXISTS idx_chunks_review ON chunks(review_status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_chunks_order ON chunks(document_id, chunk_version_id, chunk_index);

-- 청크 편집 (오버레이)
CREATE TABLE IF NOT EXISTS chunk_edits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id INTEGER NOT NULL REFERENCES chunks(id) ON DELETE CASCADE,
    edited_text TEXT NOT NULL,
    edited_at TEXT NOT NULL DEFAULT (datetime('now')),
    edit_meta_json TEXT
);
CREATE INDEX IF NOT EXISTS idx_edits_chunk ON chunk_edits(chunk_id);

-- Export 프리셋
CREATE TABLE IF NOT EXISTS presets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    schema_version TEXT NOT NULL,
    mapping_json TEXT NOT NULL,
    validators_json TEXT,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Export 실행 기록
CREATE TABLE IF NOT EXISTS export_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    preset_id INTEGER REFERENCES presets(id),
    preset_name TEXT NOT NULL,
    params_json TEXT NOT NULL,
    filters_json TEXT NOT NULL,
    stats_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
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
CREATE INDEX IF NOT EXISTS idx_export_files_run ON export_files(export_run_id);

-- 백그라운드 잡
CREATE TABLE IF NOT EXISTS jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT NOT NULL CHECK(type IN ('scan', 'chunk', 'export')),
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK(status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    progress REAL NOT NULL DEFAULT 0.0,
    payload_json TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);

-- FTS5 인덱스 (청크 검색용)
CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
    text,
    content='chunks',
    content_rowid='id'
);

-- FTS 트리거
CREATE TRIGGER IF NOT EXISTS chunks_fts_insert AFTER INSERT ON chunks BEGIN
    INSERT INTO chunks_fts(rowid, text) VALUES (NEW.id, COALESCE(NEW.text_cached, ''));
END;

CREATE TRIGGER IF NOT EXISTS chunks_fts_delete AFTER DELETE ON chunks BEGIN
    INSERT INTO chunks_fts(chunks_fts, rowid, text) VALUES('delete', OLD.id, COALESCE(OLD.text_cached, ''));
END;

CREATE TRIGGER IF NOT EXISTS chunks_fts_update AFTER UPDATE ON chunks BEGIN
    INSERT INTO chunks_fts(chunks_fts, rowid, text) VALUES('delete', OLD.id, COALESCE(OLD.text_cached, ''));
    INSERT INTO chunks_fts(rowid, text) VALUES (NEW.id, COALESCE(NEW.text_cached, ''));
END;
"#;
