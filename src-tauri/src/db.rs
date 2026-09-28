use rusqlite::{Connection, Result};
use std::io::Write;
use std::path::Path;

/// 현재 앱이 지원하는 스키마 버전. 스키마 변경 시 올리고 migrate() 에 단계를 추가.
/// v1: 최초 스키마 / v2: SchoolScore.rank, credits / v3: 나이스 내신 필드·편제·대입 결과·참고선
pub const SCHEMA_VERSION: u32 = 3;

const SCHEMA_SQL: &str = include_str!("schema.sql");

fn apply_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

fn has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info(\"{table}\")"))?;
    let cols = stmt.query_map([], |r| r.get::<_, String>(1))?;
    for c in cols {
        if c? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn add_column_if_missing(conn: &Connection, table: &str, column: &str, decl: &str) -> Result<()> {
    if !has_column(conn, table, column)? {
        conn.execute_batch(&format!("ALTER TABLE \"{table}\" ADD COLUMN {column} {decl};"))?;
    }
    Ok(())
}

pub fn migrate(conn: &mut Connection, from: u32) -> Result<()> {
    let mut v = from;
    while v < SCHEMA_VERSION {
        let tx = conn.transaction()?;
        match v {
            0 => {} // v0 → v1: schema.sql 이 IF NOT EXISTS 로 생성
            1 => {
                // v1 → v2: 내신 석차·단위수
                add_column_if_missing(&tx, "SchoolScore", "rank", "INTEGER")?;
                add_column_if_missing(&tx, "SchoolScore", "credits", "INTEGER")?;
            }
            2 => {
                // v2 → v3: 나이스 교과학습발달상황 필드 (새 테이블은 apply_schema 가 만든다)
                for (c, d) in [("area", "TEXT"), ("kind", "TEXT"), ("ach_scale", "INTEGER"), ("dist_a", "REAL"), ("dist_b", "REAL"),
                               ("dist_c", "REAL"), ("dist_d", "REAL"), ("dist_e", "REAL"), ("note", "TEXT")] {
                    add_column_if_missing(&tx, "SchoolScore", c, d)?;
                }
            }
            _ => {}
        }
        tx.execute_batch(&format!("PRAGMA user_version = {}", v + 1))?;
        tx.commit()?;
        v += 1;
    }
    apply_schema(conn)?;
    Ok(())
}

pub fn create_new(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    apply_schema(&conn)?;
    conn.execute_batch(&format!("PRAGMA user_version = {}", SCHEMA_VERSION))?;
    crate::analysis::univ::seed_reference(&conn)?;
    Ok(conn)
}

#[allow(dead_code)]
pub fn create_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    apply_schema(&conn)?;
    conn.execute_batch(&format!("PRAGMA user_version = {}", SCHEMA_VERSION))?;
    Ok(conn)
}

pub fn open_existing(db_path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| e.to_string())?;
    let ok: Result<i64, _> = conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get(0));
    ok.map_err(|_| "올바른 성적분석 파일이 아닙니다.".to_string())?;
    let user_version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if user_version > SCHEMA_VERSION {
        return Err(format!(
            "이 파일은 더 새로운 버전의 프로그램으로 만들어졌습니다. (파일 v{user_version}, 앱 v{SCHEMA_VERSION})"
        ));
    }
    Ok(conn)
}

pub fn open_global_config(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS APP_CONFIGS (config_key TEXT PRIMARY KEY, config_value TEXT NOT NULL);",
    )?;
    Ok(conn)
}

/// SQL 텍스트 덤프 백업 (메모장에서 한글 정상 표시)
pub fn dump_to_sql_file(src: &Path, dst: &Path) -> Result<(), String> {
    let conn = Connection::open(src).map_err(|e| e.to_string())?;
    let mut out = std::fs::File::create(dst).map_err(|e| e.to_string())?;
    writeln!(
        out,
        "-- score-analysis-app backup {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    )
    .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT name, sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
        .map_err(|e| e.to_string())?;
    let tables: Vec<(String, String)> = stmt
        .query_map([], |r| {
            Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default()))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    for (name, sql) in tables {
        writeln!(out, "{sql};").map_err(|e| e.to_string())?;
        let mut s = conn
            .prepare(&format!("SELECT * FROM \"{name}\""))
            .map_err(|e| e.to_string())?;
        let ncol = s.column_count();
        let mut rows = s.query([]).map_err(|e| e.to_string())?;
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let mut vals = Vec::with_capacity(ncol);
            for i in 0..ncol {
                let v: rusqlite::types::Value = row.get(i).map_err(|e| e.to_string())?;
                vals.push(match v {
                    rusqlite::types::Value::Null => "NULL".to_string(),
                    rusqlite::types::Value::Integer(i) => i.to_string(),
                    rusqlite::types::Value::Real(f) => f.to_string(),
                    rusqlite::types::Value::Text(t) => format!("'{}'", t.replace('\'', "''")),
                    rusqlite::types::Value::Blob(b) => format!(
                        "X'{}'",
                        b.iter().map(|x| format!("{x:02x}")).collect::<String>()
                    ),
                });
            }
            writeln!(out, "INSERT INTO \"{name}\" VALUES ({});", vals.join(", "))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
