//! 소스/문서 관련 커맨드

use crate::core::ingest::{self, SourceType};
use crate::db::models::{Document, Source};
use crate::db::repository;
use crate::error::{AppError, Result};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// 문서 필터
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentFilters {
    pub status: Option<String>,
    pub source_id: Option<i64>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 스캔 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub total_files: usize,
    pub new_files: usize,
    pub skipped_files: usize,
    pub errors: Vec<String>,
}

/// 소스 추가
#[tauri::command]
pub async fn add_source(
    state: State<'_, Mutex<AppState>>,
    source_type: String,
    path_or_content: String,
    display_name: Option<String>,
) -> Result<Source> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;

    let st = match source_type.as_str() {
        "folder" => SourceType::Folder,
        "file" => SourceType::File,
        "manual" => SourceType::Manual,
        _ => return Err(AppError::InvalidParameter("source_type".to_string())),
    };

    let source = ingest::add_source(pool, project_path, st, &path_or_content, display_name)?;
    Ok(source)
}

/// 소스 스캔
#[tauri::command]
pub async fn scan_sources(state: State<'_, Mutex<AppState>>) -> Result<ScanResult> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;

    let result = ingest::scan_all_sources(pool, project_path)?;
    Ok(result)
}

/// 문서 목록 조회
#[tauri::command]
pub async fn list_documents(
    state: State<'_, Mutex<AppState>>,
    filters: DocumentFilters,
) -> Result<Vec<Document>> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let documents = repository::list_documents(pool, &filters)?;
    Ok(documents)
}

/// 문서 삭제
#[tauri::command]
pub async fn remove_document(state: State<'_, Mutex<AppState>>, doc_id: i64) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    repository::delete_document(pool, doc_id)?;
    Ok(())
}

/// 소스 목록 조회
#[tauri::command]
pub async fn list_sources(state: State<'_, Mutex<AppState>>) -> Result<Vec<Source>> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT id, type, path_or_key, display_name, added_at, meta_json FROM sources ORDER BY added_at DESC"
    )?;

    let sources = stmt
        .query_map([], |row| {
            Ok(Source {
                id: row.get(0)?,
                source_type: row.get(1)?,
                path_or_key: row.get(2)?,
                display_name: row.get(3)?,
                added_at: row.get(4)?,
                meta_json: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(sources)
}

/// 소스 삭제 (연관 문서도 삭제됨)
#[tauri::command]
pub async fn remove_source(state: State<'_, Mutex<AppState>>, source_id: i64) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let conn = pool.get()?;

    // CASCADE 설정으로 연관 문서도 자동 삭제됨
    conn.execute("DELETE FROM sources WHERE id = ?1", [source_id])?;
    Ok(())
}

/// 단일 소스 재스캔
#[tauri::command]
pub async fn rescan_source(state: State<'_, Mutex<AppState>>, source_id: i64) -> Result<ScanResult> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;

    let result = ingest::rescan_source(pool, project_path, source_id)?;
    Ok(result)
}

/// 직접 입력 문서 텍스트 조회
#[tauri::command]
pub async fn get_manual_document_text(
    state: State<'_, Mutex<AppState>>,
    doc_id: i64,
) -> Result<String> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;
    let conn = pool.get()?;

    let checksum: String = conn.query_row(
        "SELECT checksum FROM documents WHERE id = ?1",
        [doc_id],
        |row| row.get(0),
    )?;

    let cache_path = project_path.join("cache").join(&checksum);
    let text = std::fs::read_to_string(&cache_path)?;
    Ok(text)
}

/// 문서 변경 정보 (프론트엔드용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChangeInfo {
    pub document_id: i64,
    pub display_name: String,
    pub original_path: String,
    pub old_checksum: String,
    pub new_checksum: String,
    pub change_type: String,
}

/// 문서 변경 감지 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDetectionResult {
    pub changes: Vec<DocumentChangeInfo>,
    pub total_checked: usize,
}

/// 문서 변경 감지
#[tauri::command]
pub async fn check_document_changes(
    state: State<'_, Mutex<AppState>>,
) -> Result<ChangeDetectionResult> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    // 전체 문서 수 조회
    let conn = pool.get()?;
    let total_checked: usize = conn.query_row(
        "SELECT COUNT(*) FROM documents d JOIN sources s ON d.source_id = s.id WHERE s.type != 'manual'",
        [],
        |row| row.get::<_, i64>(0),
    )? as usize;

    let changes = ingest::check_document_changes(pool)?;

    let change_infos: Vec<DocumentChangeInfo> = changes
        .into_iter()
        .map(|c| DocumentChangeInfo {
            document_id: c.document_id,
            display_name: c.display_name,
            original_path: c.original_path,
            old_checksum: c.old_checksum,
            new_checksum: c.new_checksum,
            change_type: c.change_type.to_string(),
        })
        .collect();

    Ok(ChangeDetectionResult {
        changes: change_infos,
        total_checked,
    })
}

/// 문서 변경 처리
#[tauri::command]
pub async fn resolve_document_change(
    state: State<'_, Mutex<AppState>>,
    document_id: i64,
    action: String,
) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    ingest::apply_document_change(pool, document_id, &action)?;
    Ok(())
}
