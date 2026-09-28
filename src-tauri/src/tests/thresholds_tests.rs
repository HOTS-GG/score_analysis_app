use super::test_conn;
use crate::thresholds::*;

#[test]
fn defaults_when_not_stored() {
    let conn = test_conn();
    let t = load(&conn).unwrap();
    assert_eq!(t, Thresholds::default());
    assert_eq!(t.consecutive_drop_delta, -7.0);
    assert_eq!(t.grade5_cuts.len(), 5);
}

#[test]
fn save_and_reload_roundtrip() {
    let conn = test_conn();
    let mut t = Thresholds::default();
    t.sharp_drop_delta = -12.0;
    t.min_n_warn = 40;
    save(&conn, &t).unwrap();
    let back = load(&conn).unwrap();
    assert_eq!(back, t);
    let r = reset(&conn).unwrap();
    assert_eq!(r, Thresholds::default());
    assert_eq!(load(&conn).unwrap(), Thresholds::default());
}

#[test]
fn partial_json_fills_defaults() {
    let conn = test_conn();
    conn.execute(
        "INSERT INTO APP_CONFIGS (config_key, config_value) VALUES (?1, ?2)",
        [CONFIG_KEY, r#"{"volatility_min": 12.5}"#],
    )
    .unwrap();
    let t = load(&conn).unwrap();
    assert_eq!(t.volatility_min, 12.5);
    assert_eq!(t.subject_gap_min, 30.0);
}

#[test]
fn validation_catches_bad_values() {
    let mut t = Thresholds::default();
    t.pct_clip_low = 100.0;
    assert!(validate(&t).is_err());
    let mut t = Thresholds::default();
    t.grade9_cuts = vec![4.0, 11.0];
    assert!(validate(&t).is_err());
    let mut t = Thresholds::default();
    t.easy_item_national_rate = 20.0;
    assert!(validate(&t).is_err());
    assert!(validate(&Thresholds::default()).is_ok());
}

#[test]
fn grade_conversion_boundaries() {
    let t = Thresholds::default();
    // 9등급제: 상위 4% 이하 → 1등급, 4~11 → 2등급, 96~100 → 9등급
    assert_eq!(grade_from_top_pct(0.5, &t.grade9_cuts), 1);
    assert_eq!(grade_from_top_pct(4.0, &t.grade9_cuts), 1);
    assert_eq!(grade_from_top_pct(4.01, &t.grade9_cuts), 2);
    assert_eq!(grade_from_top_pct(99.0, &t.grade9_cuts), 9);
    // 5등급제
    assert_eq!(grade_from_top_pct(10.0, &t.grade5_cuts), 1);
    assert_eq!(grade_from_top_pct(34.0, &t.grade5_cuts), 2);
    assert_eq!(grade_from_top_pct(66.5, &t.grade5_cuts), 4);
    assert_eq!(grade_from_top_pct(100.0, &t.grade5_cuts), 5);
}
