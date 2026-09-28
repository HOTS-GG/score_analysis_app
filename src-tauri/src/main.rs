// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis;
mod commands;
mod db;
mod engine;
mod state;
mod thresholds;
mod types;
#[cfg(test)]
mod tests;

use commands::*;
use state::{DbPathState, DbState, GlobalConfigState};
use std::sync::Mutex;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                if let Some(icon) = app.default_window_icon().cloned() {
                    let _ = window.set_icon(icon);
                }
            }
            // 앱 전역 설정 DB (%APPDATA%\kr.hs.chuncheon.score-analysis-app\config.db)
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("앱 데이터 디렉토리를 찾을 수 없습니다.");
            std::fs::create_dir_all(&app_data_dir).expect("앱 데이터 디렉토리 생성 실패");
            let global_conn = db::open_global_config(&app_data_dir.join("config.db"))
                .expect("전역 설정 DB 초기화 실패");
            app.manage(GlobalConfigState(Mutex::new(global_conn)));
            app.manage(DbState(Mutex::new(None)));
            app.manage(DbPathState(Mutex::new(None)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 프로젝트
            new_project,
            open_project,
            backup_project,
            migrate_schema,
            check_and_update_app_version,
            // 설정
            get_config,
            set_config,
            delete_config,
            get_global_config,
            set_global_config,
            get_thresholds,
            set_thresholds,
            reset_thresholds,
            // 학생
            get_students,
            create_student,
            update_student,
            delete_student,
            bulk_upsert_students,
            set_student_status,
            get_exclusion_log,
            // 데이터셋
            get_dataset_specs,
            suggest_mapping,
            import_dataset,
            clear_dataset,
            get_dataset_summary,
            get_import_logs,
            // 분석 / 출력
            get_contexts,
            get_class_report,
            get_student_bundle,
            search_student,
            get_subject_options,
            get_subject_report,
            get_school_report,
            get_class_school_reports,
            parse_neis,
            import_neis,
            parse_mock_file,
            import_mock_file,
            get_subject_catalog,
            set_subject_catalog,
            get_admission_aggregate,
            get_regional_admission,
            get_univ_reference,
            set_univ_reference,
            write_bytes_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
