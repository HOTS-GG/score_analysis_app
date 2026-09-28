use super::test_conn;
use crate::commands::dataset::*;
use crate::engine::parse_f64;
use crate::types::*;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

fn row(pairs: &[(&str, Value)]) -> Map<String, Value> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

fn cols(pairs: &[(&str, &str)]) -> HashMap<String, Option<String>> {
    pairs.iter().map(|(k, v)| (k.to_string(), Some(v.to_string()))).collect()
}

fn ctx(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

fn seed_students(conn: &mut rusqlite::Connection) {
    let mapping = ImportMapping {
        keys: cols(&[("class_no", "반"), ("number", "번호"), ("name", "이름")]),
        context: ctx(&[("cohort", json!(2026)), ("grade", json!(1))]),
        ..Default::default()
    };
    let rows = vec![
        row(&[("반", json!(1)), ("번호", json!(1)), ("이름", json!("가"))]),
        row(&[("반", json!(1)), ("번호", json!(2)), ("이름", json!("나"))]),
        row(&[("반", json!("2")), ("번호", json!("1")), ("이름", json!("다"))]),
    ];
    let r = import_dataset_impl(conn, "students", &rows, &mapping, "명단.xlsx").unwrap();
    assert_eq!((r.inserted, r.updated, r.skipped), (3, 0, 0));
}

#[test]
fn parse_missing_values_as_none_not_zero() {
    assert_eq!(parse_f64(""), None);
    assert_eq!(parse_f64("-"), None);
    assert_eq!(parse_f64("결시"), None);
    assert_eq!(parse_f64(" 85.5 "), Some(85.5));
    assert_eq!(parse_f64("1,234"), Some(1234.0));
    assert_eq!(parse_f64("70%"), Some(70.0));
}

#[test]
fn students_import_then_mock_wide_import() {
    let mut conn = test_conn();
    seed_students(&mut conn);

    let mapping = ImportMapping {
        keys: cols(&[("class_no", "반"), ("number", "번호")]),
        context: ctx(&[("grade_year", json!(2026)), ("grade", json!(1)), ("round", json!(3))]),
        subjects: vec![
            SubjectMapping { subject: "국어".into(), columns: cols(&[("raw", "국어 원점수"), ("pct", "국어 백분위"), ("grade9", "국어 등급")]) },
            SubjectMapping { subject: "수학".into(), columns: cols(&[("pct", "수학 백분위")]) },
        ],
        mode: "wide".into(),
        ..Default::default()
    };
    let rows = vec![
        row(&[("반", json!(1)), ("번호", json!(1)), ("국어 원점수", json!(88)), ("국어 백분위", json!(93.0)), ("국어 등급", json!(2)), ("수학 백분위", json!(70))]),
        // 수학 결시(빈칸) → 수학 행 없음, 국어만
        row(&[("반", json!(1)), ("번호", json!(2)), ("국어 원점수", json!("77")), ("국어 백분위", json!("80")), ("국어 등급", json!("3")), ("수학 백분위", json!(""))]),
        // 명단에 없는 학생 → unmatched
        row(&[("반", json!(3)), ("번호", json!(9)), ("국어 백분위", json!(50))]),
        // 백분위 범위 밖 → 오류
        row(&[("반", json!(2)), ("번호", json!(1)), ("국어 백분위", json!(120)), ("수학 백분위", json!(10))]),
    ];
    let r = import_dataset_impl(&mut conn, "mock", &rows, &mapping, "3월.xlsx").unwrap();
    assert_eq!(r.row_count, 4);
    assert_eq!(r.inserted, 4); // 1행 국어+수학, 2행 국어, 4행 수학
    assert_eq!(r.unmatched, vec!["1학년 3반 9번".to_string()]);
    assert_eq!(r.errors.len(), 1);

    let n: i64 = conn.query_row("SELECT COUNT(*) FROM MockScore", [], |x| x.get(0)).unwrap();
    assert_eq!(n, 4);
    let zero: i64 = conn.query_row("SELECT COUNT(*) FROM MockScore WHERE pct = 0", [], |x| x.get(0)).unwrap();
    assert_eq!(zero, 0, "결시를 0으로 치환하면 안 됨");

    // 재가져오기 → updated
    let r2 = import_dataset_impl(&mut conn, "mock", &rows[..1], &mapping, "3월.xlsx").unwrap();
    assert_eq!((r2.inserted, r2.updated), (0, 2));

    let sum = get_dataset_summary_impl(&conn).unwrap();
    assert_eq!(sum.students_total, 3);
    assert_eq!(sum.mock_rows, 4);
    assert_eq!(sum.mock_rounds.len(), 1);
    assert_eq!(sum.mock_rounds[0].round, 3);
    assert_eq!(sum.mock_rounds[0].n_students, 3);
    assert_eq!(get_import_logs_impl(&conn).unwrap().len(), 3);
}

#[test]
fn mock_long_format_import() {
    let mut conn = test_conn();
    seed_students(&mut conn);
    let mapping = ImportMapping {
        keys: cols(&[("class_no", "반"), ("number", "번호")]),
        context: ctx(&[("grade_year", json!(2026)), ("grade", json!(1)), ("round", json!(6))]),
        values: cols(&[("subject", "과목"), ("pct", "백분위"), ("std", "표준점수")]),
        mode: "long".into(),
        ..Default::default()
    };
    let rows = vec![
        row(&[("반", json!(1)), ("번호", json!(1)), ("과목", json!("국어")), ("백분위", json!(90)), ("표준점수", json!(131))]),
        row(&[("반", json!(1)), ("번호", json!(1)), ("과목", json!("수학")), ("백분위", json!(85)), ("표준점수", json!(128))]),
    ];
    let r = import_dataset_impl(&mut conn, "mock", &rows, &mapping, "6월.csv").unwrap();
    assert_eq!(r.inserted, 2);
    let subj: String = conn
        .query_row("SELECT GROUP_CONCAT(subject) FROM (SELECT subject FROM MockScore ORDER BY subject)", [], |x| x.get(0))
        .unwrap();
    assert_eq!(subj, "국어,수학");
}

#[test]
fn school_import_normalizes_achievement() {
    let mut conn = test_conn();
    seed_students(&mut conn);
    let mapping = ImportMapping {
        keys: cols(&[("class_no", "반"), ("number", "번호")]),
        context: ctx(&[("grade_year", json!(2026)), ("grade", json!(1)), ("semester", json!(1))]),
        subjects: vec![SubjectMapping {
            subject: "통합과학".into(),
            columns: cols(&[("achievement", "과학 성취도"), ("grade5", "과학 등급"), ("raw", "과학 원점수"), ("enrolled_n", "과학 수강자")]),
        }],
        ..Default::default()
    };
    let rows = vec![row(&[("반", json!(1)), ("번호", json!(1)), ("과학 성취도", json!("b")), ("과학 등급", json!(2)), ("과학 원점수", json!(84.5)), ("과학 수강자", json!(240))])];
    let r = import_dataset_impl(&mut conn, "school", &rows, &mapping, "내신.xlsx").unwrap();
    assert_eq!(r.inserted, 1);
    let a: String = conn.query_row("SELECT achievement FROM SchoolScore", [], |x| x.get(0)).unwrap();
    assert_eq!(a, "B");
}

#[test]
fn missing_required_key_column_is_error() {
    let mut conn = test_conn();
    let mapping = ImportMapping {
        keys: cols(&[("class_no", "반")]),
        context: ctx(&[("grade_year", json!(2026)), ("grade", json!(1)), ("round", json!(3))]),
        ..Default::default()
    };
    assert!(import_dataset_impl(&mut conn, "mock", &[], &mapping, "x").is_err());
}

#[test]
fn suggest_mapping_wide_headers() {
    let headers: Vec<String> = ["학년", "반", "번호", "성명", "국어 원점수", "국어 표준점수", "국어 백분위", "국어 등급", "수학 백분위", "영어 등급"]
        .iter().map(|s| s.to_string()).collect();
    let m = suggest_mapping_impl("mock", &headers).unwrap();
    assert_eq!(m.keys["class_no"].as_deref(), Some("반"));
    assert_eq!(m.keys["number"].as_deref(), Some("번호"));
    assert_eq!(m.keys["name"].as_deref(), Some("성명"));
    assert_eq!(m.mode, "wide");
    assert_eq!(m.subjects.len(), 3);
    let kor = &m.subjects[0];
    assert_eq!(kor.subject, "국어");
    assert_eq!(kor.columns["pct"].as_deref(), Some("국어 백분위"));
    assert_eq!(kor.columns["std"].as_deref(), Some("국어 표준점수"));
    assert_eq!(kor.columns["grade9"].as_deref(), Some("국어 등급"));
    assert_eq!(m.subjects[1].columns["pct"].as_deref(), Some("수학 백분위"));
}

#[test]
fn suggest_mapping_long_headers() {
    let headers: Vec<String> = ["반", "번호", "과목", "원점수", "백분위"].iter().map(|s| s.to_string()).collect();
    let m = suggest_mapping_impl("mock", &headers).unwrap();
    assert_eq!(m.mode, "long");
    assert_eq!(m.values["subject"].as_deref(), Some("과목"));
    assert_eq!(m.values["pct"].as_deref(), Some("백분위"));
}

#[test]
fn subject_names_are_normalized() {
    assert_eq!(normalize_subject("통사"), "통합사회");
    assert_eq!(normalize_subject(" 통과 "), "통합과학");
    assert_eq!(normalize_subject("수 학"), "수학");
    assert_eq!(normalize_subject("국어"), "국어");
    let headers: Vec<String> = ["반", "번호", "통사 백분위", "통과 등급"].iter().map(|s| s.to_string()).collect();
    let m = suggest_mapping_impl("mock", &headers).unwrap();
    assert_eq!(m.subjects.iter().map(|s| s.subject.as_str()).collect::<Vec<_>>(), vec!["통사", "통과"]);
}

#[test]
fn clear_dataset_removes_rows() {
    let mut conn = test_conn();
    seed_students(&mut conn);
    assert_eq!(clear_dataset_impl(&conn, "students").unwrap(), 3);
    assert_eq!(get_dataset_summary_impl(&conn).unwrap().students_total, 0);
    assert!(clear_dataset_impl(&conn, "nope").is_err());
}
