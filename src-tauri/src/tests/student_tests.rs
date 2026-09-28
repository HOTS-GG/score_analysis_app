use super::test_conn;
use crate::commands::student::*;
use crate::types::StudentInput;

fn s(cohort: i64, grade: i64, class_no: i64, number: i64, name: &str) -> StudentInput {
    StudentInput { cohort, grade, class_no, number, name: name.into() }
}

#[test]
fn create_assigns_anon_id_and_rejects_duplicates() {
    let conn = test_conn();
    let a = create_student_impl(&conn, &s(2026, 1, 1, 1, "가")).unwrap();
    assert_eq!(a.anon_id, "S00001");
    let b = create_student_impl(&conn, &s(2026, 1, 1, 2, "나")).unwrap();
    assert_eq!(b.anon_id, "S00002");
    let dup = create_student_impl(&conn, &s(2026, 1, 1, 1, "다"));
    assert!(dup.is_err());
    assert_eq!(get_students_impl(&conn).unwrap().len(), 2);
}

#[test]
fn bulk_upsert_inserts_then_updates() {
    let mut conn = test_conn();
    let r = bulk_upsert_students_impl(&mut conn, &[s(2026, 1, 1, 1, "가"), s(2026, 1, 1, 2, "나")]).unwrap();
    assert_eq!((r.inserted, r.updated), (2, 0));
    let r = bulk_upsert_students_impl(&mut conn, &[s(2026, 1, 1, 1, "가나"), s(2026, 1, 2, 1, "다")]).unwrap();
    assert_eq!((r.inserted, r.updated), (1, 1));
    let all = get_students_impl(&conn).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all.iter().find(|x| x.number == 1 && x.class_no == 1).unwrap().name, "가나");
}

#[test]
fn exclusion_requires_reason_and_logs() {
    let conn = test_conn();
    let a = create_student_impl(&conn, &s(2026, 1, 1, 1, "가")).unwrap();
    assert!(set_student_status_impl(&conn, a.id, "excluded", None).is_err());
    set_student_status_impl(&conn, a.id, "excluded", Some("전학")).unwrap();
    let st = get_students_impl(&conn).unwrap();
    assert_eq!(st[0].status, "excluded");
    assert_eq!(st[0].exclude_reason.as_deref(), Some("전학"));
    set_student_status_impl(&conn, a.id, "active", None).unwrap();
    let log = get_exclusion_log_impl(&conn).unwrap();
    assert_eq!(log.len(), 2);
    assert_eq!(log[0].action, "restore");
    assert_eq!(log[1].action, "exclude");
}

#[test]
fn invalid_input_rejected() {
    let conn = test_conn();
    assert!(create_student_impl(&conn, &s(2026, 4, 1, 1, "x")).is_err());
    assert!(create_student_impl(&conn, &s(2026, 1, 0, 1, "x")).is_err());
    assert!(create_student_impl(&conn, &s(1999, 1, 1, 1, "x")).is_err());
}
