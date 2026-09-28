//! 나이스 파서, 학급 지표, 대학 산식, 과목 리포트, 입결 집계.
use super::test_conn;
use crate::analysis::class_report::*;
use crate::analysis::data::*;
use crate::analysis::neis::*;
use crate::analysis::student_bundle::{build_bundle, search_students};
use crate::analysis::subject_report::*;
use crate::analysis::univ::*;
use crate::thresholds::Thresholds;
use rusqlite::Connection;
use serde_json::{json, Value};

fn grid(rows: &[&[&str]]) -> Vec<Vec<Value>> {
    rows.iter().map(|r| r.iter().map(|c| if c.is_empty() { Value::Null } else { json!(*c) }).collect()).collect()
}

fn neis_sample() -> Vec<Vec<Value>> {
    grid(&[
        &["", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "2026.09.07."],
        &["", "", "", "", "학교생활기록부 교과학습발달상황"],
        &["2학년 1반"],
        &["번 호", "성 명", "학 년", "학 기", "교 과", "과 목", "학 점", "원점수/과목평균", "성취도", "성취도별 분포비율", "석차등급", "수강자수", "비 고"],
        &["1", "한가람", "1", "1", "국어", "공통국어1", "4", "64/63.8", "D", "A(3.0) B(13.6) C(24.9)D(23.8) E(34.7)", "3", "265", ""],
        &["", "", "", "", "수학", "공통수학1", "4", "40/61.0", "D", "A(3.0) B(22.3) C(32.5)D(28.3) E(14.0)", "4", "265", ""],
        &["", "", "", "2", "국어", "공통국어2", "4", "33/63.2", "E", "A(8.5) B(13.9) C(20.5)D(17.0) E(40.2)", "5", "259", ""],
        &["", "", "", "", "", "", "", "", "1", "/", "54", "", ""],
        &["번 호", "성 명", "학 년", "학 기", "교 과", "과 목", "학 점", "원점수/과목평균", "성취도", "성취도별 분포비율", "석차등급", "수강자수", "비 고"],
        &["", "", "2", "1", "국어", "문학", "4", "44/67.2", "E", "A(5.4) B(25.2) C(20.2)D(20.5) E(28.7)", "4", "258", ""],
        &["", "", "", "", "사회(역사/도덕포함)", "사회문제 탐구", "3", "100/97.3", "A", "A(93.8) B(5.0) C(0.0)D(1.3) E(0.0)", "·", "80", ""],
        &["", "", "", "", "사회(역사/도덕포함)", "국제 관계의 이해", "3", "·", "·", "·", "·", "·", "학교 밖 교육"],
        &["이수학점 합계", "", "", "", "", "", "22", "", "", "", "", "", ""],
        &["2", "윤슬기", "1", "1", "국어", "공통국어1", "4", "52/63.8", "E", "A(3.0) B(13.6) C(24.9)D(23.8) E(34.7)", "4", "265", ""],
        &["번 호", "성 명", "학 년", "학 기", "교 과", "과 목", "학 점", "성취도", "비 고"],
        &["1", "한가람", "1", "1", "과학", "과학탐구실험1", "1", "B", ""],
        &["", "", "", "", "체육", "체육1", "2", "A", ""],
        &["", "", "2", "1", "체육", "스포츠 생활1", "2", "B", ""],
        &["2", "윤슬기", "1", "1", "체육", "체육1", "2", "A", ""],
    ])
}

#[test]
fn neis_grid_is_parsed() {
    let p = parse_grid(&neis_sample()).unwrap();
    assert_eq!((p.grade, p.class_no, p.students), (2, 1, 2));
    assert_eq!(p.rows.len(), 11);
    let r0 = &p.rows[0];
    assert_eq!((r0.number, r0.name.as_str(), r0.grade, r0.semester, r0.subject.as_str()), (1, "한가람", 1, 1, "공통국어1"));
    assert_eq!((r0.raw, r0.subject_mean, r0.grade5, r0.enrolled_n, r0.ach_scale), (Some(64.0), Some(63.8), Some(3), Some(265), 5));
    assert_eq!(r0.dist, [Some(3.0), Some(13.6), Some(24.9), Some(23.8), Some(34.7)]);
    // 학기 forward-fill, 학년 변경
    assert_eq!((p.rows[2].grade, p.rows[2].semester), (1, 2));
    assert_eq!((p.rows[3].grade, p.rows[3].semester, p.rows[3].subject.as_str()), (2, 1, "문학"));
    // 성취도만 있는 과목 (석차등급 '·')
    assert_eq!((p.rows[4].grade5, p.rows[4].achievement.as_deref(), p.rows[4].area.as_str()), (None, Some("A"), "사회"));
    // 학교 밖 교육
    assert_eq!((p.rows[5].raw, p.rows[5].note.as_deref()), (None, Some("학교 밖 교육")));
    // 3단계 블록
    let three: Vec<&NeisRow> = p.rows.iter().filter(|r| r.ach_scale == 3).collect();
    assert_eq!(three.len(), 4);
    assert_eq!((three[2].grade, three[2].semester, three[2].subject.as_str()), (2, 1, "스포츠 생활1"));
    assert_eq!(three[3].number, 2);
}

#[test]
fn neis_import_and_class_report() {
    let mut conn = test_conn();
    // 2학년 명단 (2025 입학) — 3명, 1명은 파일에 없음
    for (n, name) in [(1, "한가람"), (2, "윤슬기"), (3, "없음")] {
        conn.execute("INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES (?1, 2025, 2, 1, ?2, ?3)", rusqlite::params![format!("S0000{n}"), n, name]).unwrap();
    }
    let p = parse_grid(&neis_sample()).unwrap();
    let r = import_parsed(&mut conn, &p, 2026, "download-1.xlsx", false).unwrap();
    assert_eq!((r.inserted, r.updated, r.skipped), (11, 0, 0));
    // 1학년 기록은 2025학년도로
    let gy: i64 = conn.query_row("SELECT grade_year FROM SchoolScore WHERE subject='공통국어1' AND student_id=1", [], |x| x.get(0)).unwrap();
    assert_eq!(gy, 2025);
    let r2 = import_parsed(&mut conn, &p, 2026, "download-1.xlsx", false).unwrap();
    assert_eq!((r2.inserted, r2.updated), (0, 11));

    let d = load_grade(&conn, 2026, 2).unwrap();
    assert_eq!(d.students.len(), 3);
    assert_eq!(d.terms, vec!["1-1", "1-2", "2-1"]);
    let recs = d.recs_of(1);
    assert_eq!(recs.len(), 9);
    let t = Thresholds::default();
    // 1-1: 국어 3등급(4학점), 수학 4등급(4학점) → gpa 3.5, pos = (50+78)/2 = 64
    assert_eq!(gpa(recs, "1-1", "main"), Some(3.5));
    assert_eq!(pos_raw(recs, "1-1", "main", &t), Some(64.0));
    let ac = ach_count(recs, "1-1");
    assert_eq!((ac.a, ac.b, ac.d), (1, 1, 2));

    let cr = class_report(&d, 1, "ALL", "main", &t);
    assert_eq!(cr.class_n, 3);
    let s1 = cr.rows.iter().find(|m| m.student_id == 1).unwrap();
    let s2 = cr.rows.iter().find(|m| m.student_id == 2).unwrap();
    assert!(s2.pos_pct.unwrap() > s1.pos_pct.unwrap(), "학생2(4등급)가 학생1보다 아래");
    assert_eq!(s1.pos_rank, Some(1));
    assert_eq!(s1.pos_n, 2);
    assert!(cr.rows.iter().find(|m| m.student_id == 3).unwrap().gpa_main.is_none());
    assert_eq!(cr.dist.iter().map(|b| b.class_n).sum::<i64>(), 2);
    assert!(cr.radar_grade[0].is_some());

    // 과목 리포트
    let cat = subject_catalog(&d);
    assert!(cat.iter().any(|c| c.subject == "공통국어1" && c.term == "1-1" && c.n == 2));
    let sr = subject_report(&d, "1-1", "공통국어1", None).unwrap();
    assert_eq!(sr.n, 2);
    assert_eq!(sr.dist_official[0], Some(3.0));
    assert_eq!(sr.grade_counts, [0, 0, 1, 1, 0]);
    assert_eq!(sr.students[0].raw, Some(64.0));

    // 학생 번들
    let b = build_bundle(&conn, 1, "ALL", "main", &t).unwrap();
    assert_eq!(b.subjects.len(), 9);
    assert!(b.memo.contains("한가람"));
    assert!(b.ai_payload.get("내신").is_some());
    assert!(!b.ai_payload.to_string().contains("한가람"), "AI 프로필에 이름이 들어가면 안 됨");
    assert_eq!(b.trends.len(), 5);
    let kor = b.trends.iter().find(|x| x.area == "국어").unwrap();
    assert_eq!(kor.points.iter().filter(|p| p.gpa.is_some()).count(), 3);

    let found = search_students(&conn, "20101", 10).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(search_students(&conn, "슬기", 10).unwrap()[0].id, 2);
}

fn rec(subject: &str, area: &str, kind: &str, credits: f64, ach: &str, grade5: Option<i64>) -> Rec {
    Rec { grade_year: 2026, grade: 2, semester: 1, term: "2-1".into(), subject: subject.into(), area: area.into(), kind: kind.into(), credits, raw: None, subject_mean: None, subject_sd: None, achievement: Some(ach.into()), ach_scale: if grade5.is_some() { 5 } else { 3 }, grade5, rank: None, enrolled_n: None, dist: [None; 5], note: None }
}

#[test]
fn university_formulas() {
    let recs = vec![
        rec("문학", "국어", "일반", 4.0, "A", Some(2)),
        rec("대수", "수학", "일반", 4.0, "B", Some(3)),
        rec("영어Ⅰ", "영어", "일반", 4.0, "A", Some(1)),
        rec("공통국어1", "국어", "공통", 4.0, "C", Some(3)),
        rec("세계시민과 지리", "사회", "일반", 3.0, "A", Some(2)),
        rec("정치", "사회", "진로", 3.0, "A", Some(3)),
        rec("체육1", "체육", "일반", 2.0, "A", None),
    ];
    let u = calc_univ(&recs, "ALL");
    // 강원대: (500*4 + 481*4 + 500*4 + 461*4 + 500*3 + 500*3) / 22
    let expect = (500.0 * 4.0 + 481.0 * 4.0 + 500.0 * 4.0 + 461.0 * 4.0 + 500.0 * 3.0 + 500.0 * 3.0) / 22.0;
    assert!((u.kangwon.unwrap() - expect).abs() < 0.1, "{:?}", u.kangwon);
    assert_eq!(u.kangwon_n, 6);
    // 한림대: 국수영사과 성취도 평균 (60,40,60,30,60,60)/6 = 51.7
    assert_eq!(u.hallym, Some(51.7));
    // 경희대: 탐구(세계시민과 지리 A 2등급→1), 진로(정치 A 3등급→1) 대체
    assert_eq!(u.kh_swapped, vec!["세계시민과 지리", "정치"]);
    assert!(u.kh_gain.unwrap() > 0.0);
    // 동국대: 공통 제외 5과목, 영역점수 (2-1=1→10, 3+0=3→8, 1-1=0→1→10, 2-1=1→10, 3-1=2→9)
    assert_eq!(u.dongguk_total, 5);
    assert_eq!(u.dongguk, Some((10.0 + 8.0 + 10.0 + 10.0 + 9.0) / 5.0));
    assert!((grade9_continuous(4.0, &Thresholds::default().grade9_cuts) - 1.999).abs() < 0.01);
    assert!((grade9_continuous(50.0, &Thresholds::default().grade9_cuts) - 5.5).abs() < 0.01);
}

#[test]
fn admission_aggregate_and_reference() {
    let conn = test_conn();
    for (yr, univ, at, fin, g) in [(2025, "강원대", "학생부위주(교과)", "합격", 3.2), (2025, "강원대", "학생부위주(교과)", "불합격", 4.1), (2024, "강원대", "학생부위주(교과)", "충원합격", 3.6), (2024, "한림대", "학생부위주(종합)", "불합격", 5.0)] {
        conn.execute("INSERT INTO AdmissionResult (adm_year, univ, adm_type, track, final, grade_all) VALUES (?1, ?2, ?3, '자연', ?4, ?5)", rusqlite::params![yr, univ, at, fin, g]).unwrap();
    }
    let agg = admission_aggregate(&conn, 2).unwrap();
    assert_eq!(agg.len(), 1);
    let a = &agg[0];
    assert_eq!((a.apply_n, a.pass_n, a.years.clone()), (3, 2, vec![2024, 2025]));
    assert_eq!((a.pass_min, a.pass_max, a.pass_median), (Some(3.2), Some(3.6), Some(3.4)));
    seed_reference(&conn).unwrap();
    let refs = load_reference(&conn).unwrap();
    assert_eq!(refs.len(), UNIV_SEED.len());
    update_reference(&conn, refs[0].id, Some(1.3), Some("학교 입결")).unwrap();
    assert_eq!(load_reference(&conn).unwrap()[0].cut, Some(1.3));
    seed_reference(&conn).unwrap();
    assert_eq!(load_reference(&conn).unwrap().len(), UNIV_SEED.len(), "재시드 금지");
}

#[test]
fn v2_database_migrates_to_v3() {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE Student (id INTEGER PRIMARY KEY, cohort INTEGER, grade INTEGER, class_no INTEGER, number INTEGER, name TEXT, status TEXT DEFAULT 'active', anon_id TEXT);
         CREATE TABLE SchoolScore (id INTEGER PRIMARY KEY, student_id INTEGER, grade_year INTEGER, grade INTEGER, semester INTEGER, subject TEXT,
           achievement TEXT, grade5 INTEGER, raw REAL, subject_mean REAL, subject_sd REAL, enrolled_n INTEGER, written REAL, performance REAL, rank INTEGER, credits INTEGER,
           UNIQUE (student_id, grade_year, semester, subject));
         PRAGMA user_version = 2;",
    )
    .unwrap();
    crate::commands::project::migrate_schema_impl(&mut conn).unwrap();
    assert_eq!(conn.query_row::<u32, _, _>("PRAGMA user_version", [], |r| r.get(0)).unwrap(), crate::db::SCHEMA_VERSION);
    conn.execute("INSERT INTO SchoolScore (student_id, grade_year, grade, semester, subject, area, kind, dist_a) VALUES (1, 2026, 1, 1, '국어', '국어', '공통', 3.0)", []).unwrap();
    conn.execute("INSERT INTO SubjectCatalog (subject, kind) VALUES ('국어', '공통')", []).unwrap();
}

#[test]
fn area_and_common_helpers() {
    assert_eq!(normalize_area("사회(역사/도덕포함)"), "사회");
    assert_eq!(normalize_area("기술·가정/정보"), "기술·가정/정보");
    assert_eq!(normalize_area("제2외국어/한문"), "제2외국어/한문");
    assert!(is_common_subject("통합사회1", None));
    assert!(is_common_subject("문학", Some("공통")));
    assert!(!is_common_subject("문학", Some("일반")));
    assert_eq!(grade5_mid_pct(1, &[10.0, 34.0, 66.0, 90.0, 100.0]), Some(5.0));
    assert_eq!(grade5_mid_pct(3, &[10.0, 34.0, 66.0, 90.0, 100.0]), Some(50.0));
    assert_eq!(parse_dist("A(18.9) B(16.2) C(12.5)D(12.8) E(39.6)")[3], Some(12.8));
}

#[test]
fn mock_files_are_recognized() {
    use crate::analysis::mock_files::*;
    // (A) 종합분석 시트 (6월 파일 양식)
    let g = grid(&[
        &["", "2025학년도", "", "", "6월 학력평가"],
        &[],
        &[],
        &["", "학년", "반", "번호", "학번", "이름", "한국사\n등급", "국어\n표준점수", "국어\n백분위", "국어\n등급", "수학\n표준점수", "수학\n백분위", "수학\n등급", "영어\n등급", "통합사회\n표준점수", "통사\n백분위", "통사\n등급", "국영수\n등급평균", "국수탐(표준)"],
        &["", "1", "01", "1", "10101", "서바다", "1", "106", "59.56", "4", "96", "48.89", "5", "4", "56", "65.47", "4", "4", "318"],
        &["", "1", "01", "4", "10104", "남하늘", "미응시", "", "", "", "", "", "", "", "", "", "", "", ""],
        &["", "1", "01", "29", "10129", "0", "", "", "", "", "", "", "", "", "", "", "", "", ""],
    ]);
    let p = parse_mock_grid(&g).unwrap();
    assert_eq!((p.layout.as_str(), p.grade, p.round, p.students), ("analysis", Some(1), Some(6), 2));
    assert_eq!(p.subjects, vec!["국어", "수학", "영어", "통합사회", "한국사"]);
    let kor = p.rows.iter().find(|r| r.number == 1 && r.subject == "국어").unwrap();
    assert_eq!((kor.std, kor.pct, kor.grade9, kor.absent), (Some(106.0), Some(59.56), Some(4), false));
    let ts = p.rows.iter().find(|r| r.number == 1 && r.subject == "통합사회").unwrap();
    assert_eq!(ts.pct, Some(65.47));
    assert!(p.rows.iter().filter(|r| r.number == 4).all(|r| r.absent));
    assert!(!p.rows.iter().any(|r| r.number == 29));

    // (B) 가채점 시트
    let g2 = grid(&[
        &[],
        &["", "학번", "이름", "국어", "수학", "영어", "한국사", "통사", "통과", "합계"],
        &["", "10102", "임솔", "59", "61", "72", "40", "31.5", "47", "310.5"],
        &["", "10101", "서바다", "응시는 했으나 취합중", "", "", "", "", "", ""],
        &["", "10104", "남하늘", "도움반(미응시)", "", "", "", "", "", ""],
    ]);
    let p2 = parse_mock_grid(&g2).unwrap();
    assert_eq!((p2.layout.as_str(), p2.students), ("raw", 3));
    assert_eq!(p2.rows.iter().filter(|r| r.number == 2).count(), 6);
    assert_eq!(p2.rows.iter().find(|r| r.number == 2 && r.subject == "통합과학").unwrap().raw, Some(47.0));
    assert!(p2.rows.iter().filter(|r| r.number == 4).all(|r| r.absent));
    assert!(p2.warnings.iter().any(|w| w.contains("취합중")));

    // import
    let mut conn = test_conn();
    conn.execute("INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES ('S00001', 2026, 1, 1, 1, '서바다')", []).unwrap();
    let r = import_mock_parsed(&mut conn, &p, 2026, 6, "6월.xlsx", true).unwrap();
    assert_eq!((r.inserted, r.skipped), (10, 0));
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM Student", [], |x| x.get(0)).unwrap();
    assert_eq!(n, 2, "미응시 학생도 명단에 생성");
    let r2 = import_mock_parsed(&mut conn, &p2, 2026, 9, "가채점.xlsx", false).unwrap();
    assert_eq!(r2.unmatched, vec!["1학년 1반 2번 임솔".to_string()]);
    assert!(import_mock_parsed(&mut conn, &p2, 2026, 4, "x", false).is_err());
}
