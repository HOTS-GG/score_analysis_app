//! 배포 양식(templates/*.json = tools/make_templates.py 가 만든 xlsx 의 셀 격자)이 파서·매핑 가져오기에 그대로 통과하는지.
use super::test_conn;
use crate::analysis::mock_files::{import_mock_parsed, parse_mock_grid};
use crate::analysis::neis::{import_parsed, parse_grid};
use crate::commands::dataset::{import_dataset_impl, suggest_mapping_impl};
use crate::types::ImportMapping;
use serde_json::{json, Map, Value};

fn grid(src: &str) -> Vec<Vec<Value>> {
    serde_json::from_str(src).expect("template json")
}

/// 격자 → {헤더: 값} 행 배열 (프론트 ImportSection 이 보내는 형태). header_row 는 0-based.
fn object_rows(g: &[Vec<Value>], header_row: usize) -> (Vec<String>, Vec<Map<String, Value>>) {
    let headers: Vec<String> = g[header_row].iter().enumerate().map(|(i, v)| match v { Value::Null => format!("열{}", i + 1), Value::String(s) => s.trim().to_string(), o => o.to_string() }).collect();
    let rows = g[header_row + 1..]
        .iter()
        .filter(|r| r.iter().any(|v| !v.is_null()))
        .map(|r| headers.iter().enumerate().map(|(i, h)| (h.clone(), r.get(i).cloned().unwrap_or(Value::Null))).collect())
        .collect();
    (headers, rows)
}

#[test]
fn template_01_neis() {
    let p = parse_grid(&grid(include_str!("../../../templates/neis.json"))).unwrap();
    assert_eq!((p.grade, p.class_no, p.students), (2, 1, 3));
    assert_eq!(p.rows.len(), 3 * (12 + 4));
    assert!(p.warnings.is_empty(), "{:?}", p.warnings);
    let mut conn = test_conn();
    let r = import_parsed(&mut conn, &p, 2026, "01.xlsx", true).unwrap();
    assert_eq!((r.inserted, r.skipped), (48, 0));
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM Student WHERE cohort = 2025 AND grade = 2", [], |x| x.get(0)).unwrap();
    assert_eq!(n, 3);
    let three: i64 = conn.query_row("SELECT COUNT(*) FROM SchoolScore WHERE ach_scale = 3", [], |x| x.get(0)).unwrap();
    assert_eq!(three, 12);
}

#[test]
fn template_02_analysis_and_03_raw() {
    let p = parse_mock_grid(&grid(include_str!("../../../templates/mock_analysis.json"))).unwrap();
    assert_eq!((p.layout.as_str(), p.grade, p.round, p.students), ("analysis", Some(1), Some(6), 3));
    let mut subs = p.subjects.clone();
    subs.sort();
    let mut want = vec!["국어", "수학", "영어", "통합사회", "통합과학", "한국사"];
    want.sort();
    assert_eq!(subs, want);
    assert!(p.rows.iter().filter(|r| r.number == 3).all(|r| r.absent), "미응시 학생은 결측");
    let kor = p.rows.iter().find(|r| r.number == 1 && r.subject == "국어").unwrap();
    assert_eq!((kor.std, kor.pct, kor.grade9), (Some(123.0), Some(85.83), Some(3)));

    let raw = parse_mock_grid(&grid(include_str!("../../../templates/mock_raw.json"))).unwrap();
    assert_eq!((raw.layout.as_str(), raw.students), ("raw", 3));
    assert_eq!(raw.rows.iter().find(|r| r.number == 1 && r.subject == "통합과학").unwrap().raw, Some(47.0));

    let mut conn = test_conn();
    let r = import_mock_parsed(&mut conn, &p, 2026, 6, "02.xlsx", true).unwrap();
    assert_eq!(r.skipped, 0);
    let r2 = import_mock_parsed(&mut conn, &raw, 2026, 9, "03.xlsx", false).unwrap();
    assert_eq!((r2.skipped, r2.unmatched.len()), (0, 0), "같은 학번이므로 명단에서 찾아야 함");
    let rounds: i64 = conn.query_row("SELECT COUNT(DISTINCT round) FROM MockScore", [], |x| x.get(0)).unwrap();
    assert_eq!(rounds, 2);
}

fn ctx(pairs: &[(&str, Value)]) -> std::collections::HashMap<String, Value> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

#[test]
fn template_04_roster_05_entry_06_homeroom_via_mapping() {
    let mut conn = test_conn();
    // 04 명단
    let (h, rows) = object_rows(&grid(include_str!("../../../templates/roster.json")), 0);
    let mut m = suggest_mapping_impl("students", &h).unwrap();
    assert_eq!(m.keys["class_no"].as_deref(), Some("반"));
    assert_eq!(m.keys["name"].as_deref(), Some("이름"));
    m.context = ctx(&[("cohort", json!(2026)), ("grade", json!(1))]);
    let r = import_dataset_impl(&mut conn, "students", &rows, &m, "04.xlsx").unwrap();
    assert_eq!((r.inserted, r.skipped), (4, 0));

    // 05 입학
    let (h, rows) = object_rows(&grid(include_str!("../../../templates/entry.json")), 0);
    let mut m = suggest_mapping_impl("entry", &h).unwrap();
    assert_eq!(m.values["mid_school_score"].as_deref(), Some("고입내신 석차백분율"), "{:?}", m.values);
    assert_eq!(m.values["placement_rank"].as_deref(), Some("반배치 석차"));
    assert_eq!(m.values["placement_n"].as_deref(), Some("반배치 응시인원"));
    assert_eq!(m.subjects.iter().map(|s| s.subject.as_str()).collect::<Vec<_>>(), vec!["국어", "수학", "영어"]);
    m.context = ctx(&[("grade", json!(1))]);
    let r = import_dataset_impl(&mut conn, "entry", &rows, &m, "05.xlsx").unwrap();
    assert_eq!((r.inserted, r.skipped), (3, 0), "{:?}", r.errors);
    let sub: i64 = conn.query_row("SELECT COUNT(*) FROM EntrySubjectScore", [], |x| x.get(0)).unwrap();
    assert_eq!(sub, 9);

    // 06 담임용 내신 (2학년 명단 필요)
    conn.execute("INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES ('S90001', 2025, 2, 5, 18, '김서준'), ('S90002', 2025, 2, 5, 19, '이하윤')", []).unwrap();
    let (h, rows) = object_rows(&grid(include_str!("../../../templates/homeroom.json")), 0);
    let mut m = suggest_mapping_impl("school", &h).unwrap();
    assert_eq!(m.mode, "long");
    m.context = ctx(&[("grade_year", json!(2026)), ("grade", json!(2)), ("semester", json!(1))]);
    let r = import_dataset_impl(&mut conn, "school", &rows, &m, "06.xlsx").unwrap();
    assert_eq!((r.inserted, r.skipped), (4, 0), "{:?}", r.errors);
    let sd: Option<f64> = conn.query_row("SELECT subject_sd FROM SchoolScore WHERE subject = '문학' AND student_id = (SELECT id FROM Student WHERE anon_id='S90001')", [], |x| x.get(0)).unwrap();
    assert_eq!(sd, Some(17.8));
    let _: ImportMapping = m;
}
