-- 매뉴얼 데이터셋 엔트리 (SFT 데이터셋용)
CREATE TABLE IF NOT EXISTS manual_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    preset_id INTEGER NOT NULL REFERENCES presets(id) ON DELETE CASCADE,
    data_json TEXT NOT NULL,  -- 실제 데이터 ({"instruction": "...", "context": "...", "response": "..."})
    review_status TEXT NOT NULL DEFAULT 'pending'
        CHECK(review_status IN ('pending', 'approved', 'skipped', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_manual_entries_preset ON manual_entries(preset_id);
CREATE INDEX IF NOT EXISTS idx_manual_entries_status ON manual_entries(review_status);

-- FTS5 인덱스 (매뉴얼 엔트리 검색용)
CREATE VIRTUAL TABLE IF NOT EXISTS manual_entries_fts USING fts5(
    data,
    content='manual_entries',
    content_rowid='id'
);

-- FTS 트리거
CREATE TRIGGER IF NOT EXISTS manual_entries_fts_insert AFTER INSERT ON manual_entries BEGIN
    INSERT INTO manual_entries_fts(rowid, data) VALUES (NEW.id, NEW.data_json);
END;

CREATE TRIGGER IF NOT EXISTS manual_entries_fts_delete AFTER DELETE ON manual_entries BEGIN
    INSERT INTO manual_entries_fts(manual_entries_fts, rowid, data) VALUES('delete', OLD.id, OLD.data_json);
END;

CREATE TRIGGER IF NOT EXISTS manual_entries_fts_update AFTER UPDATE ON manual_entries BEGIN
    INSERT INTO manual_entries_fts(manual_entries_fts, rowid, data) VALUES('delete', OLD.id, OLD.data_json);
    INSERT INTO manual_entries_fts(rowid, data) VALUES (NEW.id, NEW.data_json);
END;
