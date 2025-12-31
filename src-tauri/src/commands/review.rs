//! 리뷰 관련 커맨드

use crate::core::quality::{self, QualityCheckResult, QualityRules};
use crate::db::models::Chunk;
use crate::db::repository;
use crate::error::{AppError, Result};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// 리뷰 필터
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReviewFilters {
    pub status: Option<String>,
    pub document_id: Option<i64>,
    pub source_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 일괄 리뷰 필터
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BulkReviewFilter {
    pub document_ids: Option<Vec<i64>>,
    pub source_ids: Option<Vec<i64>>,
    pub current_status: Option<String>,
}

/// 리뷰 큐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewQueue {
    pub chunks: Vec<Chunk>,
    pub total: i64,
    pub pending: i64,
    pub approved: i64,
    pub skipped: i64,
}

/// 청크 상세
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkDetail {
    pub chunk: Chunk,
    pub original_text: String,
    pub edited_text: Option<String>,
    pub document_name: String,
    pub prev_chunk_id: Option<i64>,
    pub next_chunk_id: Option<i64>,
}

/// 리뷰 상태
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "skipped")]
    Skipped,
    #[serde(rename = "rejected")]
    Rejected,
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewStatus::Pending => write!(f, "pending"),
            ReviewStatus::Approved => write!(f, "approved"),
            ReviewStatus::Skipped => write!(f, "skipped"),
            ReviewStatus::Rejected => write!(f, "rejected"),
        }
    }
}

/// 리뷰 큐 조회
#[tauri::command]
pub async fn get_review_queue(
    state: State<'_, Mutex<AppState>>,
    filters: ReviewFilters,
) -> Result<ReviewQueue> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let queue = repository::get_review_queue(pool, &filters)?;
    Ok(queue)
}

/// 청크 상세 조회
#[tauri::command]
pub async fn get_chunk_detail(
    state: State<'_, Mutex<AppState>>,
    chunk_id: i64,
) -> Result<ChunkDetail> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;

    let detail = repository::get_chunk_detail(pool, project_path, chunk_id)?;
    Ok(detail)
}

/// 리뷰 상태 변경
#[tauri::command]
pub async fn set_review_status(
    state: State<'_, Mutex<AppState>>,
    chunk_id: i64,
    status: ReviewStatus,
) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    repository::update_chunk_status(pool, chunk_id, &status.to_string())?;
    Ok(())
}

/// 청크 편집 저장
#[tauri::command]
pub async fn save_chunk_edit(
    state: State<'_, Mutex<AppState>>,
    chunk_id: i64,
    edited_text: String,
) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    repository::save_chunk_edit(pool, chunk_id, &edited_text)?;
    Ok(())
}

/// 청크 검색 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunks: Vec<Chunk>,
    pub total: i64,
}

/// 청크 전체 텍스트 검색
#[tauri::command]
pub async fn search_chunks(
    state: State<'_, Mutex<AppState>>,
    query: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<SearchResult> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let result = repository::search_chunks(pool, &query, limit, offset)?;
    Ok(result)
}

/// 청크 품질 검사
#[tauri::command]
pub async fn check_chunk_quality(
    state: State<'_, Mutex<AppState>>,
    chunk_id: i64,
    rules: Option<QualityRules>,
) -> Result<QualityCheckResult> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;

    let detail = repository::get_chunk_detail(pool, project_path, chunk_id)?;
    let text = detail.edited_text.as_ref().unwrap_or(&detail.original_text);
    let token_count = detail.chunk.token_est as usize;

    let rules = rules.unwrap_or_default();
    let result = quality::check_quality(text, token_count, &rules);

    Ok(result)
}

/// 기본 품질 규칙 조회
#[tauri::command]
pub async fn get_default_quality_rules() -> Result<QualityRules> {
    Ok(QualityRules::default())
}

/// 일괄 리뷰 상태 변경
#[tauri::command]
pub async fn bulk_set_review_status(
    state: State<'_, Mutex<AppState>>,
    filter: BulkReviewFilter,
    status: ReviewStatus,
) -> Result<i64> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let conn = pool.get()?;

    // SQL 구성
    let mut sql = String::from("UPDATE chunks SET review_status = ?1, updated_at = datetime('now') WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(status.to_string())];

    // 현재 상태 필터
    if let Some(ref current) = filter.current_status {
        sql.push_str(" AND review_status = ?");
        params.push(Box::new(current.clone()));
    }

    // 문서 ID 필터
    if let Some(ref doc_ids) = filter.document_ids {
        if !doc_ids.is_empty() {
            let ids: Vec<String> = doc_ids.iter().map(|id| id.to_string()).collect();
            sql.push_str(&format!(" AND document_id IN ({})", ids.join(",")));
        }
    }

    // 소스 ID 필터 (문서를 통해)
    if let Some(ref source_ids) = filter.source_ids {
        if !source_ids.is_empty() {
            let ids: Vec<String> = source_ids.iter().map(|id| id.to_string()).collect();
            sql.push_str(&format!(
                " AND document_id IN (SELECT id FROM documents WHERE source_id IN ({}))",
                ids.join(",")
            ));
        }
    }

    // 파라미터를 rusqlite::params로 변환
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let affected = conn.execute(&sql, param_refs.as_slice())?;

    Ok(affected as i64)
}
