use crate::state::{DbState, GlobalConfigState};
use crate::thresholds::{self, Thresholds};
use rusqlite::Connection;
use tauri::State;

/// 반환값: None = 버전 동일, Some(old) = 이전 버전 (old 빈 문자열 = 기록 없음)
pub fn check_and_update_app_version_impl(
    conn: &Connection,
    current_version: &str,
) -> Result<Option<String>, String> {
    let stored = get_config_impl(conn, "app_version")?;
    if stored.as_deref() == Some(current_version) {
        return Ok(None);
    }
    set_config_impl(conn, "app_version", current_version)?;
    Ok(Some(stored.unwrap_or_default()))
}

pub fn get_config_impl(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT config_value FROM APP_CONFIGS WHERE config_key = ?1")
        .map_err(|e| e.to_string())?;
    Ok(stmt.query_row([key], |row| row.get::<_, String>(0)).ok())
}

pub fn set_config_impl(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO APP_CONFIGS (config_key, config_value) VALUES (?1, ?2)",
        [key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_config(db: State<DbState>, key: String) -> Result<Option<String>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    get_config_impl(conn, &key)
}

#[tauri::command]
pub fn set_config(db: State<DbState>, key: String, value: String) -> Result<(), String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    set_config_impl(conn, &key, &value)
}

#[tauri::command]
pub fn delete_config(db: State<DbState>, key: String) -> Result<(), String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    conn.execute("DELETE FROM APP_CONFIGS WHERE config_key = ?1", [&key])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn check_and_update_app_version(
    db: State<DbState>,
    app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    let version = app.package_info().version.to_string();
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    check_and_update_app_version_impl(conn, &version)
}

// ── 전역 설정 (config.db, 프로젝트 무관) ─────────────────────────

#[tauri::command]
pub fn get_global_config(
    global: State<GlobalConfigState>,
    key: String,
) -> Result<Option<String>, String> {
    let conn = global.0.lock().map_err(|e| e.to_string())?;
    get_config_impl(&conn, &key)
}

#[tauri::command]
pub fn set_global_config(
    global: State<GlobalConfigState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = global.0.lock().map_err(|e| e.to_string())?;
    set_config_impl(&conn, &key, &value)
}

// ── 플래그 기준값 ────────────────────────────────────────────────

#[tauri::command]
pub fn get_thresholds(db: State<DbState>) -> Result<Thresholds, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    thresholds::load(conn)
}

#[tauri::command]
pub fn set_thresholds(db: State<DbState>, thresholds: Thresholds) -> Result<Thresholds, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    thresholds::save(conn, &thresholds)?;
    Ok(thresholds)
}

#[tauri::command]
pub fn reset_thresholds(db: State<DbState>) -> Result<Thresholds, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    thresholds::reset(conn)
}
