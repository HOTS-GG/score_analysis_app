//! 과목별 성취 (교과 담당 관점): 성취도 분포, 등급 분포, 반별 비교, 학생 목록.
use crate::analysis::data::*;
use crate::analysis::stats::round_to;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct SubjectOption {
    pub term: String,
    pub subject: String,
    pub area: String,
    pub kind: String,
    pub n: i64,
    pub ach_scale: i64,
}

pub fn subject_catalog(d: &GradeData) -> Vec<SubjectOption> {
    let mut map: BTreeMap<(String, String), SubjectOption> = BTreeMap::new();
    for recs in d.recs.values() {
        for r in recs {
            let e = map.entry((r.term.clone(), r.subject.clone())).or_insert_with(|| SubjectOption { term: r.term.clone(), subject: r.subject.clone(), area: r.area.clone(), kind: r.kind.clone(), n: 0, ach_scale: r.ach_scale });
            e.n += 1;
        }
    }
    map.into_values().collect()
}

#[derive(Clone, Debug, Serialize)]
pub struct SubjectStudentRow {
    pub student_id: i64,
    pub name: String,
    pub class_no: i64,
    pub number: i64,
    pub raw: Option<f64>,
    pub deviation: Option<f64>,
    pub achievement: Option<String>,
    pub grade5: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClassCompare {
    pub class_no: i64,
    pub n: i64,
    pub raw_mean: Option<f64>,
    pub a_rate: Option<f64>,
    pub gpa: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SubjectReport {
    pub term: String,
    pub term_label: String,
    pub subject: String,
    pub area: String,
    pub kind: String,
    pub ach_scale: i64,
    pub n: i64,
    pub credits: f64,
    pub enrolled_n: Option<i64>,
    pub subject_mean: Option<f64>,
    pub raw_mean: Option<f64>,
    pub raw_sd: Option<f64>,
    /// 나이스 성취도별 분포비율 (수강자 전체 기준, 파일 값)
    pub dist_official: [Option<f64>; 5],
    /// 이 학년 학생 기준 성취도 개수
    pub ach_counts: [i64; 5],
    pub grade_counts: [i64; 5],
    pub by_class: Vec<ClassCompare>,
    pub students: Vec<SubjectStudentRow>,
}

pub fn subject_report(d: &GradeData, term: &str, subject: &str, class_filter: Option<i64>) -> Option<SubjectReport> {
    let mut rows: Vec<(StudentBase, Rec)> = Vec::new();
    for s in &d.students {
        if let Some(r) = d.recs_of(s.id).iter().find(|r| r.term == term && r.subject == subject) {
            rows.push((s.clone(), r.clone()));
        }
    }
    if rows.is_empty() {
        return None;
    }
    let first = rows[0].1.clone();
    let raws: Vec<f64> = rows.iter().filter_map(|(_, r)| r.raw).collect();
    let raw_mean = if raws.is_empty() { None } else { Some(round_to(raws.iter().sum::<f64>() / raws.len() as f64, 1)) };
    let raw_sd = if raws.len() > 1 {
        let m = raws.iter().sum::<f64>() / raws.len() as f64;
        Some(round_to((raws.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (raws.len() - 1) as f64).sqrt(), 1))
    } else {
        None
    };
    let mut ach = [0i64; 5];
    let mut gr = [0i64; 5];
    for (_, r) in &rows {
        if let Some(a) = r.achievement.as_deref() {
            if let Some(i) = "ABCDE".find(a) {
                ach[i] += 1;
            }
        }
        if let Some(g) = r.grade5 {
            if (1..=5).contains(&g) {
                gr[(g - 1) as usize] += 1;
            }
        }
    }
    let mut classes: BTreeMap<i64, Vec<&Rec>> = BTreeMap::new();
    for (s, r) in &rows {
        classes.entry(s.class_no).or_default().push(r);
    }
    let by_class = classes
        .into_iter()
        .map(|(c, list)| {
            let rr: Vec<f64> = list.iter().filter_map(|r| r.raw).collect();
            let a = list.iter().filter(|r| r.achievement.as_deref() == Some("A")).count();
            let gs: Vec<f64> = list.iter().filter_map(|r| r.grade5.map(|g| g as f64)).collect();
            ClassCompare {
                class_no: c,
                n: list.len() as i64,
                raw_mean: if rr.is_empty() { None } else { Some(round_to(rr.iter().sum::<f64>() / rr.len() as f64, 1)) },
                a_rate: Some(round_to(a as f64 / list.len() as f64 * 100.0, 1)),
                gpa: if gs.is_empty() { None } else { Some(round_to(gs.iter().sum::<f64>() / gs.len() as f64, 2)) },
            }
        })
        .collect();
    let mut students: Vec<SubjectStudentRow> = rows
        .iter()
        .filter(|(s, _)| class_filter.map(|c| s.class_no == c).unwrap_or(true))
        .map(|(s, r)| SubjectStudentRow { student_id: s.id, name: s.name.clone(), class_no: s.class_no, number: s.number, raw: r.raw, deviation: r.deviation().map(|v| round_to(v, 1)), achievement: r.achievement.clone(), grade5: r.grade5 })
        .collect();
    students.sort_by(|a, b| b.raw.unwrap_or(-1.0).partial_cmp(&a.raw.unwrap_or(-1.0)).unwrap().then(a.class_no.cmp(&b.class_no)).then(a.number.cmp(&b.number)));
    Some(SubjectReport {
        term: term.into(),
        term_label: term_label(term),
        subject: subject.into(),
        area: first.area.clone(),
        kind: first.kind.clone(),
        ach_scale: first.ach_scale,
        n: rows.len() as i64,
        credits: first.credits,
        enrolled_n: first.enrolled_n,
        subject_mean: first.subject_mean,
        raw_mean,
        raw_sd,
        dist_official: first.dist,
        ach_counts: ach,
        grade_counts: gr,
        by_class,
        students,
    })
}
