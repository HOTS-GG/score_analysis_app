//! 플래그 기준값 (스펙 4장). 코드에 하드코딩하지 않고 프로젝트 DB 설정에 JSON으로 저장한다.
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub const CONFIG_KEY: &str = "thresholds";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Thresholds {
    // ── 추이 플래그 (백분위 기준) ──
    /// 연속 하락: Δ(3→6) ≤ 이 값 AND Δ(6→9) ≤ 이 값
    pub consecutive_drop_delta: f64,
    /// 급락: 어느 구간이든 Δ ≤ 이 값
    pub sharp_drop_delta: f64,
    /// 과목 불균형: subject_gap ≥ 이 값
    pub subject_gap_min: f64,
    /// 기복: volatility ≥ 이 값
    pub volatility_min: f64,
    /// 유형 불일치: |consistency| ≥ 이 값 (교내 석차백분율 차)
    pub consistency_abs_min: f64,
    /// 저평가 성장: z_resid ≥ 이 값이 growth_count 회 이상
    pub z_resid_growth: f64,
    pub z_resid_growth_count: i64,
    /// 조용한 하락: z_resid ≤ 이 값인데 등급 유지
    pub z_resid_quiet_drop: f64,
    /// 적응 실패: 반배치 상위 X% → 9월 교내 하위 Y%
    pub adaptation_entry_top_pct: f64,
    pub adaptation_sept_bottom_pct: f64,
    /// 집단 착시: 성취도 목록 ∧ 내신 등급 목록
    pub group_illusion_achievements: Vec<String>,
    pub group_illusion_grades: Vec<i64>,
    /// 경계선: cutline_margin ≤ 이 값
    pub cutline_margin_max: f64,
    /// 학급 화면 태그: 직전 학기 대비 교내 위치(석차백분율 환산) 하락 ≥ 이 값이면 표시
    pub pos_delta_warn: f64,
    /// 학급 화면 태그: 성취도 E 과목 수 ≥ 이 값이면 표시
    pub e_count_warn: i64,
    /// 학급 화면 태그: 입학 대비 이동 ≥ 이 값(p)이면 표시
    pub entry_move_warn: f64,
    /// 학급 화면 태그: 가장 약한 교과 등급평균 − 주요교과 등급평균 ≥ 이 값이면 "○○만 유독 낮음"
    pub area_gap_warn: f64,
    /// 실수형: 전국 정답률 easy 이상 문항 실점이 총 실점의 이 % 이상
    pub careless_easy_item_share: f64,
    /// 난이도 구간 (전국 정답률 %)
    pub easy_item_national_rate: f64,
    pub hard_item_national_rate: f64,
    /// 문항 격차 집중 구간 보고 기준 (교내−전국, %p)
    pub item_gap_report: f64,
    /// 공통 오개념 후보: 특정 오답 선지 집중 비율 %
    pub misconception_choice_share: f64,

    // ── 성실도 필터 ──
    pub sincerity_blank_ratio: f64,
    pub sincerity_same_choice_run: i64,

    // ── 통계 가드 ──
    pub min_n_regression: i64,
    pub min_r_regression: f64,
    pub min_n_equipercentile: i64,
    pub min_n_warn: i64,
    pub transition_cell_min: i64,
    pub skew_warn: f64,
    pub pct_clip_low: f64,
    pub pct_clip_high: f64,
    /// 상위권 표준점수 추적 기준 (백분위)
    pub top_tier_pct: f64,

    // ── 등급컷 (누적 %) ──
    pub grade5_cuts: Vec<f64>,
    pub grade9_cuts: Vec<f64>,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            consecutive_drop_delta: -7.0,
            sharp_drop_delta: -15.0,
            subject_gap_min: 30.0,
            volatility_min: 15.0,
            consistency_abs_min: 20.0,
            z_resid_growth: 1.0,
            z_resid_growth_count: 2,
            z_resid_quiet_drop: -1.0,
            adaptation_entry_top_pct: 30.0,
            adaptation_sept_bottom_pct: 50.0,
            group_illusion_achievements: vec!["C".into(), "D".into(), "E".into()],
            group_illusion_grades: vec![1, 2],
            cutline_margin_max: 3.0,
            pos_delta_warn: 10.0,
            e_count_warn: 3,
            entry_move_warn: 20.0,
            area_gap_warn: 1.2,
            careless_easy_item_share: 40.0,
            easy_item_national_rate: 70.0,
            hard_item_national_rate: 30.0,
            item_gap_report: -15.0,
            misconception_choice_share: 40.0,
            sincerity_blank_ratio: 20.0,
            sincerity_same_choice_run: 5,
            min_n_regression: 100,
            min_r_regression: 0.4,
            min_n_equipercentile: 200,
            min_n_warn: 30,
            transition_cell_min: 10,
            skew_warn: 0.5,
            pct_clip_low: 0.5,
            pct_clip_high: 99.5,
            top_tier_pct: 96.0,
            grade5_cuts: vec![10.0, 34.0, 66.0, 90.0, 100.0],
            grade9_cuts: vec![4.0, 11.0, 23.0, 40.0, 60.0, 77.0, 89.0, 96.0, 100.0],
        }
    }
}

pub fn load(conn: &Connection) -> Result<Thresholds, String> {
    let stored: Option<String> = conn
        .query_row(
            "SELECT config_value FROM APP_CONFIGS WHERE config_key = ?1",
            [CONFIG_KEY],
            |r| r.get(0),
        )
        .ok();
    match stored {
        None => Ok(Thresholds::default()),
        Some(json) => serde_json::from_str::<Thresholds>(&json)
            .map_err(|e| format!("기준값 설정을 읽을 수 없습니다: {e}")),
    }
}

pub fn save(conn: &Connection, t: &Thresholds) -> Result<(), String> {
    validate(t)?;
    let json = serde_json::to_string(t).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO APP_CONFIGS (config_key, config_value) VALUES (?1, ?2)",
        [CONFIG_KEY, &json],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reset(conn: &Connection) -> Result<Thresholds, String> {
    conn.execute("DELETE FROM APP_CONFIGS WHERE config_key = ?1", [CONFIG_KEY])
        .map_err(|e| e.to_string())?;
    Ok(Thresholds::default())
}

pub fn validate(t: &Thresholds) -> Result<(), String> {
    if !(0.0 < t.pct_clip_low && t.pct_clip_low < t.pct_clip_high && t.pct_clip_high < 100.0) {
        return Err("백분위 클리핑 범위가 잘못되었습니다 (0 < low < high < 100).".into());
    }
    if t.easy_item_national_rate <= t.hard_item_national_rate {
        return Err("쉬운 문항 기준 정답률은 어려운 문항 기준보다 커야 합니다.".into());
    }
    if t.min_n_warn < 1 || t.min_n_regression < 1 || t.min_n_equipercentile < 1 {
        return Err("최소 N 값은 1 이상이어야 합니다.".into());
    }
    if !(0.0..=1.0).contains(&t.min_r_regression) {
        return Err("회귀 최소 상관계수는 0~1 사이여야 합니다.".into());
    }
    if t.z_resid_growth_count < 1 {
        return Err("저평가 성장 판정 회수는 1 이상이어야 합니다.".into());
    }
    check_cuts(&t.grade5_cuts, 5, "5등급제")?;
    check_cuts(&t.grade9_cuts, 9, "9등급제")?;
    Ok(())
}

fn check_cuts(cuts: &[f64], n: usize, label: &str) -> Result<(), String> {
    if cuts.len() != n {
        return Err(format!("{label} 등급컷은 {n}개여야 합니다."));
    }
    let mut prev = 0.0;
    for &c in cuts {
        if c <= prev || c > 100.0 {
            return Err(format!("{label} 등급컷은 증가하는 누적 비율(≤100)이어야 합니다."));
        }
        prev = c;
    }
    if (cuts[n - 1] - 100.0).abs() > 1e-9 {
        return Err(format!("{label} 마지막 등급컷은 100이어야 합니다."));
    }
    Ok(())
}

/// 누적 백분율 → 등급 (표시용 변환. 등급끼리 연산 금지)
pub fn grade_from_top_pct(top_pct: f64, cuts: &[f64]) -> i64 {
    for (i, &c) in cuts.iter().enumerate() {
        if top_pct <= c {
            return (i + 1) as i64;
        }
    }
    cuts.len() as i64
}
