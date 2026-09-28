//! 내신 개인 분석 카드 (학생 1명 × 학기).
//! 원칙: 위치 계산은 석차/수강자 수(석차백분율)와 z점수로 한다. 등급은 표시용이며,
//! "9등급 환산"은 5등급을 9등급으로 바꾸는 것이 아니라 석차백분율을 9등급 컷에 얹은 값이다.
use crate::analysis::class_report::grade5_mid_pct;
use crate::analysis::stats::{norm_cdf, norm_ppf, round_half_up, round_to};
use crate::thresholds::{grade_from_top_pct, Thresholds};
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Clone, Debug, Default)]
pub struct SchoolRow {
    pub subject: String,
    pub credits: Option<i64>,
    pub raw: Option<f64>,
    pub achievement: Option<String>,
    pub grade5: Option<i64>,
    pub subject_mean: Option<f64>,
    pub subject_sd: Option<f64>,
    pub enrolled_n: Option<i64>,
    pub rank: Option<i64>,
    pub area: String,
    pub kind: String,
    pub term: String,
    pub ach_scale: i64,
    pub dist: [Option<f64>; 5],
}

impl SchoolRow {
    pub fn from_rec(r: &crate::analysis::data::Rec) -> Self {
        SchoolRow {
            subject: r.subject.clone(),
            credits: if r.credits > 0.0 { Some(r.credits as i64) } else { None },
            raw: r.raw,
            achievement: r.achievement.clone(),
            grade5: r.grade5,
            subject_mean: r.subject_mean,
            subject_sd: r.subject_sd,
            enrolled_n: r.enrolled_n,
            rank: r.rank,
            area: r.area.clone(),
            kind: r.kind.clone(),
            term: r.term.clone(),
            ach_scale: r.ach_scale,
            dist: r.dist,
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct SubjectReport {
    pub subject: String,
    pub credits: Option<i64>,
    pub raw: Option<f64>,
    pub achievement: Option<String>,
    pub grade5: Option<i64>,
    /// 석차백분율을 9등급 컷에 얹은 표시용 값 (등급 과목만)
    pub grade9_equiv: Option<i64>,
    pub subject_mean: Option<f64>,
    pub subject_sd: Option<f64>,
    pub enrolled_n: Option<i64>,
    pub z: Option<f64>,
    pub rank: Option<i64>,
    /// 석차가 없어 추정한 경우 true
    pub rank_estimated: bool,
    pub top_pct: Option<f64>,
    pub relative_label: String,
    pub feedback: String,
    /// 성취도만 있는 과목(등급 없음) → 등급 평균 제외
    pub excluded_from_avg: bool,
    /// 레이더용 0~100 (클수록 상대 위치 좋음)
    pub radar: Option<f64>,
    pub area: String,
    pub kind: String,
    pub term: String,
    pub ach_scale: i64,
    pub deviation: Option<f64>,
    pub dist: [Option<f64>; 5],
}

#[derive(Serialize, Clone, Debug)]
pub struct StudentSchoolReport {
    pub student_id: i64,
    pub anon_id: String,
    pub name: String,
    pub grade: i64,
    pub class_no: i64,
    pub number: i64,
    pub grade_year: i64,
    pub semester: i64,
    pub avg_grade5: Option<f64>,
    pub avg_grade9: Option<f64>,
    pub n_graded: i64,
    pub subjects: Vec<SubjectReport>,
    pub notes: Vec<String>,
}

fn relative_label(z: Option<f64>, top_pct: Option<f64>) -> String {
    let base = match z {
        Some(z) if z >= 1.5 => "매우 우수",
        Some(z) if z >= 0.5 => "평균보다 높음",
        Some(z) if z > -0.5 => "평균권",
        Some(z) if z > -1.5 => "평균보다 낮음",
        Some(_) => "보충 필요",
        None => "정보 없음",
    };
    match top_pct {
        Some(p) => format!("{base} / 상위 {:.1}%", p),
        None => base.to_string(),
    }
}

/// 과목 1개 → 카드 한 행. 순수 함수(테스트 대상).
pub fn subject_report(r: &SchoolRow, t: &Thresholds) -> SubjectReport {
    let z = match (r.raw, r.subject_mean, r.subject_sd) {
        (Some(x), Some(m), Some(sd)) if sd > 0.0 => Some(round_to((x - m) / sd, 2)),
        _ => None,
    };
    let deviation = match (r.raw, r.subject_mean) {
        (Some(x), Some(m)) => Some(round_to(x - m, 1)),
        _ => None,
    };
    let (rank, top_pct, rank_estimated) = match (r.rank, r.enrolled_n) {
        (Some(rk), Some(n)) if n > 0 && rk >= 1 => (Some(rk), Some(round_to(rk as f64 / n as f64 * 100.0, 1)), false),
        _ => match (z, r.enrolled_n) {
            (Some(z), Some(n)) if n > 0 => {
                let p = (1.0 - norm_cdf(z)) * 100.0;
                let est = round_half_up(p / 100.0 * n as f64).max(1);
                (Some(est), Some(round_to(p, 1)), true)
            }
            (Some(z), None) => (None, Some(round_to((1.0 - norm_cdf(z)) * 100.0, 1)), true),
            _ => match r.grade5 {
                // 나이스 출력물: 석차·표준편차 없음 → 등급 누적 컷 구간 중앙값으로 위치 추정
                Some(g) if (1..=t.grade5_cuts.len() as i64).contains(&g) => {
                    let p = grade5_mid_pct(g, &t.grade5_cuts).unwrap();
                    let est = r.enrolled_n.filter(|n| *n > 0).map(|n| round_half_up(p / 100.0 * n as f64).max(1));
                    (est, Some(round_to(p, 1)), true)
                }
                _ => (None, None, false),
            },
        },
    };
    let is_graded = r.grade5.is_some();
    let has_position = r.rank.is_some() || z.is_some();
    let grade9_equiv = if is_graded { top_pct.map(|p| grade_from_top_pct(p, &t.grade9_cuts)) } else { None };

    let mut parts: Vec<String> = Vec::new();
    if !is_graded {
        parts.push(if r.ach_scale == 3 {
            "3단계 성취도(A~C) 과목이므로 등급 평균에서 제외했습니다.".into()
        } else if r.achievement.as_deref() == Some("P") {
            "이수(P) 과목입니다.".into()
        } else {
            "성취도(A~E)만 있는 과목이므로 등급 평균에서 제외했습니다.".into()
        });
        if let Some(p) = top_pct {
            parts.push(format!("수강자 중 상위 {p:.1}% 부근입니다."));
        }
    } else if !has_position {
        // 나이스 출력물 경로: 원점수·과목평균·성취도 분포로 설명
        let g = r.grade5.unwrap();
        if let Some(dv) = deviation {
            parts.push(if dv >= 0.0 { format!("과목평균보다 {dv:.1}점 높습니다.") } else { format!("과목평균보다 {:.1}점 낮습니다.", dv.abs()) });
        }
        if let (Some(a), Some(ach)) = (r.dist[0], r.achievement.as_deref()) {
            let idx = "ABCDE".find(ach).unwrap_or(4);
            let above: f64 = r.dist[..idx].iter().flatten().sum();
            parts.push(format!("성취도 {ach} — 수강자 중 A 비율 {a:.1}%, 이 학생보다 높은 성취도 {above:.1}%."));
        }
        parts.push(format!("현재 {g}등급 (석차·표준편차가 없어 등급 안 세부 위치는 추정하지 않습니다)."));
    } else {
        let g = r.grade5.unwrap().clamp(1, t.grade5_cuts.len() as i64);
        let cum_prev = if g > 1 { t.grade5_cuts[(g - 2) as usize] } else { 0.0 };
        let cum_g = t.grade5_cuts[(g - 1) as usize];
        match (r.enrolled_n, rank) {
            (Some(n), Some(rk)) if n > 0 => {
                // 등급 경계 인원: 나이스 산출과 같은 반올림(0.5는 짝수로) — 45명의 10% = 4.5 → 4명
                let start = (cum_prev / 100.0 * n as f64).round_ties_even() as i64;
                let end = (cum_g / 100.0 * n as f64).round_ties_even() as i64;
                let band = (end - start).max(1);
                let pos = (rk - start).clamp(1, band);
                let remaining = band - pos;
                let lower_risk = band >= 2 && (remaining as f64) / (band as f64) <= 0.3;
                if lower_risk {
                    parts.push(format!("현재 {g}등급 하단에 있어 성적이 조금만 낮아져도 등급 하락 위험이 있습니다."));
                } else {
                    parts.push(format!("현재 {g}등급 내 {pos}/{band} 위치입니다."));
                }
                if rank_estimated {
                    parts.push("(석차가 없어 z점수로 추정한 위치입니다.)".into());
                }
                if g > 1 {
                    if let (Some(x), Some(m), Some(sd)) = (r.raw, r.subject_mean, r.subject_sd) {
                        if sd > 0.0 && cum_prev > 0.0 && cum_prev < 100.0 {
                            let cut = m + sd * norm_ppf(1.0 - cum_prev / 100.0);
                            let diff = cut - x;
                            if diff > 0.0 {
                                parts.push(format!("{:.1}점 정도 올리면 {}등급 진입권에 가까워집니다 (정규 근사 추정).", diff, g - 1));
                            } else {
                                parts.push(format!("{}등급 경계 근처에 있습니다.", g - 1));
                            }
                        }
                    }
                }
                if g == 1 {
                    parts.push("현재 1등급 구간입니다.".into());
                }
            }
            _ => {
                parts.push(format!("현재 {g}등급입니다. 석차와 수강자 수가 있으면 등급 내 위치를 계산합니다."));
            }
        }
    }

    SubjectReport {
        subject: r.subject.clone(),
        credits: r.credits,
        raw: r.raw,
        achievement: r.achievement.clone(),
        grade5: r.grade5,
        grade9_equiv,
        subject_mean: r.subject_mean,
        subject_sd: r.subject_sd,
        enrolled_n: r.enrolled_n,
        z,
        rank,
        rank_estimated,
        top_pct,
        relative_label: if has_position || !is_graded {
            relative_label(z, top_pct)
        } else {
            match top_pct {
                Some(p) => format!("{}등급 구간 / 상위 약 {p:.0}%", r.grade5.unwrap()),
                None => "정보 없음".into(),
            }
        },
        feedback: parts.join(" "),
        excluded_from_avg: !is_graded,
        radar: top_pct.map(|p| round_to(100.0 - p, 1)),
        area: r.area.clone(),
        kind: r.kind.clone(),
        term: r.term.clone(),
        ach_scale: r.ach_scale,
        deviation,
        dist: r.dist,
    }
}

fn mean_of(vals: impl Iterator<Item = i64>) -> Option<f64> {
    let v: Vec<i64> = vals.collect();
    if v.is_empty() {
        None
    } else {
        Some(round_to(v.iter().sum::<i64>() as f64 / v.len() as f64, 2))
    }
}

pub fn assemble(student: (i64, String, String, i64, i64, i64), grade_year: i64, semester: i64, rows: &[SchoolRow], t: &Thresholds) -> StudentSchoolReport {
    let subjects: Vec<SubjectReport> = rows.iter().map(|r| subject_report(r, t)).collect();
    let graded: Vec<&SubjectReport> = subjects.iter().filter(|s| !s.excluded_from_avg).collect();
    let mut notes = vec!["평균 등급은 표시용 단순 평균입니다. 위치 판단은 석차백분율과 z점수로 합니다.".to_string()];
    if subjects.iter().any(|s| s.rank_estimated) {
        notes.push("석차가 없는 과목은 추정값으로 위치를 표시했습니다.".into());
    }
    if subjects.is_empty() {
        notes.push("이 학기의 내신 데이터가 없습니다.".into());
    }
    let (id, anon_id, name, grade, class_no, number) = student;
    StudentSchoolReport {
        student_id: id,
        anon_id,
        name,
        grade,
        class_no,
        number,
        grade_year,
        semester,
        avg_grade5: mean_of(graded.iter().filter_map(|s| s.grade5)),
        avg_grade9: mean_of(graded.iter().filter_map(|s| s.grade9_equiv)),
        n_graded: graded.len() as i64,
        subjects,
        notes,
    }
}

fn load_rows(conn: &Connection, student_id: i64, grade_year: i64, semester: i64) -> Result<Vec<SchoolRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.subject, s.credits, s.raw, s.achievement, s.grade5, s.subject_mean, s.subject_sd, s.enrolled_n, s.rank,
                    COALESCE(s.area, c.area, ''), COALESCE(s.kind, c.kind, '미분류'), s.grade, s.semester,
                    COALESCE(s.ach_scale, CASE WHEN s.grade5 IS NULL THEN 3 ELSE 5 END), s.dist_a, s.dist_b, s.dist_c, s.dist_d, s.dist_e
             FROM SchoolScore s LEFT JOIN SubjectCatalog c ON c.subject = s.subject
             WHERE s.student_id = ?1 AND s.grade_year = ?2 AND s.semester = ?3 ORDER BY s.id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![student_id, grade_year, semester], |r| {
            let g: i64 = r.get(11)?;
            let sem: i64 = r.get(12)?;
            Ok(SchoolRow {
                subject: r.get(0)?,
                credits: r.get(1)?,
                raw: r.get(2)?,
                achievement: r.get(3)?,
                grade5: r.get(4)?,
                subject_mean: r.get(5)?,
                subject_sd: r.get(6)?,
                enrolled_n: r.get(7)?,
                rank: r.get(8)?,
                area: r.get(9)?,
                kind: r.get(10)?,
                term: format!("{g}-{sem}"),
                ach_scale: r.get(13)?,
                dist: [r.get(14)?, r.get(15)?, r.get(16)?, r.get(17)?, r.get(18)?],
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn load_student(conn: &Connection, id: i64) -> Result<(i64, String, String, i64, i64, i64), String> {
    conn.query_row("SELECT id, anon_id, name, grade, class_no, number FROM Student WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)))
        .map_err(|_| "학생을 찾을 수 없습니다.".to_string())
}

pub fn build_report(conn: &Connection, student_id: i64, grade_year: i64, semester: i64, t: &Thresholds) -> Result<StudentSchoolReport, String> {
    let student = load_student(conn, student_id)?;
    let rows = load_rows(conn, student_id, grade_year, semester)?;
    Ok(assemble(student, grade_year, semester, &rows, t))
}

pub fn build_class_reports(conn: &Connection, grade_year: i64, semester: i64, grade: i64, class_no: i64, t: &Thresholds) -> Result<Vec<StudentSchoolReport>, String> {
    let mut stmt = conn
        .prepare("SELECT id FROM Student WHERE grade = ?1 AND class_no = ?2 AND status = 'active' ORDER BY number")
        .map_err(|e| e.to_string())?;
    let ids: Vec<i64> = stmt.query_map(params![grade, class_no], |r| r.get(0)).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    ids.into_iter().map(|id| build_report(conn, id, grade_year, semester, t)).collect()
}
