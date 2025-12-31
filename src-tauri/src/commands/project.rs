//! 프로젝트 관련 커맨드

use crate::core::chunker::ChunkParams;
use crate::db;
use crate::error::{AppError, Result};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};

/// 프로젝트 메타데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub settings: ProjectSettings,
}

/// 프로젝트 설정
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSettings {
    #[serde(default)]
    pub chunk_params: ChunkParams,
    #[serde(default = "default_export_format")]
    pub default_export_format: String,
    #[serde(default = "default_max_export_size")]
    pub max_export_file_size_mb: u64,
}

fn default_export_format() -> String {
    "text-only".to_string()
}

fn default_max_export_size() -> u64 {
    100
}

/// 최근 프로젝트 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened: String,
}

/// 앱 설정 (로컬 스토리지)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub recent_projects: Vec<RecentProject>,
    #[serde(default = "default_max_recent")]
    pub max_recent_projects: usize,
    #[serde(default)]
    pub theme: String,
}

fn default_max_recent() -> usize {
    10
}

/// 프로젝트 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub path: String,
    pub meta: ProjectMeta,
    pub document_count: i64,
    pub chunk_count: i64,
    pub approved_count: i64,
}

/// 프로젝트 생성
#[tauri::command]
pub async fn create_project(
    app: tauri::AppHandle,
    state: State<'_, Mutex<AppState>>,
    path: String,
    name: String,
) -> Result<ProjectInfo> {
    let project_path = PathBuf::from(&path);

    // 이미 존재하는지 확인
    if project_path.join("project.json").exists() {
        return Err(AppError::ProjectExists(path.clone()));
    }

    // 프로젝트 폴더 구조 생성
    fs::create_dir_all(&project_path)?;
    fs::create_dir_all(project_path.join("exports"))?;
    fs::create_dir_all(project_path.join("cache"))?;
    fs::create_dir_all(project_path.join("logs"))?;

    // project.json 생성
    let now = chrono::Utc::now().to_rfc3339();
    let meta = ProjectMeta {
        name: name.clone(),
        version: "0.1.0".to_string(),
        created_at: now.clone(),
        updated_at: now,
        settings: ProjectSettings::default(),
    };

    let project_json = serde_json::to_string_pretty(&meta)?;
    fs::write(project_path.join("project.json"), project_json)?;

    // 데이터베이스 초기화
    let db_path = project_path.join("db.sqlite");
    let pool = db::create_pool(&db_path)?;
    db::run_migrations(&pool)?;

    // 상태 업데이트
    {
        let mut app_state = state
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        app_state.db_pool = Some(pool);
        app_state.project_path = Some(project_path.clone());
    }

    // 최근 프로젝트 목록 업데이트
    if let Ok(mut settings) = load_app_settings(&app) {
        update_recent_projects(&mut settings, &path, &name);
        let _ = save_app_settings(&app, &settings);
    }

    Ok(ProjectInfo {
        path,
        meta,
        document_count: 0,
        chunk_count: 0,
        approved_count: 0,
    })
}

/// 프로젝트 열기
#[tauri::command]
pub async fn open_project(
    app: tauri::AppHandle,
    state: State<'_, Mutex<AppState>>,
    path: String,
) -> Result<ProjectInfo> {
    let project_path = PathBuf::from(&path);

    // project.json 읽기
    let project_json_path = project_path.join("project.json");
    if !project_json_path.exists() {
        return Err(AppError::InvalidProjectPath(path.clone()));
    }

    let project_json = fs::read_to_string(&project_json_path)?;
    let meta: ProjectMeta = serde_json::from_str(&project_json)?;

    // 데이터베이스 연결
    let db_path = project_path.join("db.sqlite");
    let pool = db::create_pool(&db_path)?;

    // 마이그레이션 실행 (버전 업그레이드 대응)
    db::run_migrations(&pool)?;

    // 통계 조회
    let conn = pool.get()?;
    let document_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
        .unwrap_or(0);
    let chunk_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
        .unwrap_or(0);
    let approved_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM chunks WHERE review_status = 'approved'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // 상태 업데이트
    {
        let mut app_state = state
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        app_state.db_pool = Some(pool);
        app_state.project_path = Some(project_path);
    }

    // 최근 프로젝트 목록 업데이트
    if let Ok(mut settings) = load_app_settings(&app) {
        update_recent_projects(&mut settings, &path, &meta.name);
        let _ = save_app_settings(&app, &settings);
    }

    Ok(ProjectInfo {
        path,
        meta,
        document_count,
        chunk_count,
        approved_count,
    })
}

/// 프로젝트 닫기
#[tauri::command]
pub async fn close_project(state: State<'_, Mutex<AppState>>) -> Result<()> {
    let mut app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    app_state.db_pool = None;
    app_state.project_path = None;
    Ok(())
}

/// 프로젝트 정보 조회
#[tauri::command]
pub async fn get_project_info(state: State<'_, Mutex<AppState>>) -> Result<ProjectInfo> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;
    let pool = app_state.db_pool.as_ref().ok_or(AppError::NoProject)?;

    // project.json 읽기
    let project_json = fs::read_to_string(project_path.join("project.json"))?;
    let meta: ProjectMeta = serde_json::from_str(&project_json)?;

    // 통계 조회
    let conn = pool.get()?;
    let document_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
        .unwrap_or(0);
    let chunk_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
        .unwrap_or(0);
    let approved_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM chunks WHERE review_status = 'approved'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(ProjectInfo {
        path: project_path.to_string_lossy().to_string(),
        meta,
        document_count,
        chunk_count,
        approved_count,
    })
}

/// 앱 설정 파일 경로 반환
fn get_app_settings_path(app: &tauri::AppHandle) -> Result<PathBuf> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("앱 데이터 디렉토리 조회 실패: {}", e))
    })?;
    fs::create_dir_all(&app_data_dir)?;
    Ok(app_data_dir.join("settings.json"))
}

/// 앱 설정 로드
fn load_app_settings(app: &tauri::AppHandle) -> Result<AppSettings> {
    let settings_path = get_app_settings_path(app)?;
    if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    } else {
        Ok(AppSettings::default())
    }
}

/// 앱 설정 저장
fn save_app_settings(app: &tauri::AppHandle, settings: &AppSettings) -> Result<()> {
    let settings_path = get_app_settings_path(app)?;
    let content = serde_json::to_string_pretty(settings)?;
    fs::write(&settings_path, content)?;
    Ok(())
}

/// 최근 프로젝트 추가/업데이트
fn update_recent_projects(
    settings: &mut AppSettings,
    path: &str,
    name: &str,
) {
    let now = chrono::Utc::now().to_rfc3339();

    // 기존 항목 제거
    settings.recent_projects.retain(|p| p.path != path);

    // 새 항목 추가 (맨 앞에)
    settings.recent_projects.insert(0, RecentProject {
        path: path.to_string(),
        name: name.to_string(),
        last_opened: now,
    });

    // 최대 개수 제한
    if settings.recent_projects.len() > settings.max_recent_projects {
        settings.recent_projects.truncate(settings.max_recent_projects);
    }
}

/// 최근 프로젝트 목록 조회
#[tauri::command]
pub async fn get_recent_projects(app: tauri::AppHandle) -> Result<Vec<RecentProject>> {
    let settings = load_app_settings(&app)?;

    // 존재하지 않는 프로젝트 필터링
    let valid_projects: Vec<RecentProject> = settings
        .recent_projects
        .into_iter()
        .filter(|p| PathBuf::from(&p.path).join("project.json").exists())
        .collect();

    Ok(valid_projects)
}

/// 최근 프로젝트에서 제거
#[tauri::command]
pub async fn remove_recent_project(app: tauri::AppHandle, path: String) -> Result<()> {
    let mut settings = load_app_settings(&app)?;
    settings.recent_projects.retain(|p| p.path != path);
    save_app_settings(&app, &settings)?;
    Ok(())
}

/// 프로젝트 설정 조회
#[tauri::command]
pub async fn get_project_settings(state: State<'_, Mutex<AppState>>) -> Result<ProjectSettings> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;
    let project_json = fs::read_to_string(project_path.join("project.json"))?;
    let meta: ProjectMeta = serde_json::from_str(&project_json)?;

    Ok(meta.settings)
}

/// 프로젝트 설정 업데이트
#[tauri::command]
pub async fn update_project_settings(
    state: State<'_, Mutex<AppState>>,
    settings: ProjectSettings,
) -> Result<()> {
    let app_state = state
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let project_path = app_state.project_path.as_ref().ok_or(AppError::NoProject)?;
    let project_json_path = project_path.join("project.json");

    let project_json = fs::read_to_string(&project_json_path)?;
    let mut meta: ProjectMeta = serde_json::from_str(&project_json)?;

    meta.settings = settings;
    meta.updated_at = chrono::Utc::now().to_rfc3339();

    let content = serde_json::to_string_pretty(&meta)?;
    fs::write(&project_json_path, content)?;

    Ok(())
}

/// 앱 테마 설정 조회
#[tauri::command]
pub async fn get_app_theme(app: tauri::AppHandle) -> Result<String> {
    let settings = load_app_settings(&app)?;
    Ok(settings.theme)
}

/// 앱 테마 설정 업데이트
#[tauri::command]
pub async fn set_app_theme(app: tauri::AppHandle, theme: String) -> Result<()> {
    let mut settings = load_app_settings(&app)?;
    settings.theme = theme;
    save_app_settings(&app, &settings)?;
    Ok(())
}
