use crate::analysis::class_report::{class_report, ClassReport};
use crate::analysis::data::{available_contexts, load_catalog, load_grade, upsert_catalog, StudentBase};
use crate::analysis::mock_files::{import_mock_parsed, parse_mock_grid, MockParsed};
use crate::analysis::neis::{import_parsed, parse_grid, NeisParsed};
use crate::analysis::school_report::{build_class_reports, build_report, StudentSchoolReport};
use crate::analysis::student_bundle::{build_bundle, search_students, StudentBundle};
use crate::analysis::subject_report::{subject_catalog, subject_report, SubjectOption, SubjectReport};
use crate::analysis::univ::{admission_aggregate, load_reference, load_regional, update_reference, AdmissionAgg, RefRow, RegionalRow};
use crate::engine::validate_parent_dir_path;
use crate::state::DbState;
use crate::thresholds;
use crate::types::ImportResult;
use base64::Engine;
use serde::Serialize;
use serde_json::Value;
use tauri::State;

#[derive(Serialize)]
pub struct ContextInfo {
    pub grade_year: i64,
    pub grade: i64,
    pub classes: Vec<i64>,
    pub terms: Vec<String>,
    pub students: i64,
}

#[tauri::command]
pub fn get_contexts(db: State<DbState>) -> Result<Vec<ContextInfo>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    let mut out = Vec::new();
    for (gy, g, classes) in available_contexts(conn)? {
        let cohort = gy - g + 1;
        let mut stmt = conn
            .prepare("SELECT DISTINCT s.grade, s.semester FROM SchoolScore s JOIN Student st ON st.id = s.student_id WHERE st.cohort = ?1 ORDER BY 1, 2")
            .map_err(|e| e.to_string())?;
        let terms: Vec<String> = stmt
            .query_map([cohort], |r| Ok(format!("{}-{}", r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM Student WHERE cohort = ?1 AND status='active'", [cohort], |r| r.get(0)).map_err(|e| e.to_string())?;
        out.push(ContextInfo { grade_year: gy, grade: g, classes, terms, students: n });
    }
    Ok(out)
}

#[tauri::command]
pub fn get_class_report(db: State<DbState>, grade_year: i64, grade: i64, class_no: i64, term: String, scope: String) -> Result<ClassReport, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    let t = thresholds::load(conn)?;
    let d = load_grade(conn, grade_year, grade)?;
    Ok(class_report(&d, class_no, &term, &scope, &t))
}

#[tauri::command]
pub fn get_student_bundle(db: State<DbState>, student_id: i64, term: String, scope: String) -> Result<StudentBundle, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    let t = thresholds::load(conn)?;
    build_bundle(conn, student_id, &term, &scope, &t)
}

#[tauri::command]
pub fn search_student(db: State<DbState>, q: String) -> Result<Vec<StudentBase>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    search_students(conn, &q, 20)
}

#[tauri::command]
pub fn get_subject_options(db: State<DbState>, grade_year: i64, grade: i64) -> Result<Vec<SubjectOption>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    let d = load_grade(conn, grade_year, grade)?;
    Ok(subject_catalog(&d))
}

#[tauri::command]
pub fn get_subject_report(db: State<DbState>, grade_year: i64, grade: i64, term: String, subject: String, class_no: Option<i64>) -> Result<Option<SubjectReport>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    let d = load_grade(conn, grade_year, grade)?;
    Ok(subject_report(&d, &term, &subject, class_no))
}

// ── 내신 개인 분석 카드 (담임 양식 경로) ──
#[tauri::command]
pub fn get_school_report(db: State<DbState>, student_id: i64, grade_year: i64, semester: i64) -> Result<StudentSchoolReport, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    let t = thresholds::load(conn)?;
    build_report(conn, student_id, grade_year, semester, &t)
}

#[tauri::command]
pub fn get_class_school_reports(db: State<DbState>, grade_year: i64, semester: i64, grade: i64, class_no: i64) -> Result<Vec<StudentSchoolReport>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    let t = thresholds::load(conn)?;
    build_class_reports(conn, grade_year, semester, grade, class_no, &t)
}

// ── 나이스 교과학습발달상황 ──
#[tauri::command]
pub fn parse_neis(grid: Vec<Vec<Value>>) -> Result<NeisParsed, String> {
    parse_grid(&grid)
}

#[tauri::command]
pub fn import_neis(db: State<DbState>, grid: Vec<Vec<Value>>, grade_year: i64, file_name: String, create_missing: bool) -> Result<ImportResult, String> {
    let parsed = parse_grid(&grid)?;
    let mut guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not open")?;
    import_parsed(conn, &parsed, grade_year, &file_name, create_missing)
}

// ── 모의고사 파일 자동 인식 ──
#[tauri::command]
pub fn parse_mock_file(grid: Vec<Vec<Value>>) -> Result<MockParsed, String> {
    parse_mock_grid(&grid)
}

#[tauri::command]
pub fn import_mock_file(db: State<DbState>, grid: Vec<Vec<Value>>, grade_year: i64, round: i64, file_name: String, create_missing: bool) -> Result<ImportResult, String> {
    let parsed = parse_mock_grid(&grid)?;
    let mut guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not open")?;
    import_mock_parsed(conn, &parsed, grade_year, round, &file_name, create_missing)
}

#[tauri::command]
pub fn get_subject_catalog(db: State<DbState>) -> Result<Vec<(String, Option<String>, String)>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    load_catalog(conn)
}

#[tauri::command]
pub fn set_subject_catalog(db: State<DbState>, items: Vec<(String, Option<String>, String)>) -> Result<usize, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    let n = upsert_catalog(conn, &items)?;
    // 이미 들어온 내신 행의 kind 도 갱신
    conn.execute("UPDATE SchoolScore SET kind = (SELECT kind FROM SubjectCatalog c WHERE c.subject = SchoolScore.subject) WHERE subject IN (SELECT subject FROM SubjectCatalog)", [])
        .map_err(|e| e.to_string())?;
    Ok(n)
}

// ── 진학 ──
#[tauri::command]
pub fn get_admission_aggregate(db: State<DbState>, min_apply: i64) -> Result<Vec<AdmissionAgg>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    admission_aggregate(conn, min_apply)
}

#[tauri::command]
pub fn get_regional_admission(db: State<DbState>) -> Result<Vec<RegionalRow>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    load_regional(conn)
}

#[tauri::command]
pub fn get_univ_reference(db: State<DbState>) -> Result<Vec<RefRow>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    crate::analysis::univ::seed_reference(conn).map_err(|e| e.to_string())?;
    load_reference(conn)
}

#[tauri::command]
pub fn set_univ_reference(db: State<DbState>, id: i64, cut: Option<f64>, note: Option<String>) -> Result<(), String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    update_reference(conn, id, cut, note.as_deref())
}

/// 프론트(exceljs)가 만든 파일을 사용자가 고른 경로에 저장
#[tauri::command]
pub fn write_bytes_file(path: String, data_base64: String) -> Result<(), String> {
    validate_parent_dir_path(&path, "저장 폴더가 존재하지 않습니다.")?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_base64.as_bytes())
        .map_err(|e| format!("파일 데이터 디코딩 실패: {e}"))?;
    std::fs::write(&path, bytes).map_err(|e| format!("파일 저장 실패: {e}"))
}
