//! 데이터베이스 Repository

use crate::commands::review::{ChunkDetail, ReviewFilters, ReviewQueue, SearchResult};
use crate::commands::source::DocumentFilters;
use crate::commands::stats::ProjectStats;
use crate::db::models::*;
use crate::db::DbPool;
use crate::error::Result;
use std::path::Path;

/// 문서 목록 조회
pub fn list_documents(pool: &DbPool, filters: &DocumentFilters) -> Result<Vec<Document>> {
    let conn = pool.get()?;

    let mut sql = String::from(
        "SELECT id, source_id, display_name, original_path, checksum, encoding, status, byte_size, created_at, updated_at
         FROM documents WHERE 1=1",
    );

    if let Some(ref status) = filters.status {
        sql.push_str(&format!(" AND status = '{}'", status));
    }
    if let Some(source_id) = filters.source_id {
        sql.push_str(&format!(" AND source_id = {}", source_id));
    }

    sql.push_str(" ORDER BY created_at DESC");

    if let Some(limit) = filters.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    if let Some(offset) = filters.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }

    let mut stmt = conn.prepare(&sql)?;
    let documents = stmt
        .query_map([], |row| {
            Ok(Document {
                id: row.get(0)?,
                source_id: row.get(1)?,
                display_name: row.get(2)?,
                original_path: row.get(3)?,
                checksum: row.get(4)?,
                encoding: row.get(5)?,
                status: row.get(6)?,
                byte_size: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(documents)
}

/// 문서 삭제
pub fn delete_document(pool: &DbPool, doc_id: i64) -> Result<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM documents WHERE id = ?1", [doc_id])?;
    Ok(())
}

/// 리뷰 큐 조회
pub fn get_review_queue(pool: &DbPool, filters: &ReviewFilters) -> Result<ReviewQueue> {
    let conn = pool.get()?;

    // 통계 조회
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))?;
    let pending: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chunks WHERE review_status = 'pending'",
        [],
        |row| row.get(0),
    )?;
    let approved: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chunks WHERE review_status = 'approved'",
        [],
        |row| row.get(0),
    )?;
    let skipped: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chunks WHERE review_status = 'skipped'",
        [],
        |row| row.get(0),
    )?;

    // 청크 목록 조회
    let mut sql = String::from(
        "SELECT id, document_id, chunk_version_id, chunk_index, start_offset, end_offset,
                text_cached, char_count, token_est, overlap_prev, overlap_next,
                review_status, is_hard_cut, created_at, updated_at
         FROM chunks WHERE 1=1",
    );

    if let Some(ref status) = filters.status {
        sql.push_str(&format!(" AND review_status = '{}'", status));
    }
    if let Some(document_id) = filters.document_id {
        sql.push_str(&format!(" AND document_id = {}", document_id));
    }
    if let Some(source_id) = filters.source_id {
        sql.push_str(&format!(
            " AND document_id IN (SELECT id FROM documents WHERE source_id = {})",
            source_id
        ));
    }

    // 파일명 순서로 정렬 (소스별, 파일명순, 청크 인덱스순)
    sql.push_str(" ORDER BY (SELECT d.source_id FROM documents d WHERE d.id = chunks.document_id), (SELECT d.display_name FROM documents d WHERE d.id = chunks.document_id), chunk_index");

    if let Some(limit) = filters.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }
    if let Some(offset) = filters.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }

    let mut stmt = conn.prepare(&sql)?;
    let chunks = stmt
        .query_map([], |row| {
            Ok(Chunk {
                id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_version_id: row.get(2)?,
                chunk_index: row.get(3)?,
                start_offset: row.get(4)?,
                end_offset: row.get(5)?,
                text_cached: row.get(6)?,
                char_count: row.get(7)?,
                token_est: row.get(8)?,
                overlap_prev: row.get(9)?,
                overlap_next: row.get(10)?,
                review_status: row.get(11)?,
                is_hard_cut: row.get::<_, i64>(12)? != 0,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(ReviewQueue {
        chunks,
        total,
        pending,
        approved,
        skipped,
    })
}

/// 청크 상세 조회
pub fn get_chunk_detail(pool: &DbPool, _project_path: &Path, chunk_id: i64) -> Result<ChunkDetail> {
    let conn = pool.get()?;

    // 청크 조회
    let chunk: Chunk = conn.query_row(
        "SELECT id, document_id, chunk_version_id, chunk_index, start_offset, end_offset,
                text_cached, char_count, token_est, overlap_prev, overlap_next,
                review_status, is_hard_cut, created_at, updated_at
         FROM chunks WHERE id = ?1",
        [chunk_id],
        |row| {
            Ok(Chunk {
                id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_version_id: row.get(2)?,
                chunk_index: row.get(3)?,
                start_offset: row.get(4)?,
                end_offset: row.get(5)?,
                text_cached: row.get(6)?,
                char_count: row.get(7)?,
                token_est: row.get(8)?,
                overlap_prev: row.get(9)?,
                overlap_next: row.get(10)?,
                review_status: row.get(11)?,
                is_hard_cut: row.get::<_, i64>(12)? != 0,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        },
    )?;

    // 문서명 조회
    let document_name: String = conn.query_row(
        "SELECT display_name FROM documents WHERE id = ?1",
        [chunk.document_id],
        |row| row.get(0),
    )?;

    // 편집 텍스트 조회
    let edited_text: Option<String> = conn
        .query_row(
            "SELECT edited_text FROM chunk_edits WHERE chunk_id = ?1 ORDER BY edited_at DESC LIMIT 1",
            [chunk_id],
            |row| row.get(0),
        )
        .ok();

    // 이전/다음 청크 ID
    let prev_chunk_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM chunks WHERE document_id = ?1 AND chunk_index = ?2",
            [chunk.document_id, chunk.chunk_index - 1],
            |row| row.get(0),
        )
        .ok();

    let next_chunk_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM chunks WHERE document_id = ?1 AND chunk_index = ?2",
            [chunk.document_id, chunk.chunk_index + 1],
            |row| row.get(0),
        )
        .ok();

    let original_text = chunk.text_cached.clone().unwrap_or_default();

    Ok(ChunkDetail {
        chunk,
        original_text,
        edited_text,
        document_name,
        prev_chunk_id,
        next_chunk_id,
    })
}

/// 청크 상태 업데이트
pub fn update_chunk_status(pool: &DbPool, chunk_id: i64, status: &str) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE chunks SET review_status = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![status, chunk_id],
    )?;
    Ok(())
}

/// 청크 편집 저장
pub fn save_chunk_edit(pool: &DbPool, chunk_id: i64, edited_text: &str) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO chunk_edits (chunk_id, edited_text) VALUES (?1, ?2)",
        rusqlite::params![chunk_id, edited_text],
    )?;
    Ok(())
}

/// 프리셋 목록 조회
pub fn list_presets(pool: &DbPool) -> Result<Vec<Preset>> {
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT id, name, schema_version, mapping_json, validators_json, is_builtin, created_at
         FROM presets ORDER BY is_builtin DESC, name",
    )?;

    let presets = stmt
        .query_map([], |row| {
            Ok(Preset {
                id: row.get(0)?,
                name: row.get(1)?,
                schema_version: row.get(2)?,
                mapping_json: row.get(3)?,
                validators_json: row.get(4)?,
                is_builtin: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(presets)
}

/// Export 이력 조회
pub fn list_export_runs(pool: &DbPool) -> Result<Vec<ExportRun>> {
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT id, preset_id, preset_name, params_json, filters_json, stats_json, created_at
         FROM export_runs ORDER BY created_at DESC",
    )?;

    let runs = stmt
        .query_map([], |row| {
            Ok(ExportRun {
                id: row.get(0)?,
                preset_id: row.get(1)?,
                preset_name: row.get(2)?,
                params_json: row.get(3)?,
                filters_json: row.get(4)?,
                stats_json: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(runs)
}

/// 청크 검색 (FTS5)
pub fn search_chunks(pool: &DbPool, query: &str, limit: Option<i64>, offset: Option<i64>) -> Result<SearchResult> {
    let conn = pool.get()?;

    // 전체 결과 수 조회
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chunks_fts WHERE chunks_fts MATCH ?1",
        [query],
        |row| row.get(0),
    ).unwrap_or(0);

    // 청크 조회
    let mut sql = String::from(
        "SELECT c.id, c.document_id, c.chunk_version_id, c.chunk_index, c.start_offset, c.end_offset,
                c.text_cached, c.char_count, c.token_est, c.overlap_prev, c.overlap_next,
                c.review_status, c.is_hard_cut, c.created_at, c.updated_at
         FROM chunks c
         JOIN chunks_fts ON c.id = chunks_fts.rowid
         WHERE chunks_fts MATCH ?1
         ORDER BY rank",
    );

    let limit_val = limit.unwrap_or(50);
    let offset_val = offset.unwrap_or(0);
    sql.push_str(&format!(" LIMIT {} OFFSET {}", limit_val, offset_val));

    let mut stmt = conn.prepare(&sql)?;
    let chunks = stmt
        .query_map([query], |row| {
            Ok(Chunk {
                id: row.get(0)?,
                document_id: row.get(1)?,
                chunk_version_id: row.get(2)?,
                chunk_index: row.get(3)?,
                start_offset: row.get(4)?,
                end_offset: row.get(5)?,
                text_cached: row.get(6)?,
                char_count: row.get(7)?,
                token_est: row.get(8)?,
                overlap_prev: row.get(9)?,
                overlap_next: row.get(10)?,
                review_status: row.get(11)?,
                is_hard_cut: row.get::<_, i64>(12)? != 0,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(SearchResult { chunks, total })
}

/// 프리셋 생성
pub fn create_preset(pool: &DbPool, name: &str, mapping_json: &str, validators_json: Option<&str>) -> Result<Preset> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO presets (name, schema_version, mapping_json, validators_json, is_builtin)
         VALUES (?1, '1.0', ?2, ?3, 0)",
        rusqlite::params![name, mapping_json, validators_json],
    )?;

    let id = conn.last_insert_rowid();
    let preset = conn.query_row(
        "SELECT id, name, schema_version, mapping_json, validators_json, is_builtin, created_at
         FROM presets WHERE id = ?1",
        [id],
        |row| {
            Ok(Preset {
                id: row.get(0)?,
                name: row.get(1)?,
                schema_version: row.get(2)?,
                mapping_json: row.get(3)?,
                validators_json: row.get(4)?,
                is_builtin: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        },
    )?;

    Ok(preset)
}

/// 프리셋 업데이트
pub fn update_preset(pool: &DbPool, id: i64, name: &str, mapping_json: &str, validators_json: Option<&str>) -> Result<Preset> {
    let conn = pool.get()?;

    // builtin 프리셋은 수정 불가
    let is_builtin: i64 = conn.query_row(
        "SELECT is_builtin FROM presets WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;

    if is_builtin != 0 {
        return Err(crate::error::AppError::Internal("기본 프리셋은 수정할 수 없습니다".to_string()));
    }

    conn.execute(
        "UPDATE presets SET name = ?1, mapping_json = ?2, validators_json = ?3 WHERE id = ?4",
        rusqlite::params![name, mapping_json, validators_json, id],
    )?;

    let preset = conn.query_row(
        "SELECT id, name, schema_version, mapping_json, validators_json, is_builtin, created_at
         FROM presets WHERE id = ?1",
        [id],
        |row| {
            Ok(Preset {
                id: row.get(0)?,
                name: row.get(1)?,
                schema_version: row.get(2)?,
                mapping_json: row.get(3)?,
                validators_json: row.get(4)?,
                is_builtin: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
            })
        },
    )?;

    Ok(preset)
}

/// 프리셋 삭제
pub fn delete_preset(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;

    // builtin 프리셋은 삭제 불가
    let is_builtin: i64 = conn.query_row(
        "SELECT is_builtin FROM presets WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;

    if is_builtin != 0 {
        return Err(crate::error::AppError::Internal("기본 프리셋은 삭제할 수 없습니다".to_string()));
    }

    conn.execute("DELETE FROM presets WHERE id = ?1", [id])?;
    Ok(())
}

// ===== 매뉴얼 엔트리 CRUD =====

/// 매뉴얼 엔트리 생성
pub fn create_manual_entry(pool: &DbPool, preset_id: i64, data_json: &str) -> Result<ManualEntry> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO manual_entries (preset_id, data_json) VALUES (?1, ?2)",
        rusqlite::params![preset_id, data_json],
    )?;

    let id = conn.last_insert_rowid();
    get_manual_entry(pool, id)
}

/// 매뉴얼 엔트리 조회
pub fn get_manual_entry(pool: &DbPool, id: i64) -> Result<ManualEntry> {
    let conn = pool.get()?;
    let entry = conn.query_row(
        "SELECT id, preset_id, data_json, review_status, created_at, updated_at
         FROM manual_entries WHERE id = ?1",
        [id],
        |row| {
            Ok(ManualEntry {
                id: row.get(0)?,
                preset_id: row.get(1)?,
                data_json: row.get(2)?,
                review_status: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    )?;
    Ok(entry)
}

/// 매뉴얼 엔트리 목록 조회
pub fn list_manual_entries(
    pool: &DbPool,
    preset_id: Option<i64>,
    status: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ManualEntry>> {
    let conn = pool.get()?;

    let mut sql = String::from(
        "SELECT id, preset_id, data_json, review_status, created_at, updated_at
         FROM manual_entries WHERE 1=1",
    );

    if let Some(pid) = preset_id {
        sql.push_str(&format!(" AND preset_id = {}", pid));
    }
    if let Some(s) = status {
        sql.push_str(&format!(" AND review_status = '{}'", s));
    }

    sql.push_str(" ORDER BY created_at DESC");

    if let Some(l) = limit {
        sql.push_str(&format!(" LIMIT {}", l));
    }
    if let Some(o) = offset {
        sql.push_str(&format!(" OFFSET {}", o));
    }

    let mut stmt = conn.prepare(&sql)?;
    let entries = stmt
        .query_map([], |row| {
            Ok(ManualEntry {
                id: row.get(0)?,
                preset_id: row.get(1)?,
                data_json: row.get(2)?,
                review_status: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(entries)
}

/// 매뉴얼 엔트리 업데이트
pub fn update_manual_entry(pool: &DbPool, id: i64, data_json: &str) -> Result<ManualEntry> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE manual_entries SET data_json = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![data_json, id],
    )?;
    get_manual_entry(pool, id)
}

/// 매뉴얼 엔트리 상태 변경
pub fn update_manual_entry_status(pool: &DbPool, id: i64, status: &str) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE manual_entries SET review_status = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![status, id],
    )?;
    Ok(())
}

/// 매뉴얼 엔트리 삭제
pub fn delete_manual_entry(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM manual_entries WHERE id = ?1", [id])?;
    Ok(())
}

/// 매뉴얼 엔트리 통계
pub fn get_manual_entry_stats(pool: &DbPool, preset_id: Option<i64>) -> Result<(i64, i64, i64, i64)> {
    let conn = pool.get()?;

    let where_clause = preset_id.map_or(String::new(), |id| format!(" WHERE preset_id = {}", id));

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM manual_entries{}", where_clause),
        [],
        |row| row.get(0),
    )?;
    let pending: i64 = conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM manual_entries{} {} review_status = 'pending'",
            where_clause,
            if where_clause.is_empty() { "WHERE" } else { "AND" }
        ),
        [],
        |row| row.get(0),
    )?;
    let approved: i64 = conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM manual_entries{} {} review_status = 'approved'",
            where_clause,
            if where_clause.is_empty() { "WHERE" } else { "AND" }
        ),
        [],
        |row| row.get(0),
    )?;
    let rejected: i64 = conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM manual_entries{} {} review_status = 'rejected'",
            where_clause,
            if where_clause.is_empty() { "WHERE" } else { "AND" }
        ),
        [],
        |row| row.get(0),
    )?;

    Ok((total, pending, approved, rejected))
}

/// 프로젝트 통계 조회
pub fn get_project_stats(pool: &DbPool) -> Result<ProjectStats> {
    let conn = pool.get()?;

    let document_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
    let chunk_count: i64 = conn.query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))?;
    let pending_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chunks WHERE review_status = 'pending'",
        [],
        |row| row.get(0),
    )?;
    let approved_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chunks WHERE review_status = 'approved'",
        [],
        |row| row.get(0),
    )?;
    let skipped_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chunks WHERE review_status = 'skipped'",
        [],
        |row| row.get(0),
    )?;
    let rejected_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chunks WHERE review_status = 'rejected'",
        [],
        |row| row.get(0),
    )?;
    let export_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM export_runs", [], |row| row.get(0))?;

    let total_chars: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(char_count), 0) FROM chunks",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_tokens_est: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(token_est), 0) FROM chunks",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let avg_chunk_length: f64 = if chunk_count > 0 {
        total_chars as f64 / chunk_count as f64
    } else {
        0.0
    };

    let approval_rate: f64 = if chunk_count > 0 {
        approved_count as f64 / chunk_count as f64 * 100.0
    } else {
        0.0
    };

    Ok(ProjectStats {
        document_count,
        chunk_count,
        pending_count,
        approved_count,
        skipped_count,
        rejected_count,
        export_count,
        total_chars,
        total_tokens_est,
        avg_chunk_length,
        approval_rate,
    })
}
