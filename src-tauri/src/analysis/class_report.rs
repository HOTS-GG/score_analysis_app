//! 학급 한눈에 + 학생별 핵심 지표.
//! 등급 평균은 표시용. 교내 위치·직전 대비·입학 대비는 석차백분율 환산값(5등급 누적 컷 구간 중앙값, 학점 가중)으로 계산한다.
use crate::analysis::data::*;
use crate::analysis::stats::round_to;
use crate::thresholds::{grade_from_top_pct, Thresholds};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Default)]
pub struct AchCount {
    pub a: i64,
    pub b: i64,
    pub c: i64,
    pub d: i64,
    pub e: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Tag {
    pub text: String,
    pub level: String, // crit | warn | ok
}

#[derive(Clone, Debug, Serialize)]
pub struct StudentMetrics {
    pub student_id: i64,
    pub anon_id: String,
    pub name: String,
    pub hakbun: String,
    pub class_no: i64,
    pub number: i64,
    /// 학점 가중 등급평균 (표시용)
    pub gpa_main: Option<f64>,
    pub gpa_all: Option<f64>,
    /// 교내 위치: 석차백분율 환산 평균 → 학년 내 순위
    pub pos_pct: Option<f64>,
    pub pos_rank: Option<i64>,
    pub pos_n: i64,
    pub grade9_equiv: Option<i64>,
    pub ach: AchCount,
    /// 레이더 축(국·수·영·사·과·기타) 등급평균 (표시용)
    pub radar: Vec<Option<f64>>,
    /// 교과별 등급평균
    pub areas: HashMap<String, Option<f64>>,
    /// 직전 학기 대비 교내 위치 변화 (양수 = 하락)
    pub pos_delta: Option<f64>,
    /// 입학 대비: 입학 상위% − 현재 상위% (양수 = 올라옴)
    pub entry_move: Option<f64>,
    pub entry_top: Option<f64>,
    /// 최근 모의고사 평균 전국 백분위(높을수록 상위)와 회차
    pub mock_pct: Option<f64>,
    pub mock_round: Option<String>,
    pub tags: Vec<Tag>,
    pub graded_n: i64,
}

/// 5등급 → 누적 컷 구간 중앙 석차백분율
pub fn grade5_mid_pct(g: i64, cuts: &[f64]) -> Option<f64> {
    if g < 1 || g as usize > cuts.len() {
        return None;
    }
    let prev = if g > 1 { cuts[(g - 2) as usize] } else { 0.0 };
    Some((prev + cuts[(g - 1) as usize]) / 2.0)
}

fn weighted(iter: impl Iterator<Item = (f64, f64)>) -> Option<f64> {
    let (mut w, mut s) = (0.0, 0.0);
    for (v, c) in iter {
        let c = if c > 0.0 { c } else { 1.0 };
        w += c;
        s += v * c;
    }
    if w > 0.0 { Some(s / w) } else { None }
}

pub fn gpa(recs: &[Rec], term: &str, scope: &str) -> Option<f64> {
    weighted(recs.iter().filter(|r| r.in_term(term) && r.in_scope(scope) && r.ach_scale == 5).filter_map(|r| r.grade5.map(|g| (g as f64, r.credits))))
}

pub fn gpa_area(recs: &[Rec], term: &str, area: &str) -> Option<f64> {
    weighted(recs.iter().filter(|r| r.in_term(term) && r.area == area && r.ach_scale == 5).filter_map(|r| r.grade5.map(|g| (g as f64, r.credits))))
}

fn gpa_other(recs: &[Rec], term: &str) -> Option<f64> {
    weighted(recs.iter().filter(|r| r.in_term(term) && !is_main_area(&r.area) && r.ach_scale == 5).filter_map(|r| r.grade5.map(|g| (g as f64, r.credits))))
}

/// 석차백분율 환산 평균 (교내 위치의 원자료)
pub fn pos_raw(recs: &[Rec], term: &str, scope: &str, t: &Thresholds) -> Option<f64> {
    weighted(
        recs.iter()
            .filter(|r| r.in_term(term) && r.in_scope(scope) && r.ach_scale == 5)
            .filter_map(|r| r.grade5.and_then(|g| grade5_mid_pct(g, &t.grade5_cuts)).map(|p| (p, r.credits))),
    )
}

fn mean_dev(recs: &[Rec], term: &str, scope: &str) -> f64 {
    let v: Vec<f64> = recs.iter().filter(|r| r.in_term(term) && r.in_scope(scope)).filter_map(|r| r.deviation()).collect();
    if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 }
}

pub fn ach_count(recs: &[Rec], term: &str) -> AchCount {
    let mut c = AchCount::default();
    for r in recs.iter().filter(|r| r.in_term(term)) {
        match r.achievement.as_deref() {
            Some("A") => c.a += 1,
            Some("B") => c.b += 1,
            Some("C") => c.c += 1,
            Some("D") => c.d += 1,
            Some("E") => c.e += 1,
            _ => {}
        }
    }
    c
}

pub fn radar_values(recs: &[Rec], term: &str) -> Vec<Option<f64>> {
    RADAR_AXES.iter().map(|a| if *a == "기타" { gpa_other(recs, term) } else { gpa_area(recs, term, a) }).collect()
}

pub fn radar_average(list: &[Vec<Option<f64>>]) -> Vec<Option<f64>> {
    (0..RADAR_AXES.len())
        .map(|i| {
            let v: Vec<f64> = list.iter().filter_map(|r| r.get(i).copied().flatten()).collect();
            if v.is_empty() { None } else { Some(round_to(v.iter().sum::<f64>() / v.len() as f64, 2)) }
        })
        .collect()
}

/// 학년 내 교내 위치 순위. 반환: sid → (rank, n, pos_pct)
pub fn build_positions(d: &GradeData, term: &str, scope: &str, t: &Thresholds) -> HashMap<i64, (i64, i64, f64)> {
    let mut list: Vec<(i64, f64, f64)> = d
        .students
        .iter()
        .filter_map(|s| pos_raw(d.recs_of(s.id), term, scope, t).map(|p| (s.id, p, mean_dev(d.recs_of(s.id), term, scope))))
        .collect();
    list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(b.2.partial_cmp(&a.2).unwrap()));
    let n = list.len() as i64;
    list.iter().enumerate().map(|(i, (sid, _, _))| (*sid, ((i + 1) as i64, n, round_to((i + 1) as f64 / n as f64 * 100.0, 1)))).collect()
}

/// 최근 회차 모의고사 평균 전국 백분위 (국·수·통사·통과 중 백분위 있는 과목)
pub fn latest_mock_pct(mocks: &[MockRec]) -> Option<(String, f64)> {
    let mut rounds: Vec<(i64, i64)> = mocks.iter().filter(|m| m.pct.is_some()).map(|m| (m.grade_year, m.round)).collect();
    rounds.sort();
    rounds.dedup();
    let (gy, rd) = *rounds.last()?;
    let v: Vec<f64> = mocks
        .iter()
        .filter(|m| m.grade_year == gy && m.round == rd && matches!(m.subject.as_str(), "국어" | "수학" | "통합사회" | "통합과학"))
        .filter_map(|m| m.pct)
        .collect();
    if v.is_empty() {
        None
    } else {
        Some((format!("{}월", rd), round_to(v.iter().sum::<f64>() / v.len() as f64, 1)))
    }
}

pub fn student_metrics(d: &GradeData, s: &StudentBase, term: &str, _scope: &str, positions: &HashMap<i64, (i64, i64, f64)>, prev_positions: Option<&HashMap<i64, (i64, i64, f64)>>, prev_cur_positions: Option<&HashMap<i64, (i64, i64, f64)>>, t: &Thresholds) -> StudentMetrics {
    let recs = d.recs_of(s.id);
    let pos = positions.get(&s.id);
    let pos_pct = pos.map(|p| p.2);
    let areas: HashMap<String, Option<f64>> = MAIN_AREAS.iter().map(|a| (a.to_string(), gpa_area(recs, term, a).map(|v| round_to(v, 2)))).collect();
    let pos_delta = match (prev_cur_positions, prev_positions) {
        (Some(cur), Some(prev)) => match (cur.get(&s.id), prev.get(&s.id)) {
            (Some(c), Some(p)) => Some(round_to(c.2 - p.2, 1)),
            _ => None,
        },
        _ => None,
    };
    let entry_top = d.entry.get(&s.id).and_then(|e| e.top_pct).map(|v| round_to(v, 1));
    let entry_move = match (entry_top, pos_pct) {
        (Some(e), Some(p)) => Some(round_to(e - p, 1)),
        _ => None,
    };
    let (mock_round, mock_pct) = match d.mock.get(&s.id).and_then(|m| latest_mock_pct(m)) {
        Some((r, p)) => (Some(r), Some(p)),
        None => (None, None),
    };
    let gpa_main = gpa(recs, term, "main").map(|v| round_to(v, 2));
    let ach = ach_count(recs, term);
    let mut m = StudentMetrics {
        student_id: s.id,
        anon_id: s.anon_id.clone(),
        name: s.name.clone(),
        hakbun: s.hakbun(),
        class_no: s.class_no,
        number: s.number,
        gpa_main,
        gpa_all: gpa(recs, term, "all").map(|v| round_to(v, 2)),
        pos_pct,
        pos_rank: pos.map(|p| p.0),
        pos_n: pos.map(|p| p.1).unwrap_or(0),
        grade9_equiv: pos_pct.map(|p| grade_from_top_pct(p, &t.grade9_cuts)),
        ach,
        radar: radar_values(recs, term).into_iter().map(|v| v.map(|x| round_to(x, 2))).collect(),
        areas,
        pos_delta,
        entry_move,
        entry_top,
        mock_pct,
        mock_round,
        tags: vec![],
        graded_n: recs.iter().filter(|r| r.in_term(term) && r.ach_scale == 5 && r.grade5.is_some()).count() as i64,
    };
    m.tags = watch_tags(&m, t);
    m
}

pub fn watch_tags(m: &StudentMetrics, t: &Thresholds) -> Vec<Tag> {
    let mut tags = Vec::new();
    let push = |tags: &mut Vec<Tag>, text: String, level: &str| tags.push(Tag { text, level: level.into() });
    if let Some(dlt) = m.pos_delta {
        if dlt >= t.pos_delta_warn {
            push(&mut tags, format!("직전 학기보다 교내 위치 {dlt:.0}p 하락"), "crit");
        }
    }
    if m.ach.e >= t.e_count_warn {
        push(&mut tags, format!("E {}과목", m.ach.e), "crit");
    }
    if let Some(g) = m.gpa_main {
        let mut worst: Option<(&str, f64)> = None;
        for (a, v) in &m.areas {
            if let Some(v) = v {
                if worst.map(|w| *v > w.1).unwrap_or(true) {
                    worst = Some((a, *v));
                }
            }
        }
        if let Some((a, v)) = worst {
            if m.areas.values().filter(|x| x.is_some()).count() >= 3 && v - g >= t.area_gap_warn {
                push(&mut tags, format!("{a}만 유독 낮음"), "warn");
            }
        }
    }
    if let (Some(mp), Some(pos)) = (m.mock_pct, m.pos_pct) {
        let nat_top = 100.0 - mp;
        if nat_top - pos >= t.consistency_abs_min {
            push(&mut tags, "내신 대비 모의 약함".into(), "warn");
        }
        if pos - nat_top >= t.consistency_abs_min {
            push(&mut tags, "모의 대비 내신 약함".into(), "warn");
        }
    }
    if let Some(mv) = m.entry_move {
        if mv <= -t.entry_move_warn {
            push(&mut tags, format!("입학 때보다 {:.0}p 내려감", mv.abs()), "warn");
        } else if mv >= t.entry_move_warn {
            push(&mut tags, format!("입학 때보다 {mv:.0}p 올라옴"), "ok");
        }
    }
    if let Some(dlt) = m.pos_delta {
        if dlt <= -t.pos_delta_warn {
            push(&mut tags, format!("직전 학기보다 교내 위치 {:.0}p 상승", dlt.abs()), "ok");
        }
    }
    tags
}

#[derive(Clone, Debug, Serialize)]
pub struct DistBucket {
    pub grade: i64,
    pub class_n: i64,
    pub class_pct: f64,
    pub grade_n: i64,
    pub grade_pct: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct WatchItem {
    pub student_id: i64,
    pub name: String,
    pub number: i64,
    pub gpa_main: Option<f64>,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClassReport {
    pub grade_year: i64,
    pub grade: i64,
    pub class_no: i64,
    pub term: String,
    pub term_label: String,
    pub scope: String,
    pub terms: Vec<String>,
    pub prev_term: Option<(String, String)>,
    pub class_n: i64,
    pub grade_n: i64,
    pub class_gpa: Option<f64>,
    pub grade_gpa: Option<f64>,
    pub class_pos_mean: Option<f64>,
    pub rows: Vec<StudentMetrics>,
    pub dist: Vec<DistBucket>,
    pub radar_class: Vec<Option<f64>>,
    pub radar_grade: Vec<Option<f64>>,
    pub watch: Vec<WatchItem>,
    pub notes: Vec<String>,
}

pub fn all_metrics(d: &GradeData, term: &str, scope: &str, t: &Thresholds) -> Vec<StudentMetrics> {
    let positions = build_positions(d, term, scope, t);
    let (pcur, pprev) = match d.prev_term(term) {
        Some((cur, prev)) => (Some(build_positions(d, &cur, scope, t)), Some(build_positions(d, &prev, scope, t))),
        None => (None, None),
    };
    d.students.iter().map(|s| student_metrics(d, s, term, scope, &positions, pprev.as_ref(), pcur.as_ref(), t)).collect()
}

fn mean_opt(v: impl Iterator<Item = Option<f64>>) -> Option<f64> {
    let x: Vec<f64> = v.flatten().collect();
    if x.is_empty() { None } else { Some(round_to(x.iter().sum::<f64>() / x.len() as f64, 2)) }
}

pub fn class_report(d: &GradeData, class_no: i64, term: &str, scope: &str, t: &Thresholds) -> ClassReport {
    let all = all_metrics(d, term, scope, t);
    let rows: Vec<StudentMetrics> = all.iter().filter(|m| m.class_no == class_no).cloned().collect();
    let class_gpa = mean_opt(rows.iter().map(|m| m.gpa_main));
    let grade_gpa = mean_opt(all.iter().map(|m| m.gpa_main));
    let bucket = |list: &[StudentMetrics]| -> [i64; 5] {
        let mut b = [0i64; 5];
        for m in list {
            if let Some(g) = m.gpa_main {
                let i = (g.round() as i64).clamp(1, 5) - 1;
                b[i as usize] += 1;
            }
        }
        b
    };
    let cb = bucket(&rows);
    let gb = bucket(&all);
    let cn = cb.iter().sum::<i64>().max(1) as f64;
    let gn = gb.iter().sum::<i64>().max(1) as f64;
    let dist = (0..5)
        .map(|i| DistBucket { grade: (i + 1) as i64, class_n: cb[i], class_pct: round_to(cb[i] as f64 / cn * 100.0, 1), grade_n: gb[i], grade_pct: round_to(gb[i] as f64 / gn * 100.0, 1) })
        .collect();
    let mut watch: Vec<WatchItem> = rows
        .iter()
        .map(|m| WatchItem { student_id: m.student_id, name: m.name.clone(), number: m.number, gpa_main: m.gpa_main, reasons: m.tags.iter().filter(|x| x.level != "ok").map(|x| x.text.clone()).collect() })
        .filter(|w| !w.reasons.is_empty())
        .collect();
    watch.sort_by(|a, b| b.reasons.len().cmp(&a.reasons.len()).then(b.gpa_main.unwrap_or(0.0).partial_cmp(&a.gpa_main.unwrap_or(0.0)).unwrap()));
    watch.truncate(8);
    let mut notes = vec!["등급 평균은 학점 가중 표시용 값입니다. 교내 위치는 석차백분율 환산(5등급 누적 컷 구간 중앙값)으로 계산했습니다.".to_string()];
    if rows.iter().any(|m| m.pos_n > 0 && m.pos_n < t.min_n_warn) {
        notes.push(format!("학년 인원이 {} 미만이라 교내 위치의 신뢰도가 낮습니다.", t.min_n_warn));
    }
    ClassReport {
        grade_year: d.grade_year,
        grade: d.grade,
        class_no,
        term: term.into(),
        term_label: term_label(term),
        scope: scope.into(),
        terms: d.terms.clone(),
        prev_term: d.prev_term(term),
        class_n: rows.len() as i64,
        grade_n: all.len() as i64,
        class_gpa,
        grade_gpa,
        class_pos_mean: mean_opt(rows.iter().map(|m| m.pos_pct)),
        radar_class: radar_average(&rows.iter().map(|m| m.radar.clone()).collect::<Vec<_>>()),
        radar_grade: radar_average(&all.iter().map(|m| m.radar.clone()).collect::<Vec<_>>()),
        rows,
        dist,
        watch,
        notes,
    }
}
