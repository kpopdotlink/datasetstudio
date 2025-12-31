//! Export 관련 커맨드

use crate::core::exporter::ExportConfig;
use crate::db::models::{ExportRun, Preset};
use crate::db::repository;
use crate::error::{AppError, Result};
use crate::jobs::{self, JobType};
use crate::presets;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// Export 필터
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportFilters {
    pub approved_only: bool,
    pub document_ids: Option<Vec<i64>>,
    pub source_ids: Option<Vec<i64>>,
    pub tags: Option<Vec<String>>,
    pub include_manual_entries: Option<bool>,
    /// 소스별로 별도 파일로 내보내기
    pub split_by_source: Option<bool>,
}

/// 프리셋 목록 조회
#[tauri::command]
pub async fn list_presets(state: State<'_, Mutex<AppState>>) -> Result<Vec<Preset>> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    // 기본 프리셋이 없으면 생성
    presets::ensure_builtin_presets(pool)?;

    let presets = repository::list_presets(pool)?;
    Ok(presets)
}

/// Export 잡 시작
#[tauri::command]
pub async fn start_export_job(
    state: State<'_, Mutex<AppState>>,
    app_handle: tauri::AppHandle,
    preset_id: i64,
    filters: ExportFilters,
    max_file_size_mb: Option<usize>,
) -> Result<i64> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;

    let config = ExportConfig {
        preset_id,
        filters,
        max_file_size_mb: max_file_size_mb.unwrap_or(100),
    };

    let job_id = jobs::spawn_job(
        pool.clone(),
        project_path.clone(),
        JobType::Export { config },
        app_handle,
    )?;

    Ok(job_id)
}

/// Export 이력 조회
#[tauri::command]
pub async fn list_export_runs(state: State<'_, Mutex<AppState>>) -> Result<Vec<ExportRun>> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let runs = repository::list_export_runs(pool)?;
    Ok(runs)
}

/// Export 폴더 열기
#[tauri::command]
pub async fn open_export_folder(state: State<'_, Mutex<AppState>>) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;

    let exports_path = project_path.join("exports");

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&exports_path)
            .spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&exports_path)
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&exports_path)
            .spawn()?;
    }

    Ok(())
}

/// 업로드 명령 템플릿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadCommand {
    pub platform: String,
    pub name: String,
    pub command: String,
    pub description: String,
}

/// 프리셋 생성
#[tauri::command]
pub async fn create_preset(
    state: State<'_, Mutex<AppState>>,
    name: String,
    mapping_json: String,
    validators_json: Option<String>,
) -> Result<Preset> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let preset = repository::create_preset(
        pool,
        &name,
        &mapping_json,
        validators_json.as_deref(),
    )?;

    Ok(preset)
}

/// 프리셋 업데이트
#[tauri::command]
pub async fn update_preset(
    state: State<'_, Mutex<AppState>>,
    preset_id: i64,
    name: String,
    mapping_json: String,
    validators_json: Option<String>,
) -> Result<Preset> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    let preset = repository::update_preset(
        pool,
        preset_id,
        &name,
        &mapping_json,
        validators_json.as_deref(),
    )?;

    Ok(preset)
}

/// 프리셋 삭제
#[tauri::command]
pub async fn delete_preset(
    state: State<'_, Mutex<AppState>>,
    preset_id: i64,
) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    repository::delete_preset(pool, preset_id)?;

    Ok(())
}

/// 업로드 CLI 명령 생성
#[tauri::command]
pub async fn generate_upload_commands(
    state: State<'_, Mutex<AppState>>,
    export_id: String,
    dataset_name: String,
) -> Result<Vec<UploadCommand>> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;

    let export_path = project_path.join("exports").join(&export_id);
    let export_path_str = export_path.to_string_lossy();

    let mut commands = Vec::new();

    // HuggingFace Hub CLI
    commands.push(UploadCommand {
        platform: "huggingface".to_string(),
        name: "HuggingFace Hub".to_string(),
        command: format!(
            "huggingface-cli upload {} {} --repo-type dataset",
            dataset_name, export_path_str
        ),
        description: "HuggingFace Hub에 데이터셋 업로드. 먼저 `pip install huggingface_hub` 및 `huggingface-cli login` 실행 필요".to_string(),
    });

    // HuggingFace Hub Python
    commands.push(UploadCommand {
        platform: "huggingface_python".to_string(),
        name: "HuggingFace (Python)".to_string(),
        command: format!(
            r#"from huggingface_hub import HfApi
api = HfApi()
api.upload_folder(
    folder_path="{}",
    repo_id="{}",
    repo_type="dataset"
)"#,
            export_path_str, dataset_name
        ),
        description: "Python을 통한 HuggingFace 업로드".to_string(),
    });

    // AWS S3
    commands.push(UploadCommand {
        platform: "aws_s3".to_string(),
        name: "AWS S3".to_string(),
        command: format!(
            "aws s3 sync {} s3://<bucket-name>/{}/",
            export_path_str, dataset_name
        ),
        description: "AWS S3 버킷에 동기화. AWS CLI 설정 필요".to_string(),
    });

    // Google Cloud Storage
    commands.push(UploadCommand {
        platform: "gcs".to_string(),
        name: "Google Cloud Storage".to_string(),
        command: format!(
            "gsutil -m cp -r {}/* gs://<bucket-name>/{}/",
            export_path_str, dataset_name
        ),
        description: "Google Cloud Storage에 업로드. gcloud CLI 설정 필요".to_string(),
    });

    // rsync
    commands.push(UploadCommand {
        platform: "rsync".to_string(),
        name: "rsync (원격 서버)".to_string(),
        command: format!(
            "rsync -avz {}/ <user>@<host>:/path/to/{}/",
            export_path_str, dataset_name
        ),
        description: "rsync를 통한 원격 서버 전송".to_string(),
    });

    // Simple copy
    commands.push(UploadCommand {
        platform: "copy".to_string(),
        name: "로컬 복사".to_string(),
        command: format!(
            "cp -r {} /path/to/destination/{}/",
            export_path_str, dataset_name
        ),
        description: "로컬 파일 시스템에 복사".to_string(),
    });

    Ok(commands)
}
