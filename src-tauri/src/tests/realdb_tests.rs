//! 실제 .db 파일 점검 (기본 실행에서는 제외).
//! 사용: SCORE_DB="경로.db" cargo test realdb -- --ignored --nocapture
use crate::analysis::class_report::class_report;
use crate::analysis::data::{available_contexts, load_grade};
use crate::analysis::student_bundle::build_bundle;
use crate::analysis::univ::admission_aggregate;
use crate::commands::dataset::get_dataset_summary_impl;
use crate::commands::student::get_students_impl;
use crate::thresholds;

#[test]
#[ignore]
fn open_real_db_and_summarize() {
    let Ok(path) = std::env::var("SCORE_DB") else {
        eprintln!("SCORE_DB 환경변수가 없어 건너뜀");
        return;
    };
    let mut conn = crate::db::open_existing(std::path::Path::new(&path)).expect("open_existing");
    crate::commands::project::migrate_schema_impl(&mut conn).expect("migrate");
    let students = get_students_impl(&conn).expect("students");
    let summary = get_dataset_summary_impl(&conn).expect("summary");
    let t = thresholds::load(&conn).expect("thresholds");
    println!("students={} active={} excluded={} school_rows={}", students.len(), summary.students_active, summary.students_excluded, summary.school_rows);
    for r in &summary.mock_rounds {
        println!("  {}학년도 {}학년 {}월: N={} subjects={:?}", r.grade_year, r.grade, r.round, r.n_students, r.subjects);
    }
    for (gy, g, classes) in available_contexts(&conn).expect("contexts") {
        let d = load_grade(&conn, gy, g).expect("load_grade");
        let cr = class_report(&d, classes[0], "ALL", "main", &t);
        let with_gpa = cr.rows.iter().filter(|m| m.gpa_main.is_some()).count();
        let with_mock = cr.rows.iter().filter(|m| m.mock_pct.is_some()).count();
        println!("  {gy}학년도 {g}학년 {}반: N={} terms={:?} gpa있음={with_gpa} 모의있음={with_mock} watch={}", classes[0], cr.class_n, d.terms, cr.watch.len());
        if let Some(first) = cr.rows.first() {
            let b = build_bundle(&conn, first.student_id, "ALL", "main", &t).expect("bundle");
            println!("    첫 학생: 과목 {}개, 모의 회차 {}개, 입학 {}, 태그 {:?}", b.subjects.len(), b.mocks.len(), b.entry.is_some(), b.metrics.tags.iter().map(|x| &x.text).collect::<Vec<_>>());
        }
    }
    let agg = admission_aggregate(&conn, 3).expect("agg");
    println!("입결 집계 {}건", agg.len());
    assert!(!students.is_empty());
}
