//! 매뉴얼 데이터셋 관련 커맨드

use crate::db::models::{ManualEntry, Preset};
use crate::db::repository;
use crate::error::{AppError, Result};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// 매뉴얼 엔트리 필터
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManualEntryFilters {
    pub preset_id: Option<i64>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 매뉴얼 엔트리 목록 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualEntryListResult {
    pub entries: Vec<ManualEntry>,
    pub total: i64,
    pub pending: i64,
    pub approved: i64,
    pub rejected: i64,
    pub preset: Option<Preset>,
}

/// 매뉴얼 엔트리 생성
#[tauri::command]
pub async fn create_manual_entry(
    state: State<'_, Mutex<AppState>>,
    preset_id: i64,
    data_json: String,
) -> Result<ManualEntry> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let entry = repository::create_manual_entry(pool, preset_id, &data_json)?;
    Ok(entry)
}

/// 매뉴얼 엔트리 목록 조회
#[tauri::command]
pub async fn list_manual_entries(
    state: State<'_, Mutex<AppState>>,
    filters: ManualEntryFilters,
) -> Result<ManualEntryListResult> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let entries = repository::list_manual_entries(
        pool,
        filters.preset_id,
        filters.status.as_deref(),
        filters.limit,
        filters.offset,
    )?;

    let (total, pending, approved, rejected) =
        repository::get_manual_entry_stats(pool, filters.preset_id)?;

    // 프리셋 정보 조회
    let preset = if let Some(preset_id) = filters.preset_id {
        let conn = pool.get()?;
        conn.query_row(
            "SELECT id, name, schema_version, mapping_json, validators_json, is_builtin, created_at
             FROM presets WHERE id = ?1",
            [preset_id],
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
        )
        .ok()
    } else {
        None
    };

    Ok(ManualEntryListResult {
        entries,
        total,
        pending,
        approved,
        rejected,
        preset,
    })
}

/// 매뉴얼 엔트리 업데이트
#[tauri::command]
pub async fn update_manual_entry(
    state: State<'_, Mutex<AppState>>,
    entry_id: i64,
    data_json: String,
) -> Result<ManualEntry> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let entry = repository::update_manual_entry(pool, entry_id, &data_json)?;
    Ok(entry)
}

/// 매뉴얼 엔트리 상태 변경
#[tauri::command]
pub async fn set_manual_entry_status(
    state: State<'_, Mutex<AppState>>,
    entry_id: i64,
    status: String,
) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    repository::update_manual_entry_status(pool, entry_id, &status)?;
    Ok(())
}

/// 매뉴얼 엔트리 삭제
#[tauri::command]
pub async fn delete_manual_entry(
    state: State<'_, Mutex<AppState>>,
    entry_id: i64,
) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    repository::delete_manual_entry(pool, entry_id)?;
    Ok(())
}

/// JSONL 파일 임포트
#[tauri::command]
pub async fn import_jsonl(
    state: State<'_, Mutex<AppState>>,
    preset_id: i64,
    file_path: String,
) -> Result<i64> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let content = std::fs::read_to_string(&file_path)?;
    let mut count = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // JSON 유효성 검사
        if serde_json::from_str::<serde_json::Value>(line).is_ok() {
            repository::create_manual_entry(pool, preset_id, line)?;
            count += 1;
        }
    }

    Ok(count)
}

/// 프리셋 필드 목록 조회 (매핑에서 추출)
#[tauri::command]
pub async fn get_preset_fields(
    state: State<'_, Mutex<AppState>>,
    preset_id: i64,
) -> Result<Vec<String>> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let conn = pool.get()?;
    let mapping_json: String = conn.query_row(
        "SELECT mapping_json FROM presets WHERE id = ?1",
        [preset_id],
        |row| row.get(0),
    )?;

    let mapping: serde_json::Value = serde_json::from_str(&mapping_json)?;
    let fields = extract_fields_from_mapping(&mapping);

    Ok(fields)
}

/// 매핑에서 필드 추출
fn extract_fields_from_mapping(mapping: &serde_json::Value) -> Vec<String> {
    let mut fields = Vec::new();

    if let Some(obj) = mapping.as_object() {
        // messages 배열 (Chat 형식)
        if let Some(messages) = obj.get("messages") {
            if let Some(msgs) = messages.as_array() {
                for msg in msgs {
                    if let Some(msg_obj) = msg.as_object() {
                        if let Some(role) = msg_obj.get("role").and_then(|v| v.as_str()) {
                            if let Some(_content_type) = msg_obj.get("content").and_then(|v| v.as_str()) {
                                fields.push(format!("{}_{}", role, "content"));
                            }
                        }
                    }
                }
            }
        }
        // conversations 배열 (ShareGPT 형식)
        else if let Some(conversations) = obj.get("conversations") {
            if let Some(convs) = conversations.as_array() {
                for conv in convs {
                    if let Some(conv_obj) = conv.as_object() {
                        if let Some(from) = conv_obj.get("from").and_then(|v| v.as_str()) {
                            fields.push(format!("{}_value", from));
                        }
                    }
                }
            }
        }
        // 일반 객체
        else {
            for key in obj.keys() {
                fields.push(key.clone());
            }
        }
    }

    fields
}
