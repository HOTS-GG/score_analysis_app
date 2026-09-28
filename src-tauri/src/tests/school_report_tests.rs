//! 화면 예시(2학년 5반 18번)의 수치를 그대로 재현하는지 검사한다.
use super::test_conn;
use crate::analysis::school_report::*;
use crate::analysis::stats::*;
use crate::commands::dataset::split_subject_credits;
use crate::thresholds::Thresholds;

fn row(subject: &str, credits: i64, raw: f64, ach: &str, g5: Option<i64>, mean: f64, sd: f64, n: i64, rank: Option<i64>) -> SchoolRow {
    SchoolRow {
        subject: subject.into(),
        credits: Some(credits),
        raw: Some(raw),
        achievement: Some(ach.into()),
        grade5: g5,
        subject_mean: Some(mean),
        subject_sd: Some(sd),
        enrolled_n: Some(n),
        rank,
        ..Default::default()
    }
}

fn sample() -> Vec<SchoolRow> {
    vec![
        row("문학", 4, 86.77, "B", Some(2), 67.2, 17.8, 258, Some(34)),
        row("대수", 4, 87.19, "A", Some(1), 54.9, 23.4, 258, Some(10)),
        row("영어Ⅰ", 4, 94.36, "A", Some(1), 50.7, 25.4, 258, Some(13)),
        row("현대사회와 윤리", 3, 96.22, "A", Some(1), 69.9, 21.9, 103, Some(7)),
        row("경제", 3, 96.01, "A", Some(1), 73.9, 17.9, 45, Some(3)),
        row("사회문제 탐구", 3, 100.0, "A", None, 97.3, 5.7, 80, Some(1)),
        row("정보", 3, 90.88, "A", Some(1), 70.4, 16.7, 258, Some(19)),
        row("일본어", 3, 97.7, "A", Some(1), 77.9, 12.7, 130, Some(5)),
        row("스포츠 생활1", 2, 92.0, "A", None, 84.8, 15.8, 258, Some(104)),
    ]
}

#[test]
fn normal_functions_are_accurate() {
    assert!((norm_ppf(0.9) - 1.2816).abs() < 1e-3);
    assert!((norm_ppf(0.5)).abs() < 1e-6);
    assert!((norm_ppf(0.975) - 1.95996).abs() < 1e-3);
    assert!((norm_cdf(1.1) - 0.8643).abs() < 1e-3);
    assert!((norm_cdf(-1.96) - 0.025).abs() < 1e-3);
    for p in [0.005, 0.04, 0.3, 0.66, 0.9, 0.995] {
        assert!((norm_cdf(norm_ppf(p)) - p).abs() < 1e-6, "roundtrip {p}");
    }
    assert_eq!(round_half_up(4.5), 5);
    assert_eq!(round_half_up(25.8), 26);
    assert_eq!(clip_pct(0.0, 0.5, 99.5), 0.5);
    assert_eq!(clip_pct(100.0, 0.5, 99.5), 99.5);
}

#[test]
fn subject_rows_match_reference_card() {
    let t = Thresholds::default();
    let rows = sample();
    let r: Vec<SubjectReport> = rows.iter().map(|x| subject_report(x, &t)).collect();

    // 문학: z 1.1, 34/258 = 13.2%, 9등급 환산 3, 2등급 내 8/62
    assert_eq!(r[0].z, Some(1.1));
    assert_eq!(r[0].top_pct, Some(13.2));
    assert_eq!(r[0].grade9_equiv, Some(3));
    assert_eq!(r[0].relative_label, "평균보다 높음 / 상위 13.2%");
    assert!(r[0].feedback.contains("현재 2등급 내 8/62 위치입니다."), "{}", r[0].feedback);
    assert!(r[0].feedback.contains("1등급 진입권"), "{}", r[0].feedback);
    // 대수: 1등급 내 10/26, 환산 1
    assert_eq!(r[1].z, Some(1.38));
    assert_eq!(r[1].grade9_equiv, Some(1));
    assert!(r[1].feedback.contains("현재 1등급 내 10/26 위치입니다."), "{}", r[1].feedback);
    assert!(r[1].feedback.contains("현재 1등급 구간입니다."));
    // 영어: 매우 우수, 5.0% → 2등급 환산
    assert_eq!(r[2].z, Some(1.72));
    assert_eq!(r[2].grade9_equiv, Some(2));
    assert!(r[2].relative_label.starts_with("매우 우수"));
    // 현대사회와 윤리: 1등급 하단 위험 (7/10)
    assert!(r[3].feedback.contains("1등급 하단"), "{}", r[3].feedback);
    assert_eq!(r[3].grade9_equiv, Some(2));
    // 경제: 3/45 → 하단
    assert!(r[4].feedback.contains("1등급 하단"), "{}", r[4].feedback);
    // 사회문제 탐구: 성취도 과목 → 제외, 평균권, 상위 1.3%
    assert!(r[5].excluded_from_avg);
    assert_eq!(r[5].grade9_equiv, None);
    assert_eq!(r[5].relative_label, "평균권 / 상위 1.3%");
    assert!(r[5].feedback.contains("등급 평균에서 제외"));
    // 정보: 19/258 → 1등급 band 26, remaining 7 → 하단
    assert!(r[6].feedback.contains("1등급 하단"), "{}", r[6].feedback);
    // 일본어: 5/13 → 위치 문장
    assert!(r[7].feedback.contains("현재 1등급 내 5/13 위치입니다."), "{}", r[7].feedback);
    assert_eq!(r[7].grade9_equiv, Some(1));
    // 스포츠: 104/258 = 40.3%
    assert_eq!(r[8].top_pct, Some(40.3));
    assert!(r[8].excluded_from_avg);
}

#[test]
fn averages_and_assembly() {
    let t = Thresholds::default();
    let rep = assemble((1, "S00001".into(), "홍길동".into(), 2, 5, 18), 2026, 1, &sample(), &t);
    assert_eq!(rep.n_graded, 7);
    assert_eq!(rep.avg_grade5, Some(1.14));
    assert_eq!(rep.avg_grade9, Some(1.86));
    assert_eq!(rep.subjects.len(), 9);
    assert!(rep.subjects[0].radar.unwrap() > 80.0);
}

#[test]
fn rank_missing_falls_back_to_z_estimate() {
    let t = Thresholds::default();
    let mut r = row("문학", 4, 86.77, "B", Some(2), 67.2, 17.8, 258, None);
    r.rank = None;
    let s = subject_report(&r, &t);
    assert!(s.rank_estimated);
    assert!(s.rank.is_some());
    assert!(s.top_pct.unwrap() > 10.0 && s.top_pct.unwrap() < 16.0, "{:?}", s.top_pct);
    assert!(s.feedback.contains("추정"));
    // 정보가 전혀 없는 과목
    let bare = SchoolRow { subject: "체육".into(), achievement: Some("A".into()), ..Default::default() };
    let s2 = subject_report(&bare, &t);
    assert_eq!(s2.relative_label, "정보 없음");
    assert!(s2.excluded_from_avg);
}

#[test]
fn report_from_db_and_class() {
    let t = Thresholds::default();
    let conn = test_conn();
    conn.execute("INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES ('S00001', 2025, 2, 5, 18, '가')", []).unwrap();
    conn.execute("INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES ('S00002', 2025, 2, 5, 19, '나')", []).unwrap();
    conn.execute(
        "INSERT INTO SchoolScore (student_id, grade_year, grade, semester, subject, achievement, grade5, raw, subject_mean, subject_sd, enrolled_n, rank, credits)
         VALUES (1, 2026, 2, 1, '문학', 'B', 2, 86.77, 67.2, 17.8, 258, 34, 4)",
        [],
    )
    .unwrap();
    let rep = build_report(&conn, 1, 2026, 1, &t).unwrap();
    assert_eq!(rep.class_no, 5);
    assert_eq!(rep.subjects.len(), 1);
    assert_eq!(rep.subjects[0].credits, Some(4));
    let class = build_class_reports(&conn, 2026, 1, 2, 5, &t).unwrap();
    assert_eq!(class.len(), 2);
    assert!(class[1].subjects.is_empty());
    assert!(class[1].notes.iter().any(|n| n.contains("데이터가 없습니다")));
    assert!(build_report(&conn, 99, 2026, 1, &t).is_err());
}

/// 담임용 입력 양식(세로형) 헤더가 자동 매핑되고, 석차·단위수까지 카드에 도달하는지
#[test]
fn homeroom_template_roundtrip() {
    use crate::commands::dataset::*;
    use serde_json::json;
    let headers: Vec<String> = ["학년", "반", "번호", "이름", "과목", "단위수", "원점수", "성취도", "5등급", "과목평균", "표준편차", "수강자수", "석차"]
        .iter().map(|s| s.to_string()).collect();
    let mut m = suggest_mapping_impl("school", &headers).unwrap();
    assert_eq!(m.mode, "long");
    for (k, h) in [("subject", "과목"), ("credits", "단위수"), ("raw", "원점수"), ("achievement", "성취도"), ("grade5", "5등급"), ("subject_mean", "과목평균"), ("subject_sd", "표준편차"), ("enrolled_n", "수강자수"), ("rank", "석차")] {
        assert_eq!(m.values.get(k).and_then(|v| v.as_deref()), Some(h), "field {k}");
    }
    assert_eq!(m.keys["class_no"].as_deref(), Some("반"));
    m.context = [("grade_year", json!(2026)), ("grade", json!(2)), ("semester", json!(1))].into_iter().map(|(k, v)| (k.to_string(), v)).collect();

    let mut conn = test_conn();
    conn.execute("INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES ('S00001', 2025, 2, 5, 18, '가')", []).unwrap();
    let row = |pairs: &[(&str, serde_json::Value)]| -> serde_json::Map<String, serde_json::Value> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    };
    let rows = vec![
        row(&[("학년", json!(2)), ("반", json!(5)), ("번호", json!(18)), ("과목", json!("문학")), ("단위수", json!(4)), ("원점수", json!(86.77)), ("성취도", json!("B")), ("5등급", json!(2)), ("과목평균", json!(67.2)), ("표준편차", json!(17.8)), ("수강자수", json!(258)), ("석차", json!(34))]),
        row(&[("학년", json!(2)), ("반", json!(5)), ("번호", json!(18)), ("과목", json!("사회문제 탐구(3)")), ("단위수", json!("")), ("원점수", json!(100)), ("성취도", json!("A")), ("5등급", json!("")), ("과목평균", json!(97.3)), ("표준편차", json!(5.7)), ("수강자수", json!(80)), ("석차", json!(1))]),
        // 석차가 수강자 수를 넘으면 오류로 건너뜀
        row(&[("학년", json!(2)), ("반", json!(5)), ("번호", json!(18)), ("과목", json!("경제")), ("원점수", json!(90)), ("5등급", json!(1)), ("수강자수", json!(45)), ("석차", json!(50))]),
    ];
    let r = import_dataset_impl(&mut conn, "school", &rows, &m, "양식.xlsx").unwrap();
    assert_eq!((r.inserted, r.skipped), (2, 1), "{:?}", r.errors);
    let rep = build_report(&conn, 1, 2026, 1, &Thresholds::default()).unwrap();
    assert_eq!(rep.subjects.len(), 2);
    assert_eq!(rep.subjects[0].rank, Some(34));
    assert_eq!(rep.subjects[0].credits, Some(4));
    assert_eq!(rep.subjects[1].subject, "사회문제 탐구");
    assert_eq!(rep.subjects[1].credits, Some(3));
    assert!(rep.subjects[1].excluded_from_avg);
    assert_eq!(rep.avg_grade5, Some(2.0));
}

#[test]
fn subject_credit_suffix_is_parsed() {
    assert_eq!(split_subject_credits("문학(4)"), ("문학".into(), Some(4)));
    assert_eq!(split_subject_credits("스포츠 생활1(2)"), ("스포츠 생활1".into(), Some(2)));
    assert_eq!(split_subject_credits("정치와 법"), ("정치와 법".into(), None));
    assert_eq!(split_subject_credits("영어(심화)"), ("영어(심화)".into(), None));
}

#[test]
fn v1_database_migrates_to_v2() {
    let mut conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE Student (id INTEGER PRIMARY KEY);
         CREATE TABLE SchoolScore (id INTEGER PRIMARY KEY, student_id INTEGER, grade_year INTEGER, grade INTEGER, semester INTEGER, subject TEXT,
           achievement TEXT, grade5 INTEGER, raw REAL, subject_mean REAL, subject_sd REAL, enrolled_n INTEGER, written REAL, performance REAL,
           UNIQUE (student_id, grade_year, semester, subject));
         PRAGMA user_version = 1;",
    )
    .unwrap();
    crate::commands::project::migrate_schema_impl(&mut conn).unwrap();
    let v: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
    assert_eq!(v, crate::db::SCHEMA_VERSION);
    conn.execute("INSERT INTO SchoolScore (student_id, grade_year, grade, semester, subject, rank, credits) VALUES (1, 2026, 1, 1, '국어', 3, 4)", []).unwrap();
    // 두 번 실행해도 안전
    crate::commands::project::migrate_schema_impl(&mut conn).unwrap();
}
