//! 학생 상담 리포트 한 장에 필요한 모든 것. 잔차·회귀·확률은 넣지 않는다.
use crate::analysis::class_report::{all_metrics, gpa, gpa_area, radar_average, StudentMetrics};
use crate::analysis::data::*;
use crate::analysis::school_report::{subject_report, SchoolRow, SubjectReport};
use crate::analysis::stats::round_to;
use crate::analysis::univ::{calc_univ, grade9_continuous, rank_by, UnivCalc};
use crate::thresholds::Thresholds;
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize)]
pub struct TrendPoint {
    pub term: String,
    pub gpa: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AreaTrend {
    pub area: String,
    pub points: Vec<TrendPoint>,
    pub latest: Option<f64>,
    pub change: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MockRoundView {
    pub grade_year: i64,
    pub grade: i64,
    pub round: i64,
    pub label: String,
    /// (과목, 원점수, 표준점수, 백분위, 등급, 결시)
    pub subjects: Vec<(String, Option<f64>, Option<f64>, Option<f64>, Option<i64>, bool)>,
    pub avg_pct: Option<f64>,
    pub raw_only: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnivView {
    pub calc: UnivCalc,
    pub kangwon_top: Option<f64>,
    pub hallym_top: Option<f64>,
    pub dongguk_top: Option<f64>,
    pub grade9_cont: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StudentBundle {
    pub student: StudentBase,
    pub grade_year: i64,
    pub term: String,
    pub term_label: String,
    pub scope: String,
    pub terms: Vec<String>,
    pub metrics: StudentMetrics,
    pub radar_grade: Vec<Option<f64>>,
    pub radar_note: String,
    pub subjects: Vec<SubjectReport>,
    pub trends: Vec<AreaTrend>,
    pub mocks: Vec<MockRoundView>,
    pub in_pct: Option<f64>,
    pub mock_note: String,
    pub entry: Option<EntryInfo>,
    pub entry_note: String,
    pub univ: UnivView,
    pub memo: String,
    pub ai_payload: serde_json::Value,
    pub notes: Vec<String>,
}

fn f2(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.2}")).unwrap_or_else(|| "—".into())
}

fn josa(word: &str, with_batchim: &str, without: &str) -> &'static str {
    let last = word.chars().last().unwrap_or('가') as u32;
    let has = (0xAC00..=0xD7A3).contains(&last) && (last - 0xAC00) % 28 != 0;
    if has { Box::leak(with_batchim.to_string().into_boxed_str()) } else { Box::leak(without.to_string().into_boxed_str()) }
}

pub fn build_bundle(conn: &Connection, student_id: i64, term: &str, scope: &str, t: &Thresholds) -> Result<StudentBundle, String> {
    let (cohort, grade): (i64, i64) = conn
        .query_row("SELECT cohort, grade FROM Student WHERE id = ?1", [student_id], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|_| "학생을 찾을 수 없습니다.".to_string())?;
    let grade_year = cohort + grade - 1;
    let d = load_grade(conn, grade_year, grade)?;
    let s = d.student(student_id).cloned().ok_or("활성 학생이 아닙니다.")?;
    let all = all_metrics(&d, term, scope, t);
    let metrics = all.iter().find(|m| m.student_id == student_id).cloned().ok_or("지표 계산 실패")?;
    let recs = d.recs_of(student_id);

    // 레이더 (학년 평균) + 해설
    let radar_grade = radar_average(&all.iter().map(|m| m.radar.clone()).collect::<Vec<_>>());
    let pairs: Vec<(&str, f64, Option<f64>)> = RADAR_AXES.iter().enumerate().filter_map(|(i, a)| metrics.radar[i].map(|v| (*a, v, radar_grade[i]))).collect();
    let radar_note = if pairs.len() < 2 {
        "비교할 교과가 부족합니다.".to_string()
    } else {
        let best = pairs.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        let worst = pairs.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        let gap = worst.1 - best.1;
        let mut t1 = format!("가장 강한 교과는 {}({:.2}), 가장 약한 교과는 {}({:.2})입니다. ", best.0, best.1, worst.0, worst.1);
        t1 += if gap >= 1.5 { "둘의 차이가 커서 모양이 한쪽으로 치우쳐 있습니다." } else if gap >= 0.8 { "차이가 뚜렷한 편입니다." } else { "교과 사이 편차가 작아 고르게 나오고 있습니다." };
        let vs: Vec<(&str, f64)> = pairs.iter().filter_map(|(a, v, g)| g.map(|g| (*a, g - v))).collect();
        if let (Some(ahead), Some(behind)) = (vs.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()), vs.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())) {
            if ahead.0 != behind.0 {
                t1 += &format!(" 학년 평균과 견주면 {}{} {:.2}등급 앞서고, {}{} {:.2}등급 뒤집니다.", ahead.0, josa(ahead.0, "이", "가"), ahead.1.abs(), behind.0, josa(behind.0, "이", "가"), behind.1.abs());
            }
        }
        t1
    };

    // 과목 카드
    let mut rows: Vec<&Rec> = recs.iter().filter(|r| r.in_term(term)).collect();
    rows.sort_by(|a, b| a.term.cmp(&b.term).then((!is_main_area(&a.area)).cmp(&!is_main_area(&b.area))).then(a.subject.cmp(&b.subject)));
    let subjects: Vec<SubjectReport> = rows.iter().map(|r| subject_report(&SchoolRow::from_rec(r), t)).collect();

    // 교과별 추이
    let trends: Vec<AreaTrend> = MAIN_AREAS
        .iter()
        .map(|a| {
            let points: Vec<TrendPoint> = d.terms.iter().map(|tm| TrendPoint { term: tm.clone(), gpa: gpa_area(recs, tm, a).map(|v| round_to(v, 2)) }).collect();
            let seen: Vec<f64> = points.iter().filter_map(|p| p.gpa).collect();
            AreaTrend { area: a.to_string(), latest: seen.last().copied(), change: if seen.len() >= 2 { Some(round_to(seen[seen.len() - 1] - seen[0], 2)) } else { None }, points }
        })
        .collect();

    // 모의고사
    let mut mocks: Vec<MockRoundView> = Vec::new();
    if let Some(list) = d.mock.get(&student_id) {
        let mut keys: Vec<(i64, i64, i64)> = list.iter().map(|m| (m.grade_year, m.grade, m.round)).collect();
        keys.sort();
        keys.dedup();
        for (gy, g, rd) in keys {
            let subs: Vec<_> = list.iter().filter(|m| m.grade_year == gy && m.round == rd).map(|m| (m.subject.clone(), m.raw, m.std, m.pct, m.grade9, m.absent)).collect();
            let pcts: Vec<f64> = subs.iter().filter(|x| matches!(x.0.as_str(), "국어" | "수학" | "통합사회" | "통합과학")).filter_map(|x| x.3).collect();
            let raw_only = subs.iter().all(|x| x.3.is_none() && x.4.is_none());
            mocks.push(MockRoundView { grade_year: gy, grade: g, round: rd, label: format!("{g}학년 {rd}월"), avg_pct: if pcts.is_empty() { None } else { Some(round_to(pcts.iter().sum::<f64>() / pcts.len() as f64, 1)) }, subjects: subs, raw_only });
        }
    }
    let in_pct = metrics.pos_pct.map(|p| round_to(100.0 - p, 1));
    let mock_note = match (in_pct, metrics.mock_pct) {
        (Some(i), Some(n)) => {
            let dlt = i - n;
            if dlt.abs() < 10.0 { "내신과 모의고사 위치가 비슷합니다. 지금 방식을 유지해도 좋습니다.".to_string() }
            else if dlt > 0.0 { format!("교내 내신이 전국 모의고사보다 {:.0}p 앞섭니다. 수시 교과·종합이 유리하고, 수능 최저 충족 여부를 확인해야 합니다.", dlt) }
            else { format!("전국 모의고사가 교내 내신보다 {:.0}p 앞섭니다. 정시·논술을 함께 열어 두고 내신 관리 계획을 세워 보세요.", dlt.abs()) }
        }
        _ => "비교할 값이 부족합니다.".to_string(),
    };

    // 입학 대비
    let entry = d.entry.get(&student_id).cloned();
    let entry_note = match (&entry, metrics.entry_move) {
        (None, _) => "입학 성적이 연결되지 않은 학생입니다 (동명이인은 자동 연결하지 않습니다).".to_string(),
        (Some(_), None) => "비교할 값이 부족합니다.".to_string(),
        (Some(_), Some(mv)) => {
            if mv.abs() < 8.0 { "입학 때 위치를 그대로 지키고 있습니다.".to_string() }
            else if mv > 0.0 { format!("입학 때보다 {mv:.0}p 올라왔습니다. 무엇이 통했는지 학생이 스스로 말하게 해 보세요.") }
            else { format!("입학 때보다 {:.0}p 내려왔습니다. 어느 시점부터였는지 학기별 추이와 함께 짚어 보세요.", mv.abs()) }
        }
    };

    // 대학 산식
    let calc = calc_univ(recs, term);
    let kw = rank_by(&d, term, |u| u.kangwon);
    let hl = rank_by(&d, term, |u| u.hallym);
    let dg = rank_by(&d, term, |u| u.dongguk);
    let univ = UnivView {
        kangwon_top: kw.get(&student_id).map(|x| x.2),
        hallym_top: hl.get(&student_id).map(|x| x.2),
        dongguk_top: dg.get(&student_id).map(|x| x.2),
        grade9_cont: metrics.pos_pct.map(|p| grade9_continuous(p, &t.grade9_cuts)),
        calc,
    };

    let memo = build_memo(&s, &metrics, &d, term, &subjects, &mocks, &entry);
    let ai_payload = ai_profile(&metrics, &d, term, &subjects, &mocks, &univ);
    let mut notes = vec!["등급 평균은 표시용 단순(학점 가중) 평균입니다. 교내 위치는 석차백분율 환산으로 계산했습니다.".to_string(), "9등급 환산은 참고용이며 9등급제 시절 입결과 숫자를 그대로 맞대지 마세요.".to_string()];
    if subjects.iter().any(|x| x.rank_estimated) {
        notes.push("석차·표준편차가 없는 과목은 위치를 추정값으로 표시했습니다.".into());
    }
    Ok(StudentBundle {
        student: s,
        grade_year,
        term: term.into(),
        term_label: term_label(term),
        scope: scope.into(),
        terms: d.terms.clone(),
        metrics,
        radar_grade,
        radar_note,
        subjects,
        trends,
        mocks,
        in_pct,
        mock_note,
        entry,
        entry_note,
        univ,
        memo,
        ai_payload,
        notes,
    })
}

fn build_memo(s: &StudentBase, m: &StudentMetrics, d: &GradeData, term: &str, subjects: &[SubjectReport], mocks: &[MockRoundView], entry: &Option<EntryInfo>) -> String {
    let mut l: Vec<String> = Vec::new();
    l.push(format!("[{} {} · {}학년 {}반 {}번 · {} 기준]", s.hakbun(), s.name, s.grade, s.class_no, s.number, term_label(term)));
    l.push(String::new());
    let mut pos = format!("· 성적 위치 : 주요교과 {}등급, 전 과목 {}등급", f2(m.gpa_main), f2(m.gpa_all));
    if let (Some(rk), Some(p)) = (m.pos_rank, m.pos_pct) {
        pos += &format!(", 학년 {}명 중 {}번째(상위 {:.1}%)", m.pos_n, rk, p);
    }
    l.push(pos);
    if let Some(g9) = m.grade9_equiv {
        l.push(format!("  9등급으로 환산하면 약 {g9}등급 위치입니다. (5등급제 첫 시행이라 참고용)"));
    }
    let mut av: Vec<(&String, f64)> = m.areas.iter().filter_map(|(a, v)| v.map(|v| (a, v))).collect();
    av.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    if !av.is_empty() {
        l.push(format!("· 강한 교과 : {}", av.iter().take(2).map(|(a, v)| format!("{a}({v:.2})")).collect::<Vec<_>>().join(", ")));
        l.push(format!("· 보완할 교과 : {}", av.iter().rev().take(2).map(|(a, v)| format!("{a}({v:.2})")).collect::<Vec<_>>().join(", ")));
    }
    l.push(format!("· 성취도 : A {} / B {} / C {} / D {} / E {} 과목", m.ach.a, m.ach.b, m.ach.c, m.ach.d, m.ach.e));
    let mut lows: Vec<&SubjectReport> = subjects.iter().filter(|x| x.deviation.map(|v| v <= -12.0).unwrap_or(false)).collect();
    lows.sort_by(|a, b| a.deviation.partial_cmp(&b.deviation).unwrap());
    if !lows.is_empty() {
        l.push(format!("· 과목평균보다 크게 낮은 과목 : {}", lows.iter().take(3).map(|x| format!("{}({:.1})", x.subject, x.deviation.unwrap())).collect::<Vec<_>>().join(", ")));
    }
    let mut highs: Vec<&SubjectReport> = subjects.iter().filter(|x| x.deviation.map(|v| v >= 12.0).unwrap_or(false)).collect();
    highs.sort_by(|a, b| b.deviation.partial_cmp(&a.deviation).unwrap());
    if !highs.is_empty() {
        l.push(format!("· 과목평균보다 크게 높은 과목 : {}", highs.iter().take(3).map(|x| format!("{}(+{:.1})", x.subject, x.deviation.unwrap())).collect::<Vec<_>>().join(", ")));
    }
    if let (Some(dlt), Some((cur, prev))) = (m.pos_delta, d.prev_term(term)) {
        l.push(format!("· 학기 변화 : {} → {} 교내 위치 {}", term_label(&prev), term_label(&cur), if dlt > 0.0 { format!("{dlt:.0}p 하락") } else if dlt < 0.0 { format!("{:.0}p 상승", dlt.abs()) } else { "변화 없음".into() }));
    }
    if let (Some(e), Some(mv), Some(p)) = (entry, m.entry_move, m.pos_pct) {
        let et = e.top_pct.unwrap_or(0.0);
        l.push(format!("· 입학 대비 : 입학 상위 {et:.1}% → 현재 상위 {p:.1}% — {}", if mv.abs() < 8.0 { "위치 유지".to_string() } else if mv > 0.0 { format!("{mv:.0}p 상승") } else { format!("{:.0}p 하락", mv.abs()) }));
    }
    if let (Some(mk), Some(p)) = (mocks.iter().rev().find(|x| x.avg_pct.is_some()), m.pos_pct) {
        let nat = mk.avg_pct.unwrap();
        let inp = 100.0 - p;
        l.push(format!("· 모의고사({}) 평균 백분위 {nat:.1} vs 교내 내신 백분위 {inp:.1}", mk.label));
        l.push(format!("  → {}", if inp - nat >= 10.0 { "수시 중심 설계가 유리" } else if nat - inp >= 10.0 { "정시·논술을 함께 열어 둘 필요" } else { "수시·정시 균형 유지" }));
    }
    l.push(String::new());
    l.push("[상담에서 확인할 것]".into());
    l.push("1. 위 수치 중 학생 본인이 예상과 다르다고 느끼는 부분은?".into());
    l.push("2. 보완할 교과의 원인 — 학습량 / 시험 유형 / 수업 이해 중 무엇인가?".into());
    l.push("3. 다음 학기 선택과목이 희망 진로·전형과 맞는가?".into());
    l.push(String::new());
    l.push("[담임 의견]".into());
    l.push(String::new());
    l.join("\n")
}

/// 외부(해석 프롬프트)로 보낼 익명 프로필. 이름·학번·출신중학교는 넣지 않는다.
fn ai_profile(m: &StudentMetrics, d: &GradeData, term: &str, subjects: &[SubjectReport], mocks: &[MockRoundView], u: &UnivView) -> serde_json::Value {
    use serde_json::json;
    let recs = d.recs_of(m.student_id);
    let latest = mocks.iter().rev().find(|x| !x.raw_only);
    json!({
        "학년": d.grade,
        "산출범위": term_label(term),
        "내신": {
            "주요교과_5등급평균": m.gpa_main, "전과목_5등급평균": m.gpa_all,
            "교내_상위백분율": m.pos_pct, "학년인원": m.pos_n,
            "성취도_과목수": {"A": m.ach.a, "B": m.ach.b, "C": m.ach.c, "D": m.ach.d, "E": m.ach.e},
        },
        "교과별_등급": m.areas,
        "학기별_추이": d.terms.iter().map(|tm| json!({"학기": tm, "주요교과등급": gpa(recs, tm, "main").map(|v| round_to(v, 2))})).collect::<Vec<_>>(),
        "과목": subjects.iter().map(|x| json!({"과목": x.subject, "교과": x.area, "구분": x.kind, "학점": x.credits, "성취도": x.achievement, "석차등급": x.grade5, "원점수": x.raw, "과목평균": x.subject_mean, "수강자수": x.enrolled_n})).collect::<Vec<_>>(),
        "모의고사": latest.map(|mk| json!({"회차": mk.label, "과목": mk.subjects.iter().map(|s| json!({"과목": s.0, "백분위": s.3, "등급": s.4})).collect::<Vec<_>>()})),
        "입학대비": m.entry_move.map(|mv| json!({"입학_상위백분율": m.entry_top, "현재_상위백분율": m.pos_pct, "이동": mv})),
        "대학별_계산결과": {
            "강원대_성취도환산점수": u.calc.kangwon.map(|v| json!({"점수": v, "만점": 500, "교내상위백분율": u.kangwon_top, "반영과목수": u.calc.kangwon_n})),
            "한림대_성취도가산점": u.calc.hallym.map(|v| json!({"점수": v, "만점": 60, "교내상위백분율": u.hallym_top, "국수영사과_이수학점": u.calc.hallym_cred})),
            "경희대_A대체_등급개선": u.calc.kh_before.map(|_| json!({"개선폭": u.calc.kh_gain, "대체과목": u.calc.kh_swapped, "반영전평균": u.calc.kh_before, "반영후평균": u.calc.kh_after})),
            "동국대_상위10과목점수": u.calc.dongguk.map(|v| json!({"점수": v, "만점": 10, "교내상위백분율": u.dongguk_top, "반영과목수": u.calc.dongguk_used.len(), "후보과목수": u.calc.dongguk_total})),
        }
    })
}

/// 학생 검색 (이름·학번 일부)
pub fn search_students(conn: &Connection, q: &str, limit: usize) -> Result<Vec<StudentBase>, String> {
    let q = q.trim();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let mut stmt = conn
        .prepare("SELECT id, anon_id, name, grade, class_no, number, status FROM Student WHERE status='active' AND (name LIKE ?1 OR anon_id LIKE ?1 OR (grade || printf('%02d', class_no) || printf('%02d', number)) LIKE ?1) ORDER BY grade, class_no, number LIMIT ?2")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![format!("%{q}%"), limit as i64], |r| Ok(StudentBase { id: r.get(0)?, anon_id: r.get(1)?, name: r.get(2)?, grade: r.get(3)?, class_no: r.get(4)?, number: r.get(5)?, status: r.get(6)? }))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub fn _unused(_: HashMap<i64, i64>) {}
