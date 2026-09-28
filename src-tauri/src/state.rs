use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct DbState(pub Mutex<Option<Connection>>);
pub struct DbPathState(pub Mutex<Option<PathBuf>>);

/// 앱 전역 설정 DB — 프로젝트와 무관한 설정(테마 등)
pub struct GlobalConfigState(pub Mutex<Connection>);

pub fn unique_err(e: &rusqlite::Error, conflict_msg: &str) -> String {
    if e.to_string().contains("UNIQUE constraint failed") {
        conflict_msg.to_string()
    } else {
        e.to_string()
    }
}
