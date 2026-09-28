use crate::state::{unique_err, DbState};
use crate::types::{BulkUpsertResult, ExclusionLogItem, StudentInput, StudentItem};
use rusqlite::{params, Connection};
use tauri::State;

const DUP_MSG: &str = "같은 입학연도·반·번호의 학생이 이미 있습니다.";

pub fn next_anon_id(conn: &Connection) -> Result<String, String> {
    let max: i64 = conn
        .query_row("SELECT COALESCE(MAX(id), 0) FROM Student", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(format!("S{:05}", max + 1))
}

fn row_to_student(r: &rusqlite::Row) -> rusqlite::Result<StudentItem> {
    Ok(StudentItem {
        id: r.get(0)?,
        anon_id: r.get(1)?,
        cohort: r.get(2)?,
        grade: r.get(3)?,
        class_no: r.get(4)?,
        number: r.get(5)?,
        name: r.get(6)?,
        status: r.get(7)?,
        exclude_reason: r.get(8)?,
    })
}

const SELECT_STUDENT: &str =
    "SELECT id, anon_id, cohort, grade, class_no, number, name, status, exclude_reason FROM Student";

pub fn get_students_impl(conn: &Connection) -> Result<Vec<StudentItem>, String> {
    let mut stmt = conn
        .prepare(&format!("{SELECT_STUDENT} ORDER BY cohort DESC, grade, class_no, number"))
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], row_to_student)
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn validate_input(s: &StudentInput) -> Result<(), String> {
    if s.grade < 1 || s.grade > 3 {
        return Err("학년은 1~3 사이여야 합니다.".into());
    }
    if s.class_no < 1 || s.number < 1 {
        return Err("반과 번호는 1 이상이어야 합니다.".into());
    }
    if s.cohort < 2000 || s.cohort > 2100 {
        return Err("입학연도가 올바르지 않습니다.".into());
    }
    Ok(())
}

pub fn create_student_impl(conn: &Connection, s: &StudentInput) -> Result<StudentItem, String> {
    validate_input(s)?;
    let anon = next_anon_id(conn)?;
    conn.execute(
        "INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![anon, s.cohort, s.grade, s.class_no, s.number, s.name.trim()],
    )
    .map_err(|e| unique_err(&e, DUP_MSG))?;
    let id = conn.last_insert_rowid();
    conn.query_row(&format!("{SELECT_STUDENT} WHERE id = ?1"), [id], row_to_student)
        .map_err(|e| e.to_string())
}

pub fn update_student_impl(conn: &Connection, id: i64, s: &StudentInput) -> Result<(), String> {
    validate_input(s)?;
    let n = conn
        .execute(
            "UPDATE Student SET cohort = ?1, grade = ?2, class_no = ?3, number = ?4, name = ?5 WHERE id = ?6",
            params![s.cohort, s.grade, s.class_no, s.number, s.name.trim(), id],
        )
        .map_err(|e| unique_err(&e, DUP_MSG))?;
    if n == 0 {
        return Err("학생을 찾을 수 없습니다.".into());
    }
    Ok(())
}

pub fn delete_student_impl(conn: &Connection, id: i64) -> Result<(), String> {
    let n = conn
        .execute("DELETE FROM Student WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("학생을 찾을 수 없습니다.".into());
    }
    Ok(())
}

/// (cohort, class_no, number) 기준 upsert. 이름만 갱신.
pub fn bulk_upsert_students_impl(
    conn: &mut Connection,
    students: &[StudentInput],
) -> Result<BulkUpsertResult, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut result = BulkUpsertResult::default();
    for s in students {
        validate_input(s)?;
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM Student WHERE cohort = ?1 AND class_no = ?2 AND number = ?3",
                params![s.cohort, s.class_no, s.number],
                |r| r.get(0),
            )
            .ok();
        match existing {
            Some(id) => {
                tx.execute(
                    "UPDATE Student SET grade = ?1, name = ?2 WHERE id = ?3",
                    params![s.grade, s.name.trim(), id],
                )
                .map_err(|e| e.to_string())?;
                result.updated += 1;
            }
            None => {
                let anon = next_anon_id(&tx)?;
                tx.execute(
                    "INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![anon, s.cohort, s.grade, s.class_no, s.number, s.name.trim()],
                )
                .map_err(|e| e.to_string())?;
                result.inserted += 1;
            }
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(result)
}

/// 제외(전학·자퇴 등) / 복원. 생존 편향 기록을 위해 로그를 남긴다.
pub fn set_student_status_impl(
    conn: &Connection,
    id: i64,
    status: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    if status != "active" && status != "excluded" {
        return Err("상태는 active 또는 excluded 여야 합니다.".into());
    }
    if status == "excluded" && reason.map(|r| r.trim().is_empty()).unwrap_or(true) {
        return Err("제외 사유를 입력해야 합니다.".into());
    }
    let reason_val = if status == "excluded" { reason.map(|r| r.trim().to_string()) } else { None };
    let n = conn
        .execute(
            "UPDATE Student SET status = ?1, exclude_reason = ?2 WHERE id = ?3",
            params![status, reason_val, id],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("학생을 찾을 수 없습니다.".into());
    }
    let action = if status == "excluded" { "exclude" } else { "restore" };
    conn.execute(
        "INSERT INTO ExclusionLog (student_id, action, reason) VALUES (?1, ?2, ?3)",
        params![id, action, reason.map(|r| r.trim().to_string())],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_exclusion_log_impl(conn: &Connection) -> Result<Vec<ExclusionLogItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT l.id, l.student_id, s.anon_id, l.action, l.reason, l.logged_at
             FROM ExclusionLog l JOIN Student s ON s.id = l.student_id
             ORDER BY l.id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ExclusionLogItem {
                id: r.get(0)?,
                student_id: r.get(1)?,
                anon_id: r.get(2)?,
                action: r.get(3)?,
                reason: r.get(4)?,
                logged_at: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ── Tauri commands ───────────────────────────────────────────

#[tauri::command]
pub fn get_students(db: State<DbState>) -> Result<Vec<StudentItem>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    get_students_impl(conn)
}

#[tauri::command]
pub fn create_student(db: State<DbState>, student: StudentInput) -> Result<StudentItem, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    create_student_impl(conn, &student)
}

#[tauri::command]
pub fn update_student(db: State<DbState>, id: i64, student: StudentInput) -> Result<(), String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    update_student_impl(conn, id, &student)
}

#[tauri::command]
pub fn delete_student(db: State<DbState>, id: i64) -> Result<(), String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    delete_student_impl(conn, id)
}

#[tauri::command]
pub fn bulk_upsert_students(
    db: State<DbState>,
    students: Vec<StudentInput>,
) -> Result<BulkUpsertResult, String> {
    let mut guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not open")?;
    bulk_upsert_students_impl(conn, &students)
}

#[tauri::command]
pub fn set_student_status(
    db: State<DbState>,
    id: i64,
    status: String,
    reason: Option<String>,
) -> Result<(), String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    set_student_status_impl(conn, id, &status, reason.as_deref())
}

#[tauri::command]
pub fn get_exclusion_log(db: State<DbState>) -> Result<Vec<ExclusionLogItem>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    get_exclusion_log_impl(conn)
}
