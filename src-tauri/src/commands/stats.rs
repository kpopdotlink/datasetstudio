//! 통계 관련 커맨드

use crate::db::repository;
use crate::error::{AppError, Result};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// 프로젝트 통계
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub document_count: i64,
    pub chunk_count: i64,
    pub pending_count: i64,
    pub approved_count: i64,
    pub skipped_count: i64,
    pub rejected_count: i64,
    pub export_count: i64,
    pub total_chars: i64,
    pub total_tokens_est: i64,
    pub avg_chunk_length: f64,
    pub approval_rate: f64,
}

/// 프로젝트 통계 조회
#[tauri::command]
pub async fn get_project_stats(state: State<'_, Mutex<AppState>>) -> Result<ProjectStats> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let stats = repository::get_project_stats(pool)?;
    Ok(stats)
}
