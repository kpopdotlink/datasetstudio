//! 청크 관련 커맨드

use crate::core::chunker::{self, ChunkParams};
use crate::error::{AppError, Result};
use crate::jobs::{self, types::ChunkFilter, JobType};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// 청크 미리보기
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPreview {
    pub index: usize,
    pub text: String,
    pub char_count: usize,
    pub token_est: usize,
    pub overlap_prev: usize,
    pub overlap_next: usize,
    pub is_hard_cut: bool,
}

/// 청크 생성 잡 시작
#[tauri::command]
pub async fn start_chunk_job(
    state: State<'_, Mutex<AppState>>,
    app_handle: tauri::AppHandle,
    params: ChunkParams,
    filter: Option<ChunkFilter>,
) -> Result<i64> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;

    let job_id = jobs::spawn_job(
        pool.clone(),
        project_path.clone(),
        JobType::Chunk {
            params,
            filter: filter.unwrap_or_default(),
        },
        app_handle,
    )?;

    Ok(job_id)
}

/// 청크 미리보기
#[tauri::command]
pub async fn get_chunk_preview(
    state: State<'_, Mutex<AppState>>,
    doc_id: i64,
    params: ChunkParams,
    sample_size: usize,
) -> Result<Vec<ChunkPreview>> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;

    let previews = chunker::generate_preview(pool, project_path, doc_id, &params, sample_size)?;
    Ok(previews)
}

/// 잡 취소
#[tauri::command]
pub async fn cancel_job(state: State<'_, Mutex<AppState>>, job_id: i64) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    jobs::cancel_job(pool, job_id)?;
    Ok(())
}

/// 청크 병합 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedChunk {
    pub id: i64,
    pub text: String,
    pub char_count: i64,
    pub token_est: i64,
}

/// 두 인접한 청크 병합
#[tauri::command]
pub async fn merge_chunks(
    state: State<'_, Mutex<AppState>>,
    chunk_id_1: i64,
    chunk_id_2: i64,
) -> Result<MergedChunk> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let conn = pool.get()?;

    // 두 청크 조회
    let chunk1: (i64, i64, i64, Option<String>, i64, i64) = conn.query_row(
        "SELECT id, document_id, chunk_index, text_cached, char_count, token_est FROM chunks WHERE id = ?1",
        [chunk_id_1],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
    )?;

    let chunk2: (i64, i64, i64, Option<String>, i64, i64) = conn.query_row(
        "SELECT id, document_id, chunk_index, text_cached, char_count, token_est FROM chunks WHERE id = ?1",
        [chunk_id_2],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
    )?;

    // 같은 문서의 인접한 청크인지 확인
    if chunk1.1 != chunk2.1 {
        return Err(AppError::InvalidParameter("다른 문서의 청크는 병합할 수 없습니다".to_string()));
    }

    if (chunk1.2 - chunk2.2).abs() != 1 {
        return Err(AppError::InvalidParameter("인접하지 않은 청크는 병합할 수 없습니다".to_string()));
    }

    // 순서 정렬 (작은 인덱스가 먼저)
    let (first, second) = if chunk1.2 < chunk2.2 {
        (chunk1, chunk2)
    } else {
        (chunk2, chunk1)
    };

    // 텍스트 병합
    let text1 = first.3.clone().unwrap_or_default();
    let text2 = second.3.clone().unwrap_or_default();
    let merged_text = format!("{}\n\n{}", text1, text2);
    let char_count = merged_text.chars().count() as i64;
    let token_est = first.5 + second.5;

    // 첫 번째 청크 업데이트
    conn.execute(
        "UPDATE chunks SET text_cached = ?1, char_count = ?2, token_est = ?3,
         review_status = 'pending', updated_at = datetime('now') WHERE id = ?4",
        rusqlite::params![merged_text, char_count, token_est, first.0],
    )?;

    // 두 번째 청크 삭제
    conn.execute("DELETE FROM chunks WHERE id = ?1", [second.0])?;

    // 나머지 청크들의 인덱스 조정
    conn.execute(
        "UPDATE chunks SET chunk_index = chunk_index - 1, updated_at = datetime('now')
         WHERE document_id = ?1 AND chunk_index > ?2",
        rusqlite::params![first.1, second.2],
    )?;

    Ok(MergedChunk {
        id: first.0,
        text: merged_text,
        char_count,
        token_est,
    })
}

/// 청크 분할 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitChunks {
    pub first: MergedChunk,
    pub second: MergedChunk,
}

/// 청크를 특정 위치에서 분할
#[tauri::command]
pub async fn split_chunk(
    state: State<'_, Mutex<AppState>>,
    chunk_id: i64,
    split_position: usize,
) -> Result<SplitChunks> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let conn = pool.get()?;

    // 청크 조회
    let (id, doc_id, version_id, chunk_index, text, char_count, token_est): (i64, i64, i64, i64, Option<String>, i64, i64) = conn.query_row(
        "SELECT id, document_id, chunk_version_id, chunk_index, text_cached, char_count, token_est FROM chunks WHERE id = ?1",
        [chunk_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
    )?;

    let text = text.ok_or(AppError::InvalidParameter("청크 텍스트가 없습니다".to_string()))?;

    if split_position == 0 || split_position >= text.chars().count() {
        return Err(AppError::InvalidParameter("유효하지 않은 분할 위치입니다".to_string()));
    }

    // 텍스트 분할
    let chars: Vec<char> = text.chars().collect();
    let first_text: String = chars[..split_position].iter().collect();
    let second_text: String = chars[split_position..].iter().collect();

    let first_char_count = first_text.chars().count() as i64;
    let second_char_count = second_text.chars().count() as i64;

    // 토큰 추정 (비율로 분배)
    let ratio = first_char_count as f64 / char_count as f64;
    let first_token_est = (token_est as f64 * ratio).round() as i64;
    let second_token_est = token_est - first_token_est;

    // 뒤에 있는 청크들의 인덱스를 1 증가
    conn.execute(
        "UPDATE chunks SET chunk_index = chunk_index + 1, updated_at = datetime('now')
         WHERE document_id = ?1 AND chunk_index > ?2",
        rusqlite::params![doc_id, chunk_index],
    )?;

    // 첫 번째 청크 (기존 청크 업데이트)
    conn.execute(
        "UPDATE chunks SET text_cached = ?1, char_count = ?2, token_est = ?3,
         review_status = 'pending', updated_at = datetime('now') WHERE id = ?4",
        rusqlite::params![first_text, first_char_count, first_token_est, id],
    )?;

    // 두 번째 청크 (새로 생성)
    conn.execute(
        "INSERT INTO chunks (document_id, chunk_version_id, chunk_index, start_offset, end_offset,
         text_cached, char_count, token_est, overlap_prev, overlap_next, review_status, is_hard_cut)
         VALUES (?1, ?2, ?3, 0, 0, ?4, ?5, ?6, 0, 0, 'pending', 1)",
        rusqlite::params![doc_id, version_id, chunk_index + 1, second_text, second_char_count, second_token_est],
    )?;

    let new_id = conn.last_insert_rowid();

    Ok(SplitChunks {
        first: MergedChunk {
            id,
            text: first_text,
            char_count: first_char_count,
            token_est: first_token_est,
        },
        second: MergedChunk {
            id: new_id,
            text: second_text,
            char_count: second_char_count,
            token_est: second_token_est,
        },
    })
}
