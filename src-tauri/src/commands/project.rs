use crate::commands::config::set_config_impl;
use crate::engine::{validate_existing_path, validate_parent_dir_path};
use crate::state::{DbPathState, DbState};
use rusqlite::Connection;
use tauri::State;

pub(crate) fn new_project_impl(
    path: &str,
    current_version: &str,
    state: &DbState,
    db_path_state: &DbPathState,
) -> Result<(), String> {
    validate_parent_dir_path(path, "디렉토리가 존재하지 않습니다.")?;
    let p = std::path::Path::new(path);
    if p.exists() {
        return Err(format!("이미 파일이 존재합니다: {path}"));
    }
    let conn = crate::db::create_new(p).map_err(|e| e.to_string())?;
    set_config_impl(&conn, "app_version", current_version)?;
    *state.0.lock().map_err(|e| e.to_string())? = Some(conn);
    *db_path_state.0.lock().map_err(|e| e.to_string())? = Some(p.to_path_buf());
    Ok(())
}

pub(crate) fn open_project_impl(
    path: &str,
    state: &DbState,
    db_path_state: &DbPathState,
) -> Result<(), String> {
    validate_existing_path(path, "파일이 존재하지 않거나 접근할 수 없습니다.")?;
    let src = std::path::Path::new(path);
    let conn = crate::db::open_existing(src)?;
    *state.0.lock().map_err(|e| e.to_string())? = Some(conn);
    *db_path_state.0.lock().map_err(|e| e.to_string())? = Some(src.to_path_buf());
    Ok(())
}

pub(crate) fn backup_project_impl(db_path_state: &DbPathState) -> Result<(), String> {
    let guard = db_path_state.0.lock().map_err(|e| e.to_string())?;
    let src = guard.as_ref().ok_or("DB path not set")?;
    if let Some(parent) = src.parent() {
        let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("backup");
        let bak_path = parent.join(format!("{stem}.db.backup"));
        crate::db::dump_to_sql_file(src, &bak_path)?;
    }
    Ok(())
}

pub fn migrate_schema_impl(conn: &mut Connection) -> Result<(), String> {
    let from: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if from < crate::db::SCHEMA_VERSION {
        crate::db::migrate(conn, from).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn new_project(
    path: String,
    app: tauri::AppHandle,
    state: State<DbState>,
    db_path: State<DbPathState>,
) -> Result<(), String> {
    let version = app.package_info().version.to_string();
    new_project_impl(&path, &version, &state, &db_path)
}

#[tauri::command]
pub fn open_project(
    path: String,
    state: State<DbState>,
    db_path: State<DbPathState>,
) -> Result<(), String> {
    open_project_impl(&path, &state, &db_path)
}

#[tauri::command]
pub fn backup_project(db_path: State<DbPathState>) -> Result<(), String> {
    backup_project_impl(&db_path)
}

#[tauri::command]
pub fn migrate_schema(state: State<DbState>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not open")?;
    migrate_schema_impl(conn)
}
