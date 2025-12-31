//! 잡 실행기

use crate::core::chunker::{ChunkParams, Chunker};
use crate::core::exporter;
use crate::core::ingest;
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::jobs::types::*;
use encoding_rs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

/// 파일을 인코딩 자동 감지하여 읽기
fn read_file_with_encoding(path: &str) -> Result<String> {
    let bytes = std::fs::read(path)?;

    // BOM 확인
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Ok(String::from_utf8_lossy(&bytes[3..]).to_string());
    }

    // UTF-8 시도
    if let Ok(text) = std::str::from_utf8(&bytes) {
        return Ok(text.to_string());
    }

    // EUC-KR/CP949 시도
    let (text, _, _) = encoding_rs::EUC_KR.decode(&bytes);
    Ok(text.to_string())
}

/// 잡 생성 및 실행
pub fn spawn_job(
    pool: DbPool,
    project_path: PathBuf,
    job_type: JobType,
    app_handle: AppHandle,
) -> Result<i64> {
    let conn = pool.get()?;

    // 잡 생성
    let payload = serde_json::to_string(&job_type)?;
    conn.execute(
        "INSERT INTO jobs (job_type, status, payload_json) VALUES (?1, 'running', ?2)",
        rusqlite::params![job_type.to_string(), payload],
    )?;

    let job_id = conn.last_insert_rowid();
    drop(conn);

    // 비동기 실행
    let pool_clone = pool.clone();
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    std::thread::spawn(move || {
        let result = match job_type {
            JobType::Scan => run_scan_job(&pool_clone, &project_path, job_id, &app_handle),
            JobType::Chunk { params, filter } => run_chunk_job(
                &pool_clone,
                &project_path,
                job_id,
                &params,
                &filter,
                &app_handle,
                &cancelled_clone,
            ),
            JobType::Export { config } => {
                run_export_job(&pool_clone, &project_path, job_id, &config, &app_handle)
            }
        };

        // 결과 처리
        if let Err(e) = result {
            let conn = pool_clone.get().ok();
            if let Some(conn) = conn {
                let _ = conn.execute(
                    "UPDATE jobs SET status = 'failed', error_message = ?1, updated_at = datetime('now') WHERE id = ?2",
                    rusqlite::params![e.to_string(), job_id],
                );
            }

            let _ = app_handle.emit(
                "job-failed",
                JobEvent {
                    job_id,
                    event_type: JobEventType::Failed {
                        error: e.to_string(),
                    },
                },
            );
        }
    });

    Ok(job_id)
}

/// 잡 취소
pub fn cancel_job(pool: &DbPool, job_id: i64) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE jobs SET status = 'cancelled', updated_at = datetime('now') WHERE id = ?1 AND status = 'running'",
        [job_id],
    )?;
    Ok(())
}

/// 스캔 잡 실행
fn run_scan_job(
    pool: &DbPool,
    project_path: &PathBuf,
    job_id: i64,
    app_handle: &AppHandle,
) -> Result<()> {
    let result = ingest::scan_all_sources(pool, project_path)?;

    // 완료
    let conn = pool.get()?;
    conn.execute(
        "UPDATE jobs SET status = 'completed', progress = 1.0, updated_at = datetime('now') WHERE id = ?1",
        [job_id],
    )?;

    let _ = app_handle.emit(
        "job-completed",
        JobEvent {
            job_id,
            event_type: JobEventType::Completed {
                result: Some(serde_json::to_string(&result)?),
            },
        },
    );

    Ok(())
}

/// 청크 잡 실행
fn run_chunk_job(
    pool: &DbPool,
    project_path: &PathBuf,
    job_id: i64,
    params: &ChunkParams,
    filter: &crate::jobs::types::ChunkFilter,
    app_handle: &AppHandle,
    cancelled: &AtomicBool,
) -> Result<()> {
    let conn = pool.get()?;

    // 문서 목록 조회 (필터 적용)
    let mut sql = String::from(
        "SELECT d.id, d.original_path, d.checksum FROM documents d WHERE 1=1"
    );

    // 상태 필터
    if filter.rechunk {
        sql.push_str(" AND d.status IN ('unprocessed', 'chunked')");
    } else {
        sql.push_str(" AND d.status = 'unprocessed'");
    }

    // 문서 ID 필터
    if let Some(ref doc_ids) = filter.document_ids {
        if !doc_ids.is_empty() {
            let ids: Vec<String> = doc_ids.iter().map(|id| id.to_string()).collect();
            sql.push_str(&format!(" AND d.id IN ({})", ids.join(",")));
        }
    }

    // 소스 ID 필터
    if let Some(ref source_ids) = filter.source_ids {
        if !source_ids.is_empty() {
            let ids: Vec<String> = source_ids.iter().map(|id| id.to_string()).collect();
            sql.push_str(&format!(" AND d.source_id IN ({})", ids.join(",")));
        }
    }

    // 정렬: 소스별, 파일명순
    sql.push_str(" ORDER BY d.source_id, d.display_name");

    let mut stmt = conn.prepare(&sql)?;
    let documents: Vec<(i64, Option<String>, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    if documents.is_empty() {
        // 완료
        conn.execute(
            "UPDATE jobs SET status = 'completed', progress = 1.0, updated_at = datetime('now') WHERE id = ?1",
            [job_id],
        )?;

        let _ = app_handle.emit(
            "job-completed",
            JobEvent {
                job_id,
                event_type: JobEventType::Completed { result: None },
            },
        );

        return Ok(());
    }

    // 청크 버전 생성
    let params_json = serde_json::to_string(params)?;
    conn.execute(
        "INSERT INTO chunk_versions (params_json) VALUES (?1)",
        [&params_json],
    )?;
    let chunk_version_id = conn.last_insert_rowid();

    let total = documents.len();
    let chunker = Chunker::new(params);

    for (idx, (doc_id, original_path, checksum)) in documents.iter().enumerate() {
        // 취소 확인
        if cancelled.load(Ordering::Relaxed) {
            return Err(AppError::JobCancelled);
        }

        // rechunk인 경우 기존 청크 삭제
        if filter.rechunk {
            conn.execute("DELETE FROM chunks WHERE document_id = ?1", [doc_id])?;
        }

        // 텍스트 읽기 (인코딩 자동 감지)
        let text = if let Some(path) = original_path {
            read_file_with_encoding(path)?
        } else {
            // 캐시는 UTF-8로 저장됨
            let cache_path = project_path.join("cache").join(checksum);
            std::fs::read_to_string(&cache_path)?
        };

        // 청크 생성
        let chunks = chunker.chunk(&text);

        // DB에 저장
        for chunk in chunks {
            conn.execute(
                "INSERT INTO chunks (document_id, chunk_version_id, chunk_index, start_offset, end_offset, text_cached, char_count, token_est, overlap_prev, overlap_next, is_hard_cut)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    doc_id,
                    chunk_version_id,
                    chunk.index,
                    chunk.start_offset,
                    chunk.end_offset,
                    chunk.text,
                    chunk.char_count,
                    chunk.token_est,
                    chunk.overlap_prev,
                    chunk.overlap_next,
                    chunk.is_hard_cut as i64
                ],
            )?;
        }

        // 문서 상태 업데이트
        conn.execute(
            "UPDATE documents SET status = 'chunked', updated_at = datetime('now') WHERE id = ?1",
            [doc_id],
        )?;

        // 진행률 업데이트
        let progress = (idx + 1) as f64 / total as f64;
        conn.execute(
            "UPDATE jobs SET progress = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![progress, job_id],
        )?;

        let _ = app_handle.emit(
            "job-progress",
            JobEvent {
                job_id,
                event_type: JobEventType::Progress {
                    progress,
                    message: Some(format!("처리 중: {}/{}", idx + 1, total)),
                },
            },
        );
    }

    // 완료
    conn.execute(
        "UPDATE jobs SET status = 'completed', progress = 1.0, updated_at = datetime('now') WHERE id = ?1",
        [job_id],
    )?;

    let _ = app_handle.emit(
        "job-completed",
        JobEvent {
            job_id,
            event_type: JobEventType::Completed { result: None },
        },
    );

    Ok(())
}

/// Export 잡 실행
fn run_export_job(
    pool: &DbPool,
    project_path: &PathBuf,
    job_id: i64,
    config: &crate::core::exporter::ExportConfig,
    app_handle: &AppHandle,
) -> Result<()> {
    let manifest = exporter::export_to_jsonl(pool, project_path, config)?;

    // 완료
    let conn = pool.get()?;
    conn.execute(
        "UPDATE jobs SET status = 'completed', progress = 1.0, updated_at = datetime('now') WHERE id = ?1",
        [job_id],
    )?;

    let _ = app_handle.emit(
        "job-completed",
        JobEvent {
            job_id,
            event_type: JobEventType::Completed {
                result: Some(serde_json::to_string(&manifest)?),
            },
        },
    );

    Ok(())
}
