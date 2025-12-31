//! Dataset Studio - LLM 학습용 텍스트 데이터셋 제작 도구
//!
//! 이 라이브러리는 텍스트 데이터를 청크로 분할하고,
//! 리뷰/편집 후 JSONL 형식으로 내보내는 기능을 제공합니다.

pub mod commands;
pub mod core;
pub mod db;
pub mod error;
pub mod jobs;
pub mod presets;

use tauri::Manager;

/// 애플리케이션 상태
pub struct AppState {
    pub db_pool: Option<db::DbPool>,
    pub project_path: Option<std::path::PathBuf>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            db_pool: None,
            project_path: None,
        }
    }
}

/// Tauri 애플리케이션 실행
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(std::sync::Mutex::new(AppState::default()))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // 프로젝트 관리
            commands::project::create_project,
            commands::project::open_project,
            commands::project::close_project,
            commands::project::get_project_info,
            commands::project::get_recent_projects,
            commands::project::remove_recent_project,
            commands::project::get_project_settings,
            commands::project::update_project_settings,
            commands::project::get_app_theme,
            commands::project::set_app_theme,
            // 소스/문서 관리
            commands::source::add_source,
            commands::source::scan_sources,
            commands::source::list_sources,
            commands::source::remove_source,
            commands::source::rescan_source,
            commands::source::list_documents,
            commands::source::remove_document,
            commands::source::get_manual_document_text,
            commands::source::check_document_changes,
            commands::source::resolve_document_change,
            // 청크
            commands::chunk::start_chunk_job,
            commands::chunk::get_chunk_preview,
            commands::chunk::cancel_job,
            commands::chunk::merge_chunks,
            commands::chunk::split_chunk,
            // 리뷰
            commands::review::get_review_queue,
            commands::review::get_chunk_detail,
            commands::review::set_review_status,
            commands::review::save_chunk_edit,
            commands::review::search_chunks,
            commands::review::check_chunk_quality,
            commands::review::get_default_quality_rules,
            commands::review::bulk_set_review_status,
            // Export
            commands::export::list_presets,
            commands::export::start_export_job,
            commands::export::list_export_runs,
            commands::export::open_export_folder,
            commands::export::generate_upload_commands,
            commands::export::create_preset,
            commands::export::update_preset,
            commands::export::delete_preset,
            // 통계
            commands::stats::get_project_stats,
            // 매뉴얼 데이터셋
            commands::dataset::create_manual_entry,
            commands::dataset::list_manual_entries,
            commands::dataset::update_manual_entry,
            commands::dataset::set_manual_entry_status,
            commands::dataset::delete_manual_entry,
            commands::dataset::import_jsonl,
            commands::dataset::get_preset_fields,
        ])
        .setup(|app| {
            log::info!("Dataset Studio 시작");

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri 애플리케이션 실행 실패");
}
