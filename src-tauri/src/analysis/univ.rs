//! 진학 참고: 네 대학 성취도 반영 산식(2028학년도 전형계획 기준 가이드), 참고선, 3개년 입결 집계.
//! 산식은 대시보드 초안(dashboard.js)을 그대로 옮겼다. 최종 모집요강 확인 필수.
use crate::analysis::data::*;
use crate::analysis::stats::round_to;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;

pub const DK_PTS: [f64; 9] = [10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 2.0, 0.0];

fn dk_adj(a: &str) -> Option<i64> {
    match a {
        "A" => Some(-1),
        "B" => Some(0),
        "C" => Some(1),
        "D" => Some(2),
        "E" => Some(3),
        _ => None,
    }
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct UnivCalc {
    /// 강원대: 성취도 환산점수 (학점 가중, 만점 500)
    pub kangwon: Option<f64>,
    pub kangwon_n: i64,
    pub kangwon_cnt: HashMap<String, i64>,
    /// 한림대: 국수영사과 성취도 가산점 평균 (만점 60)
    pub hallym: Option<f64>,
    pub hallym_n: i64,
    pub hallym_cred: f64,
    /// 경희대 지역균형: 탐구·진로 A → 1등급 대체 전/후 학점 가중 등급평균
    pub kh_before: Option<f64>,
    pub kh_after: Option<f64>,
    pub kh_gain: Option<f64>,
    pub kh_swapped: Vec<String>,
    pub kh_excluded: Vec<String>,
    /// 동국대: 상위 10과목 영역점수 평균 (만점 10)
    pub dongguk: Option<f64>,
    pub dongguk_used: Vec<(String, i64, f64)>,
    pub dongguk_total: i64,
}

pub fn calc_univ(recs: &[Rec], term: &str) -> UnivCalc {
    let mut u = UnivCalc::default();
    // 강원대
    let kp = |a: &str| match a { "A" => Some(500.0), "B" => Some(481.0), "C" => Some(461.0), "D" => Some(441.0), "E" => Some(400.0), _ => None };
    let (mut w, mut s) = (0.0, 0.0);
    for r in recs.iter().filter(|r| r.in_term(term) && r.ach_scale == 5) {
        if let Some(p) = r.achievement.as_deref().and_then(kp) {
            let c = if r.credits > 0.0 { r.credits } else { 1.0 };
            w += c;
            s += p * c;
            u.kangwon_n += 1;
            *u.kangwon_cnt.entry(r.achievement.clone().unwrap()).or_default() += 1;
        }
    }
    u.kangwon = if w > 0.0 { Some(round_to(s / w, 1)) } else { None };
    // 한림대
    let hp = |a: &str| match a { "A" => Some(60.0), "B" => Some(40.0), "C" => Some(30.0), "D" => Some(15.0), "E" => Some(0.0), _ => None };
    let (mut sum, mut n) = (0.0, 0);
    for r in recs.iter().filter(|r| r.in_term(term) && is_main_area(&r.area) && r.ach_scale == 5) {
        if let Some(p) = r.achievement.as_deref().and_then(hp) {
            sum += p;
            n += 1;
            u.hallym_cred += r.credits;
        }
    }
    u.hallym_n = n;
    u.hallym = if n > 0 { Some(round_to(sum / n as f64, 1)) } else { None };
    // 경희대
    let (mut bw, mut bs, mut aw, mut as_) = (0.0, 0.0, 0.0, 0.0);
    for r in recs.iter().filter(|r| r.in_term(term) && is_main_area(&r.area)) {
        let tamgu_area = (r.area == "과학" || r.area == "사회") && !is_common_subject(&r.subject, Some(&r.kind));
        let tamgu = tamgu_area || r.kind == "진로";
        let Some(g) = r.grade5 else {
            if r.ach_scale == 5 && tamgu_area && r.kind == "융합" {
                u.kh_excluded.push(r.subject.clone());
            }
            continue;
        };
        let c = if r.credits > 0.0 { r.credits } else { 1.0 };
        bw += c;
        bs += g as f64 * c;
        let mut g2 = g as f64;
        if tamgu && r.achievement.as_deref() == Some("A") && g > 1 {
            g2 = 1.0;
            u.kh_swapped.push(r.subject.clone());
        }
        aw += c;
        as_ += g2 * c;
    }
    if bw > 0.0 {
        u.kh_before = Some(round_to(bs / bw, 2));
        u.kh_after = Some(round_to(as_ / aw, 2));
        u.kh_gain = Some(round_to(bs / bw - as_ / aw, 2));
    }
    // 동국대
    let mut list: Vec<(String, i64, f64)> = recs
        .iter()
        .filter(|r| r.in_term(term) && is_main_area(&r.area) && !is_common_subject(&r.subject, Some(&r.kind)))
        .filter_map(|r| {
            let g = r.grade5?;
            let adj = dk_adj(r.achievement.as_deref()?)?;
            let zone = (g + adj).clamp(1, 9);
            Some((r.subject.clone(), zone, DK_PTS[(zone - 1) as usize]))
        })
        .collect();
    u.dongguk_total = list.len() as i64;
    list.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap().then(a.1.cmp(&b.1)));
    list.truncate(10);
    if !list.is_empty() {
        u.dongguk = Some(round_to(list.iter().map(|x| x.2).sum::<f64>() / list.len() as f64, 2));
    }
    u.dongguk_used = list;
    u
}

/// 학년 내 상위 % (값이 클수록 상위). sid → (rank, n, top%)
pub fn rank_by<F: Fn(&UnivCalc) -> Option<f64>>(d: &GradeData, term: &str, f: F) -> HashMap<i64, (i64, i64, f64)> {
    let mut list: Vec<(i64, f64)> = d.students.iter().filter_map(|s| f(&calc_univ(d.recs_of(s.id), term)).map(|v| (s.id, v))).collect();
    list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let n = list.len() as i64;
    list.iter().enumerate().map(|(i, (sid, _))| (*sid, ((i + 1) as i64, n, round_to((i + 1) as f64 / n as f64 * 100.0, 1)))).collect()
}

// ── 참고선 ───────────────────────────────────────────────────

pub const UNIV_SEED: &[(&str, &str, &str, &str, f64)] = &[
    ("서울권", "서울대", "인문/자연", "지역균형(교과)", 1.5),
    ("서울권", "연세대", "인문/자연", "추천형(교과)", 1.6),
    ("서울권", "고려대", "인문/자연", "학교추천(교과 80 + 서류 20)", 1.7),
    ("서울권", "한양대", "인문/자연", "추천형(교과)", 1.8),
    ("서울권", "성균관대", "인문/자연", "학교장추천(교과)", 1.9),
    ("서울권", "서강대", "인문/자연", "지역균형(교과)", 2.0),
    ("서울권", "중앙대", "인문/자연", "지역균형(교과)", 2.1),
    ("서울권", "서울시립대", "인문/자연", "지역균형(교과)", 2.2),
    ("서울권", "한국외대", "인문", "학교장추천(교과)", 2.3),
    ("서울권", "건국대", "인문/자연", "KU지역균형(교과)", 2.4),
    ("서울권", "숙명여대", "인문/자연", "지역균형(교과)", 2.6),
    ("서울권", "세종대", "인문/자연", "지역균형(교과)", 2.8),
    ("서울권", "국민대", "인문/자연", "교과성적우수자", 3.0),
    ("서울권", "숭실대", "인문/자연", "학생부우수자(교과)", 3.0),
    ("지방거점국립", "부산대", "인문/자연", "학생부교과", 2.6),
    ("지방거점국립", "경북대", "인문/자연", "교과우수자", 2.7),
    ("지방거점국립", "충남대", "인문/자연", "일반전형(교과)", 3.0),
    ("지방거점국립", "전남대", "인문/자연", "학생부교과(일반)", 3.2),
    ("지방거점국립", "충북대", "인문/자연", "학생부교과", 3.3),
    ("지방거점국립", "전북대", "인문/자연", "일반학생(교과)", 3.4),
    ("지방거점국립", "제주대", "인문/자연", "일반학생(교과)", 3.7),
    ("지방거점국립", "경상국립대", "인문/자연", "일반전형(교과)", 3.8),
    ("강원권", "강원대(춘천)", "의예과", "지역인재(교과)", 1.2),
    ("강원권", "한림대", "의예과", "지역인재(교과)", 1.2),
    ("강원권", "강원대(춘천)", "간호학과", "일반전형(교과)", 2.4),
    ("강원권", "한림대", "간호학과", "학생부교과(일반)", 2.5),
    ("강원권", "연세대 미래캠퍼스", "인문/자연", "교과우수자", 3.2),
    ("강원권", "강원대(춘천)", "인문/자연", "일반전형(교과)", 3.6),
    ("강원권", "한림대", "인문/자연", "학생부교과(일반)", 3.7),
    ("강원권", "강릉원주대", "인문/자연", "교과일반", 4.4),
    ("강원권", "가톨릭관동대", "인문/자연", "교과일반", 4.6),
    ("강원권", "한라대", "인문/자연", "일반전형(교과)", 5.0),
];

pub fn seed_reference(conn: &Connection) -> rusqlite::Result<()> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM UnivReference", [], |r| r.get(0))?;
    if n > 0 {
        return Ok(());
    }
    for (i, (region, univ, unit, adm, cut)) in UNIV_SEED.iter().enumerate() {
        conn.execute(
            "INSERT INTO UnivReference (region, univ, unit, adm_type, cut, note, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, '예시값 — 학교 입결로 교체', ?6)",
            params![region, univ, unit, adm, cut, i as i64],
        )?;
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
pub struct RefRow {
    pub id: i64,
    pub region: String,
    pub univ: String,
    pub unit: Option<String>,
    pub adm_type: Option<String>,
    pub cut: Option<f64>,
    pub note: Option<String>,
}

pub fn load_reference(conn: &Connection) -> Result<Vec<RefRow>, String> {
    let mut stmt = conn.prepare("SELECT id, region, univ, unit, adm_type, cut, note FROM UnivReference ORDER BY sort_order, id").map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok(RefRow { id: r.get(0)?, region: r.get(1)?, univ: r.get(2)?, unit: r.get(3)?, adm_type: r.get(4)?, cut: r.get(5)?, note: r.get(6)? }))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

pub fn update_reference(conn: &Connection, id: i64, cut: Option<f64>, note: Option<&str>) -> Result<(), String> {
    conn.execute("UPDATE UnivReference SET cut = ?1, note = ?2 WHERE id = ?3", params![cut, note, id]).map_err(|e| e.to_string())?;
    Ok(())
}

// ── 3개년 입결 집계 ───────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct AdmissionAgg {
    pub univ: String,
    pub adm_type: String,
    pub track: String,
    pub years: Vec<i64>,
    pub apply_n: i64,
    pub pass_n: i64,
    pub pass_rate: f64,
    pub apply_min: Option<f64>,
    pub apply_max: Option<f64>,
    pub pass_min: Option<f64>,
    pub pass_median: Option<f64>,
    pub pass_max: Option<f64>,
}

fn median(v: &mut Vec<f64>) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    Some(if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 })
}

/// 대학·전형유형·계열별 지원/합격 집계 (전교과 등급 기준, 9등급제 시절 값)
pub fn admission_aggregate(conn: &Connection, min_apply: i64) -> Result<Vec<AdmissionAgg>, String> {
    let mut stmt = conn
        .prepare("SELECT univ, COALESCE(adm_type,''), COALESCE(track,''), adm_year, final, grade_all FROM AdmissionResult")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?, r.get::<_, Option<String>>(4)?, r.get::<_, Option<f64>>(5)?)))
        .map_err(|e| e.to_string())?;
    let mut groups: HashMap<(String, String, String), (Vec<i64>, Vec<f64>, Vec<f64>, i64, i64)> = HashMap::new();
    for row in rows {
        let (univ, at, tr, yr, fin, g) = row.map_err(|e| e.to_string())?;
        let e = groups.entry((univ, at, tr)).or_insert_with(|| (vec![], vec![], vec![], 0, 0));
        if !e.0.contains(&yr) {
            e.0.push(yr);
        }
        e.3 += 1;
        let passed = matches!(fin.as_deref(), Some("합격") | Some("충원합격"));
        if passed {
            e.4 += 1;
        }
        if let Some(g) = g {
            e.1.push(g);
            if passed {
                e.2.push(g);
            }
        }
    }
    let mut out: Vec<AdmissionAgg> = groups
        .into_iter()
        .filter(|(_, v)| v.3 >= min_apply)
        .map(|((univ, adm_type, track), (mut years, apply, mut pass, an, pn))| {
            years.sort();
            AdmissionAgg {
                univ,
                adm_type,
                track,
                years,
                apply_n: an,
                pass_n: pn,
                pass_rate: round_to(pn as f64 / an.max(1) as f64 * 100.0, 1),
                apply_min: apply.iter().cloned().fold(None, |m: Option<f64>, x| Some(m.map_or(x, |y| y.min(x)))),
                apply_max: apply.iter().cloned().fold(None, |m: Option<f64>, x| Some(m.map_or(x, |y| y.max(x)))),
                pass_min: pass.iter().cloned().fold(None, |m: Option<f64>, x| Some(m.map_or(x, |y| y.min(x)))),
                pass_max: pass.iter().cloned().fold(None, |m: Option<f64>, x| Some(m.map_or(x, |y| y.max(x)))),
                pass_median: median(&mut pass).map(|v| round_to(v, 2)),
            }
        })
        .collect();
    out.sort_by(|a, b| b.apply_n.cmp(&a.apply_n).then(a.univ.cmp(&b.univ)));
    Ok(out)
}

#[derive(Clone, Debug, Serialize)]
pub struct RegionalRow {
    pub track: String,
    pub category: String,
    pub univ: String,
    pub adm_name: Option<String>,
    pub apply_n: Option<i64>,
    pub apply_best: Option<f64>,
    pub apply_worst: Option<f64>,
    pub pass_n: Option<i64>,
    pub pass_best: Option<f64>,
    pub pass_worst: Option<f64>,
    pub pass_rate: Option<f64>,
    pub cut70: Option<f64>,
}

pub fn load_regional(conn: &Connection) -> Result<Vec<RegionalRow>, String> {
    let mut stmt = conn
        .prepare("SELECT track, category, univ, adm_name, apply_n, apply_best, apply_worst, pass_n, pass_best, pass_worst, pass_rate, cut70 FROM RegionalAdmission ORDER BY track, category, univ, adm_name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(RegionalRow { track: r.get(0)?, category: r.get(1)?, univ: r.get(2)?, adm_name: r.get(3)?, apply_n: r.get(4)?, apply_best: r.get(5)?, apply_worst: r.get(6)?, pass_n: r.get(7)?, pass_best: r.get(8)?, pass_worst: r.get(9)?, pass_rate: r.get(10)?, cut70: r.get(11)? })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

/// 9등급제 시절 입결과 맞대기 위한 연속 환산 등급 (석차백분율 → 9등급 컷 위 선형 보간, 1.0~9.99)
pub fn grade9_continuous(top_pct: f64, cuts9: &[f64]) -> f64 {
    let mut prev = 0.0;
    for (i, &c) in cuts9.iter().enumerate() {
        if top_pct <= c {
            let frac = if c > prev { (top_pct - prev) / (c - prev) } else { 0.0 };
            return round_to(i as f64 + 1.0 + frac.clamp(0.0, 0.999), 2);
        }
        prev = c;
    }
    9.99
}
