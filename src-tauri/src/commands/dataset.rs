//! 데이터셋 가져오기 — 컬럼 매핑 적용, 형 변환, 학생 매칭, upsert.
//! 프론트는 파일을 파싱해 {헤더: 값} 행 배열만 넘긴다. 매핑 적용과 검증은 전부 여기서 한다.
use crate::engine::{parse_achievement, parse_f64, parse_i64};
use crate::state::DbState;
use crate::types::*;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{Map, Value};
use std::collections::HashMap;
use tauri::State;

type Row = Map<String, Value>;

// ── 표준 필드 정의 ────────────────────────────────────────────

fn f(key: &str, label: &str, required: bool, kind: &str, hint: &str) -> FieldSpec {
    FieldSpec {
        key: key.into(),
        label: label.into(),
        required,
        kind: kind.into(),
        hint: hint.into(),
    }
}

fn student_keys() -> Vec<FieldSpec> {
    vec![
        f("class_no", "반", true, "int", ""),
        f("number", "번호", true, "int", ""),
        f("name", "이름", false, "text", "매칭 확인용. 분석 테이블에는 저장하지 않음"),
        f("grade", "학년", false, "int", "비우면 공통값 사용"),
    ]
}

pub fn dataset_specs() -> Vec<DatasetSpec> {
    vec![
        DatasetSpec {
            id: "students".into(),
            label: "학생 명단".into(),
            desc: "학년·반·번호·이름. 분석에 쓰이는 익명 ID는 자동 부여됩니다.".into(),
            key_fields: vec![
                f("class_no", "반", true, "int", ""),
                f("number", "번호", true, "int", ""),
                f("name", "이름", false, "text", ""),
                f("grade", "학년", false, "int", "비우면 공통값 사용"),
                f("cohort", "입학연도", false, "int", "비우면 공통값 사용"),
            ],
            context_fields: vec![
                f("cohort", "입학연도", true, "int", "예: 2026"),
                f("grade", "학년", true, "int", "1~3"),
            ],
            subject_fields: vec![],
            value_fields: vec![],
            supports_subjects: false,
        },
        DatasetSpec {
            id: "mock".into(),
            label: "모의고사(학력평가)".into(),
            desc: "회차 × 과목별 원점수·표준점수·전국 백분위·등급. 결시는 빈칸으로 두세요 (0 아님).".into(),
            key_fields: student_keys(),
            context_fields: vec![
                f("grade_year", "학년도", true, "int", "예: 2026"),
                f("grade", "학년", true, "int", "1~3"),
                f("round", "회차(월)", true, "int", "3 / 6 / 9 / 11"),
            ],
            subject_fields: vec![
                f("raw", "원점수", false, "float", ""),
                f("std", "표준점수", false, "float", "상위권 추적에 필요"),
                f("pct", "전국 백분위", false, "float", "핵심 지표"),
                f("grade9", "등급(9등급)", false, "int", "표시용"),
                f("absent", "결시 여부", false, "text", "Y/1/결시 → 결시"),
                f("blank_n", "무응답 문항 수", false, "int", "성실도 필터"),
            ],
            value_fields: vec![f("subject", "과목", true, "text", "세로형일 때 과목 컬럼")],
            supports_subjects: true,
        },
        DatasetSpec {
            id: "school".into(),
            label: "내신".into(),
            desc: "학기 × 과목별 성취도·등급(5등급)·원점수·과목평균·표준편차·수강자 수.".into(),
            key_fields: student_keys(),
            context_fields: vec![
                f("grade_year", "학년도", true, "int", "예: 2026"),
                f("grade", "학년", true, "int", "1~3"),
                f("semester", "학기", true, "int", "1 / 2"),
            ],
            subject_fields: vec![
                f("achievement", "성취도(A~E)", false, "text", "절대 기준"),
                f("grade5", "등급(5등급)", false, "int", "성취도만 있는 과목은 비움"),
                f("raw", "원점수", false, "float", ""),
                f("subject_mean", "과목 평균", false, "float", "개인 분석 z점수에 필요"),
                f("subject_sd", "표준편차", false, "float", "개인 분석 z점수에 필요"),
                f("enrolled_n", "수강자 수", false, "int", "석차 위치에 필요"),
                f("rank", "석차", false, "int", "없으면 z점수로 추정"),
                f("credits", "단위수", false, "int", "과목명 뒤 (4) 로도 인식"),
                f("written", "지필 점수", false, "float", "가능하면"),
                f("performance", "수행 점수", false, "float", "가능하면"),
            ],
            value_fields: vec![f("subject", "과목", true, "text", "세로형일 때 과목 컬럼")],
            supports_subjects: true,
        },
        DatasetSpec {
            id: "entry".into(),
            label: "입학 시점(고입내신·반배치)".into(),
            desc: "고입내신 환산점수, 반배치고사 교내 석차와 과목별 원점수.".into(),
            key_fields: student_keys(),
            context_fields: vec![f("grade", "학년", true, "int", "명단 매칭용")],
            subject_fields: vec![f("raw", "반배치 원점수", false, "float", "")],
            value_fields: vec![
                f("mid_school_score", "고입내신 환산점수", false, "float", "또는 석차백분율"),
                f("placement_rank", "반배치 교내 석차", false, "float", ""),
                f("placement_n", "반배치 응시 인원", false, "int", "비우면 명단 인원 사용"),
            ],
            supports_subjects: true,
        },
        DatasetSpec {
            id: "item_stat".into(),
            label: "문항 통계".into(),
            desc: "문항별 교내 정답률·전국 정답률·선지별 응답 분포 (학생 단위 아님).".into(),
            key_fields: vec![],
            context_fields: vec![
                f("grade_year", "학년도", true, "int", ""),
                f("round", "회차(월)", true, "int", ""),
                f("subject", "과목", true, "text", ""),
            ],
            subject_fields: vec![],
            value_fields: vec![
                f("item_no", "문항 번호", true, "int", ""),
                f("unit", "단원", false, "text", ""),
                f("item_type", "유형", false, "text", ""),
                f("our_correct_rate", "교내 정답률(%)", false, "float", ""),
                f("national_correct_rate", "전국 정답률(%)", false, "float", ""),
                f("answer_distribution", "선지별 응답 분포", false, "text", "예: 12/48/20/15/5"),
            ],
            supports_subjects: false,
        },
        DatasetSpec {
            id: "item_response".into(),
            label: "문항 응답(학생별)".into(),
            desc: "학생 × 문항 정오답. 한 행 = 학생 1명 × 문항 1개 (세로형).".into(),
            key_fields: student_keys(),
            context_fields: vec![
                f("grade_year", "학년도", true, "int", ""),
                f("grade", "학년", true, "int", ""),
                f("round", "회차(월)", true, "int", ""),
                f("subject", "과목", true, "text", ""),
            ],
            subject_fields: vec![],
            value_fields: vec![
                f("item_no", "문항 번호", true, "int", ""),
                f("correct", "정답 여부", false, "text", "O/X, 1/0, Y/N"),
                f("chosen", "선택 선지", false, "text", ""),
            ],
            supports_subjects: false,
        },
        DatasetSpec {
            id: "attendance".into(),
            label: "출결·과제".into(),
            desc: "월별 지각·조퇴·결석 횟수, 과제 제출률.".into(),
            key_fields: student_keys(),
            context_fields: vec![
                f("grade_year", "학년도", true, "int", ""),
                f("grade", "학년", true, "int", ""),
            ],
            subject_fields: vec![],
            value_fields: vec![
                f("month", "월", true, "int", "1~12"),
                f("late", "지각", false, "int", ""),
                f("early_leave", "조퇴", false, "int", ""),
                f("absent", "결석", false, "int", ""),
                f("homework_rate", "과제 제출률(%)", false, "float", ""),
            ],
            supports_subjects: false,
        },
    ]
}

fn spec_of(dataset: &str) -> Result<DatasetSpec, String> {
    dataset_specs()
        .into_iter()
        .find(|s| s.id == dataset)
        .ok_or_else(|| format!("알 수 없는 데이터셋: {dataset}"))
}

// ── 매핑 자동 추정 ────────────────────────────────────────────

/// 긴 이름을 앞에 두어 "통합사회"가 "사회"로 잘리지 않게 한다.
const SUBJECT_NAMES: &[&str] = &[
    "통합사회", "통합과학", "한국사", "생명과학", "지구과학", "물리학", "화학", "사회문화", "생활과윤리",
    "윤리와사상", "정치와법", "한국지리", "세계지리", "동아시아사", "세계사", "경제", "제2외국어",
    "일본어", "중국어", "한문", "국어", "수학", "영어", "통사", "통과", "과학", "사회", "탐구", "물리",
    "생명", "지구",
];

/// "문학(4)" → ("문학", Some(4)). 괄호 안이 숫자가 아니면 그대로 둔다.
pub fn split_subject_credits(s: &str) -> (String, Option<i64>) {
    let t = s.trim();
    if let (Some(open), true) = (t.rfind('('), t.ends_with(')')) {
        let inner = &t[open + 1..t.len() - 1];
        if let Ok(c) = inner.trim().parse::<i64>() {
            return (t[..open].trim().to_string(), Some(c));
        }
    }
    (t.to_string(), None)
}

/// 파일마다 다른 축약 과목명을 표준 이름으로 통일한다 (통사 → 통합사회 등).
pub fn normalize_subject(s: &str) -> String {
    let key = s.trim().replace(' ', "");
    match key.as_str() {
        "통사" | "통합사회" => "통합사회".into(),
        "통과" | "통합과학" => "통합과학".into(),
        "국" | "국어" => "국어".into(),
        "수" | "수학" => "수학".into(),
        "영" | "영어" => "영어".into(),
        "한국사영역" | "한사" | "한국사" => "한국사".into(),
        "제2외" | "제2외국어/한문" | "제2외국어" => "제2외국어".into(),
        // 그 외 과목은 띄어쓰기를 하나로만 정리하고 원래 이름을 유지 ("사회문제 탐구")
        _ => s.split_whitespace().collect::<Vec<_>>().join(" "),
    }
}

fn header_matches(h: &str, kws: &[&str]) -> bool {
    let hl = h.replace(' ', "").to_lowercase();
    kws.iter().any(|k| hl.contains(&k.replace(' ', "").to_lowercase()))
}

fn key_keywords(key: &str) -> &'static [&'static str] {
    match key {
        "grade" => &["학년"],
        "class_no" => &["반", "class"],
        "number" => &["번호", "번", "no."],
        "name" => &["이름", "성명", "name"],
        "cohort" => &["입학연도", "입학년도", "cohort"],
        "subject" => &["과목", "교과"],
        "raw" => &["원점수", "점수", "raw"],
        "std" => &["표준점수", "표점", "std"],
        "pct" => &["백분위", "pct", "percentile"],
        "grade9" | "grade5" => &["등급"],
        "rank" => &["석차", "순위"],
        "credits" => &["단위"],
        "absent" => &["결시", "응시"],
        "blank_n" => &["무응답", "미기재"],
        "achievement" => &["성취도"],
        "subject_mean" => &["평균"],
        "subject_sd" => &["표준편차"],
        "enrolled_n" => &["수강자", "수강인원", "인원"],
        "written" => &["지필"],
        "performance" => &["수행"],
        "mid_school_score" => &["고입", "중학교내신", "환산", "석차백분율"],
        "placement_rank" => &["석차", "순위", "등수"],
        "placement_n" => &["응시인원", "인원"],
        "item_no" => &["문항", "번호"],
        "unit" => &["단원", "영역"],
        "item_type" => &["유형"],
        "our_correct_rate" => &["교내정답률", "본교", "학교정답률"],
        "national_correct_rate" => &["전국정답률", "전국"],
        "answer_distribution" => &["선지", "응답분포", "반응"],
        "correct" => &["정답여부", "정오", "정답"],
        "chosen" => &["선택", "응답"],
        "month" => &["월"],
        "late" => &["지각"],
        "early_leave" => &["조퇴"],
        "homework_rate" => &["과제", "제출"],
        _ => &[],
    }
}

fn pick_header<'a>(headers: &'a [String], key: &str, exclude_subject: bool) -> Option<&'a String> {
    let kws = key_keywords(key);
    headers.iter().find(|h| {
        if exclude_subject && SUBJECT_NAMES.iter().any(|s| h.contains(s)) {
            return false;
        }
        // "반" 은 "반배치" 등에 오탐하므로 정확 일치 우선
        if key == "class_no" {
            let t = h.trim();
            return t == "반" || t.to_lowercase() == "class";
        }
        if key == "number" {
            let t = h.trim();
            return t == "번호" || t == "번";
        }
        // 석차는 "석차백분율"·"응시인원"이 아닌 열, 인원은 "인원"이 들어간 열
        if key == "placement_rank" && (h.contains("백분") || h.contains("인원")) {
            return false;
        }
        if key == "mid_school_score" && h.contains("반배치") {
            return false;
        }
        header_matches(h, kws)
    })
}

pub fn suggest_mapping_impl(dataset: &str, headers: &[String]) -> Result<ImportMapping, String> {
    let spec = spec_of(dataset)?;
    let mut m = ImportMapping {
        mode: "wide".into(),
        ..Default::default()
    };
    for fs in &spec.key_fields {
        let h = pick_header(headers, &fs.key, true).cloned();
        m.keys.insert(fs.key.clone(), h);
    }
    for fs in &spec.value_fields {
        if fs.key == "subject" {
            continue;
        }
        let h = pick_header(headers, &fs.key, false).cloned();
        m.values.insert(fs.key.clone(), h);
    }
    if spec.supports_subjects {
        // 헤더에서 과목명이 포함된 컬럼을 과목별로 묶는다
        let mut order: Vec<String> = Vec::new();
        let mut blocks: HashMap<String, SubjectMapping> = HashMap::new();
        for h in headers {
            let Some(subj) = SUBJECT_NAMES.iter().find(|s| h.contains(*s)) else { continue };
            let subj = subj.to_string();
            let rest = h.replace(&subj, "");
            let entry = blocks.entry(subj.clone()).or_insert_with(|| {
                order.push(subj.clone());
                SubjectMapping { subject: subj.clone(), columns: HashMap::new() }
            });
            for fs in &spec.subject_fields {
                let matched = if rest.trim().is_empty() {
                    // "국어" 단독 컬럼 → 대표 필드(pct 또는 raw)
                    fs.key == if dataset == "mock" { "pct" } else { "raw" }
                } else if fs.key == "raw" && rest.contains("표준") {
                    false // "표준점수"가 "점수"로 원점수에 잡히는 것 방지
                } else if fs.key == "rank" && rest.contains("등급") {
                    false // "석차등급"은 등급이지 석차가 아님
                } else if (fs.key == "grade5" || fs.key == "grade9") && rest.contains("평균") {
                    false // "등급평균" 제외
                } else {
                    header_matches(&rest, key_keywords(&fs.key))
                };
                if matched && !entry.columns.get(&fs.key).map(|v| v.is_some()).unwrap_or(false) {
                    entry.columns.insert(fs.key.clone(), Some(h.clone()));
                }
            }
            // "반배치 국어"처럼 항목 이름이 없고 과목 필드가 하나뿐이면 그 필드로
            if spec.subject_fields.len() == 1 && !entry.columns.values().any(|v| v.is_some()) {
                entry.columns.insert(spec.subject_fields[0].key.clone(), Some(h.clone()));
            }
        }
        m.subjects = order.into_iter().filter_map(|s| blocks.remove(&s)).collect();
        if m.subjects.is_empty() {
            // 세로형으로 추정
            if let Some(h) = pick_header(headers, "subject", false) {
                m.mode = "long".into();
                m.values.insert("subject".into(), Some(h.clone()));
                for fs in &spec.subject_fields {
                    let h = pick_header(headers, &fs.key, false).cloned();
                    m.values.insert(fs.key.clone(), h);
                }
            }
        }
    }
    Ok(m)
}

// ── 셀 읽기 ──────────────────────────────────────────────────

fn cell_str(row: &Row, col: &Option<String>) -> Option<String> {
    let col = col.as_ref()?;
    match row.get(col)? {
        Value::Null => None,
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() { None } else { Some(t.to_string()) }
        }
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(if *b { "1".into() } else { "0".into() }),
        other => Some(other.to_string()),
    }
}

fn cell_f64(row: &Row, col: &Option<String>) -> Option<f64> {
    cell_str(row, col).and_then(|s| parse_f64(&s))
}

fn cell_i64(row: &Row, col: &Option<String>) -> Option<i64> {
    cell_str(row, col).and_then(|s| parse_i64(&s))
}

fn cell_bool(row: &Row, col: &Option<String>) -> Option<bool> {
    let s = cell_str(row, col)?;
    let t = s.trim().to_uppercase();
    Some(matches!(t.as_str(), "1" | "Y" | "YES" | "O" | "TRUE" | "결시" | "미응시" | "정답" | "정"))
}

fn ctx_i64(m: &ImportMapping, key: &str) -> Result<i64, String> {
    let v = m.context.get(key).ok_or_else(|| format!("공통값 '{key}' 이(가) 없습니다."))?;
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f.round() as i64)),
        Value::String(s) => parse_i64(s),
        _ => None,
    }
    .ok_or_else(|| format!("공통값 '{key}' 이(가) 숫자가 아닙니다."))
}

fn ctx_str(m: &ImportMapping, key: &str) -> Result<String, String> {
    let v = m.context.get(key).ok_or_else(|| format!("공통값 '{key}' 이(가) 없습니다."))?;
    let s = match v {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    };
    if s.is_empty() {
        return Err(format!("공통값 '{key}' 이(가) 비어 있습니다."));
    }
    Ok(s)
}

fn key_col<'a>(m: &'a ImportMapping, key: &str) -> &'a Option<String> {
    static NONE: Option<String> = None;
    m.keys.get(key).unwrap_or(&NONE)
}

fn val_col<'a>(m: &'a ImportMapping, key: &str) -> &'a Option<String> {
    static NONE: Option<String> = None;
    m.values.get(key).unwrap_or(&NONE)
}

// ── 학생 매칭 ─────────────────────────────────────────────────

struct StudentIndex {
    by_key: HashMap<(i64, i64, i64), i64>, // (cohort, class_no, number) → id
}

impl StudentIndex {
    fn load(conn: &Connection) -> Result<Self, String> {
        let mut stmt = conn
            .prepare("SELECT id, cohort, class_no, number FROM Student")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?)))
            .map_err(|e| e.to_string())?;
        let mut by_key = HashMap::new();
        for r in rows {
            let (id, cohort, class_no, number) = r.map_err(|e| e.to_string())?;
            by_key.insert((cohort, class_no, number), id);
        }
        Ok(Self { by_key })
    }
}

/// 학년도와 학년으로 입학연도 계산 (2026학년도 1학년 → 2026 입학)
fn cohort_of(grade_year: i64, grade: i64) -> i64 {
    grade_year - grade + 1
}

struct RowKey {
    class_no: i64,
    number: i64,
    grade: i64,
    label: String,
}

fn read_row_key(row: &Row, m: &ImportMapping, default_grade: i64) -> Result<RowKey, String> {
    let class_no = cell_i64(row, key_col(m, "class_no")).ok_or("반 값이 비어 있음")?;
    let number = cell_i64(row, key_col(m, "number")).ok_or("번호 값이 비어 있음")?;
    let grade = cell_i64(row, key_col(m, "grade")).unwrap_or(default_grade);
    let name = cell_str(row, key_col(m, "name")).unwrap_or_default();
    Ok(RowKey {
        class_no,
        number,
        grade,
        label: format!("{grade}학년 {class_no}반 {number}번 {name}").trim().to_string(),
    })
}

// ── 가져오기 본체 ─────────────────────────────────────────────

pub fn import_dataset_impl(
    conn: &mut Connection,
    dataset: &str,
    rows: &[Row],
    m: &ImportMapping,
    file_name: &str,
) -> Result<ImportResult, String> {
    let spec = spec_of(dataset)?;
    for fs in spec.key_fields.iter().filter(|f| f.required) {
        if key_col(m, &fs.key).is_none() {
            return Err(format!("필수 식별 필드 '{}' 의 컬럼을 지정하세요.", fs.label));
        }
    }
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut res = ImportResult { row_count: rows.len() as i64, ..Default::default() };
    match dataset {
        "students" => import_students(&tx, rows, m, &mut res)?,
        "mock" => import_mock(&tx, rows, m, &mut res)?,
        "school" => import_school(&tx, rows, m, &mut res)?,
        "entry" => import_entry(&tx, rows, m, &mut res)?,
        "item_stat" => import_item_stat(&tx, rows, m, &mut res)?,
        "item_response" => import_item_response(&tx, rows, m, &mut res)?,
        "attendance" => import_attendance(&tx, rows, m, &mut res)?,
        _ => return Err(format!("알 수 없는 데이터셋: {dataset}")),
    }
    res.unmatched.sort();
    res.unmatched.dedup();
    tx.execute(
        "INSERT INTO ImportLog (dataset, file_name, row_count, inserted, updated, skipped, mapping_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            dataset,
            file_name,
            res.row_count,
            res.inserted,
            res.updated,
            res.skipped,
            serde_json::to_string(m).unwrap_or_else(|_| "{}".into())
        ],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(res)
}

fn import_students(conn: &Connection, rows: &[Row], m: &ImportMapping, res: &mut ImportResult) -> Result<(), String> {
    let ctx_cohort = ctx_i64(m, "cohort")?;
    let ctx_grade = ctx_i64(m, "grade")?;
    for (i, row) in rows.iter().enumerate() {
        let k = match read_row_key(row, m, ctx_grade) {
            Ok(k) => k,
            Err(e) => {
                res.skipped += 1;
                res.errors.push(format!("{}행: {e}", i + 2));
                continue;
            }
        };
        let cohort = cell_i64(row, key_col(m, "cohort")).unwrap_or(ctx_cohort);
        let name = cell_str(row, key_col(m, "name")).unwrap_or_default();
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM Student WHERE cohort = ?1 AND class_no = ?2 AND number = ?3",
                params![cohort, k.class_no, k.number],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        match existing {
            Some(id) => {
                conn.execute(
                    "UPDATE Student SET grade = ?1, name = CASE WHEN ?2 = '' THEN name ELSE ?2 END WHERE id = ?3",
                    params![k.grade, name, id],
                )
                .map_err(|e| e.to_string())?;
                res.updated += 1;
            }
            None => {
                let anon = crate::commands::student::next_anon_id(conn)?;
                conn.execute(
                    "INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![anon, cohort, k.grade, k.class_no, k.number, name],
                )
                .map_err(|e| e.to_string())?;
                res.inserted += 1;
            }
        }
    }
    Ok(())
}

/// 과목 블록 또는 세로형 행에서 표준 필드 → 컬럼 매핑을 얻는다.
fn subject_columns<'a>(m: &'a ImportMapping) -> Vec<(String, std::borrow::Cow<'a, HashMap<String, Option<String>>>)> {
    if m.mode == "long" {
        vec![(String::new(), std::borrow::Cow::Borrowed(&m.values))]
    } else {
        m.subjects
            .iter()
            .filter(|s| !s.subject.trim().is_empty())
            .map(|s| (s.subject.trim().to_string(), std::borrow::Cow::Borrowed(&s.columns)))
            .collect()
    }
}

fn col<'a>(cols: &'a HashMap<String, Option<String>>, key: &str) -> &'a Option<String> {
    static NONE: Option<String> = None;
    cols.get(key).unwrap_or(&NONE)
}

fn resolve_student(
    idx: &StudentIndex,
    cohort: i64,
    k: &RowKey,
    res: &mut ImportResult,
) -> Option<i64> {
    match idx.by_key.get(&(cohort, k.class_no, k.number)) {
        Some(id) => Some(*id),
        None => {
            res.skipped += 1;
            res.unmatched.push(k.label.clone());
            None
        }
    }
}

fn import_mock(conn: &Connection, rows: &[Row], m: &ImportMapping, res: &mut ImportResult) -> Result<(), String> {
    let grade_year = ctx_i64(m, "grade_year")?;
    let grade = ctx_i64(m, "grade")?;
    let round = ctx_i64(m, "round")?;
    if !matches!(round, 3 | 6 | 9 | 11) {
        return Err("회차는 3, 6, 9, 11 중 하나여야 합니다.".into());
    }
    let blocks = subject_columns(m);
    if blocks.is_empty() {
        return Err("과목 블록을 하나 이상 지정하세요.".into());
    }
    let idx = StudentIndex::load(conn)?;
    let cohort = cohort_of(grade_year, grade);
    for (i, row) in rows.iter().enumerate() {
        let k = match read_row_key(row, m, grade) {
            Ok(k) => k,
            Err(e) => {
                res.skipped += 1;
                res.errors.push(format!("{}행: {e}", i + 2));
                continue;
            }
        };
        let Some(sid) = resolve_student(&idx, cohort, &k, res) else { continue };
        for (subj_name, cols) in &blocks {
            let subject = if m.mode == "long" {
                match cell_str(row, col(cols, "subject")) {
                    Some(s) => s,
                    None => {
                        res.skipped += 1;
                        res.errors.push(format!("{}행: 과목 값이 비어 있음", i + 2));
                        continue;
                    }
                }
            } else {
                subj_name.clone()
            };
            let subject = normalize_subject(&subject);
            let raw = cell_f64(row, col(cols, "raw"));
            let std = cell_f64(row, col(cols, "std"));
            let pct = cell_f64(row, col(cols, "pct"));
            let grade9 = cell_i64(row, col(cols, "grade9"));
            let absent_flag = cell_bool(row, col(cols, "absent")).unwrap_or(false);
            let blank_n = cell_i64(row, col(cols, "blank_n"));
            let has_data = raw.is_some() || std.is_some() || pct.is_some() || grade9.is_some();
            if !has_data && !absent_flag {
                continue; // 아무 값도 없음 → 기록 없음 (0점 치환 금지)
            }
            if let Some(p) = pct {
                if !(0.0..=100.0).contains(&p) {
                    res.errors.push(format!("{}행 {subject}: 백분위 범위 밖({p})", i + 2));
                    res.skipped += 1;
                    continue;
                }
            }
            let absent = if absent_flag || !has_data { 1 } else { 0 };
            upsert_count(
                conn,
                "SELECT 1 FROM MockScore WHERE student_id=?1 AND grade_year=?2 AND round=?3 AND subject=?4",
                params![sid, grade_year, round, subject],
                "INSERT INTO MockScore (student_id, grade_year, grade, round, subject, raw, std, pct, grade9, absent, blank_n)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(student_id, grade_year, round, subject) DO UPDATE SET
                   grade=excluded.grade, raw=excluded.raw, std=excluded.std, pct=excluded.pct,
                   grade9=excluded.grade9, absent=excluded.absent, blank_n=excluded.blank_n",
                params![sid, grade_year, grade, round, subject, raw, std, pct, grade9, absent, blank_n],
                res,
            )?;
        }
    }
    Ok(())
}

fn import_school(conn: &Connection, rows: &[Row], m: &ImportMapping, res: &mut ImportResult) -> Result<(), String> {
    let grade_year = ctx_i64(m, "grade_year")?;
    let grade = ctx_i64(m, "grade")?;
    let semester = ctx_i64(m, "semester")?;
    if !matches!(semester, 1 | 2) {
        return Err("학기는 1 또는 2 여야 합니다.".into());
    }
    let blocks = subject_columns(m);
    if blocks.is_empty() {
        return Err("과목 블록을 하나 이상 지정하세요.".into());
    }
    let idx = StudentIndex::load(conn)?;
    let cohort = cohort_of(grade_year, grade);
    for (i, row) in rows.iter().enumerate() {
        let k = match read_row_key(row, m, grade) {
            Ok(k) => k,
            Err(e) => {
                res.skipped += 1;
                res.errors.push(format!("{}행: {e}", i + 2));
                continue;
            }
        };
        let Some(sid) = resolve_student(&idx, cohort, &k, res) else { continue };
        for (subj_name, cols) in &blocks {
            let subject = if m.mode == "long" {
                match cell_str(row, col(cols, "subject")) {
                    Some(s) => s,
                    None => {
                        res.skipped += 1;
                        continue;
                    }
                }
            } else {
                subj_name.clone()
            };
            let (subject, credits_in_name) = split_subject_credits(&subject);
            let subject = normalize_subject(&subject);
            let achievement = cell_str(row, col(cols, "achievement")).and_then(|s| parse_achievement(&s));
            let grade5 = cell_i64(row, col(cols, "grade5"));
            let raw = cell_f64(row, col(cols, "raw"));
            let mean = cell_f64(row, col(cols, "subject_mean"));
            let sd = cell_f64(row, col(cols, "subject_sd"));
            let n = cell_i64(row, col(cols, "enrolled_n"));
            let rank = cell_i64(row, col(cols, "rank"));
            let credits = cell_i64(row, col(cols, "credits")).or(credits_in_name);
            let written = cell_f64(row, col(cols, "written"));
            let perf = cell_f64(row, col(cols, "performance"));
            if achievement.is_none() && grade5.is_none() && raw.is_none() {
                continue;
            }
            if let (Some(rk), Some(nn)) = (rank, n) {
                if rk < 1 || rk > nn {
                    res.errors.push(format!("{}행 {subject}: 석차 {rk}가 수강자 수 {nn} 범위 밖", i + 2));
                    res.skipped += 1;
                    continue;
                }
            }
            if let Some(g) = grade5 {
                if !(1..=9).contains(&g) {
                    res.errors.push(format!("{}행 {subject}: 등급 범위 밖({g})", i + 2));
                    res.skipped += 1;
                    continue;
                }
            }
            upsert_count(
                conn,
                "SELECT 1 FROM SchoolScore WHERE student_id=?1 AND grade_year=?2 AND semester=?3 AND subject=?4",
                params![sid, grade_year, semester, subject],
                "INSERT INTO SchoolScore (student_id, grade_year, grade, semester, subject, achievement, grade5, raw, subject_mean, subject_sd, enrolled_n, written, performance, rank, credits)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                 ON CONFLICT(student_id, grade_year, semester, subject) DO UPDATE SET
                   grade=excluded.grade, achievement=excluded.achievement, grade5=excluded.grade5, raw=excluded.raw,
                   subject_mean=excluded.subject_mean, subject_sd=excluded.subject_sd, enrolled_n=excluded.enrolled_n,
                   written=excluded.written, performance=excluded.performance, rank=excluded.rank, credits=excluded.credits",
                params![sid, grade_year, grade, semester, subject, achievement, grade5, raw, mean, sd, n, written, perf, rank, credits],
                res,
            )?;
        }
    }
    Ok(())
}

fn import_entry(conn: &Connection, rows: &[Row], m: &ImportMapping, res: &mut ImportResult) -> Result<(), String> {
    let grade = ctx_i64(m, "grade")?;
    let idx = StudentIndex::load(conn)?;
    // 입학 데이터는 학년 1 기준 cohort 를 명단에서 역산: 명단의 cohort 중 grade 에 해당하는 것을 사용
    let cohort: i64 = conn
        .query_row(
            "SELECT cohort FROM Student WHERE grade = ?1 GROUP BY cohort ORDER BY COUNT(*) DESC LIMIT 1",
            [grade],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or("해당 학년의 학생 명단이 없습니다. 학생 명단을 먼저 가져오세요.")?;
    let blocks = subject_columns(m);
    for (i, row) in rows.iter().enumerate() {
        let k = match read_row_key(row, m, grade) {
            Ok(k) => k,
            Err(e) => {
                res.skipped += 1;
                res.errors.push(format!("{}행: {e}", i + 2));
                continue;
            }
        };
        let Some(sid) = resolve_student(&idx, cohort, &k, res) else { continue };
        let mid = cell_f64(row, val_col(m, "mid_school_score"));
        let rank = cell_f64(row, val_col(m, "placement_rank"));
        let n = cell_i64(row, val_col(m, "placement_n"));
        upsert_count(
            conn,
            "SELECT 1 FROM EntryScore WHERE student_id=?1",
            params![sid],
            "INSERT INTO EntryScore (student_id, mid_school_score, placement_rank, placement_n) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(student_id) DO UPDATE SET
               mid_school_score=COALESCE(excluded.mid_school_score, mid_school_score),
               placement_rank=COALESCE(excluded.placement_rank, placement_rank),
               placement_n=COALESCE(excluded.placement_n, placement_n)",
            params![sid, mid, rank, n],
            res,
        )?;
        if m.mode != "long" {
            for (subj, cols) in &blocks {
                if let Some(raw) = cell_f64(row, col(cols, "raw")) {
                    conn.execute(
                        "INSERT INTO EntrySubjectScore (student_id, subject, raw) VALUES (?1, ?2, ?3)
                         ON CONFLICT(student_id, subject) DO UPDATE SET raw=excluded.raw",
                        params![sid, subj, raw],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
    }
    Ok(())
}

fn import_item_stat(conn: &Connection, rows: &[Row], m: &ImportMapping, res: &mut ImportResult) -> Result<(), String> {
    let grade_year = ctx_i64(m, "grade_year")?;
    let round = ctx_i64(m, "round")?;
    let subject = ctx_str(m, "subject")?;
    for (i, row) in rows.iter().enumerate() {
        let Some(item_no) = cell_i64(row, val_col(m, "item_no")) else {
            res.skipped += 1;
            res.errors.push(format!("{}행: 문항 번호 없음", i + 2));
            continue;
        };
        let unit = cell_str(row, val_col(m, "unit"));
        let item_type = cell_str(row, val_col(m, "item_type"));
        let our = cell_f64(row, val_col(m, "our_correct_rate"));
        let nat = cell_f64(row, val_col(m, "national_correct_rate"));
        let dist = cell_str(row, val_col(m, "answer_distribution"));
        upsert_count(
            conn,
            "SELECT 1 FROM ItemStat WHERE grade_year=?1 AND round=?2 AND subject=?3 AND item_no=?4",
            params![grade_year, round, subject, item_no],
            "INSERT INTO ItemStat (grade_year, round, subject, item_no, unit, item_type, our_correct_rate, national_correct_rate, answer_distribution)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(grade_year, round, subject, item_no) DO UPDATE SET
               unit=excluded.unit, item_type=excluded.item_type, our_correct_rate=excluded.our_correct_rate,
               national_correct_rate=excluded.national_correct_rate, answer_distribution=excluded.answer_distribution",
            params![grade_year, round, subject, item_no, unit, item_type, our, nat, dist],
            res,
        )?;
    }
    Ok(())
}

fn import_item_response(conn: &Connection, rows: &[Row], m: &ImportMapping, res: &mut ImportResult) -> Result<(), String> {
    let grade_year = ctx_i64(m, "grade_year")?;
    let grade = ctx_i64(m, "grade")?;
    let round = ctx_i64(m, "round")?;
    let subject = ctx_str(m, "subject")?;
    let idx = StudentIndex::load(conn)?;
    let cohort = cohort_of(grade_year, grade);
    for (i, row) in rows.iter().enumerate() {
        let k = match read_row_key(row, m, grade) {
            Ok(k) => k,
            Err(e) => {
                res.skipped += 1;
                res.errors.push(format!("{}행: {e}", i + 2));
                continue;
            }
        };
        let Some(sid) = resolve_student(&idx, cohort, &k, res) else { continue };
        let Some(item_no) = cell_i64(row, val_col(m, "item_no")) else {
            res.skipped += 1;
            continue;
        };
        let correct = cell_bool(row, val_col(m, "correct")).map(|b| b as i64);
        let chosen = cell_str(row, val_col(m, "chosen"));
        upsert_count(
            conn,
            "SELECT 1 FROM ItemResponse WHERE student_id=?1 AND grade_year=?2 AND round=?3 AND subject=?4 AND item_no=?5",
            params![sid, grade_year, round, subject, item_no],
            "INSERT INTO ItemResponse (student_id, grade_year, round, subject, item_no, correct, chosen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(student_id, grade_year, round, subject, item_no) DO UPDATE SET
               correct=excluded.correct, chosen=excluded.chosen",
            params![sid, grade_year, round, subject, item_no, correct, chosen],
            res,
        )?;
    }
    Ok(())
}

fn import_attendance(conn: &Connection, rows: &[Row], m: &ImportMapping, res: &mut ImportResult) -> Result<(), String> {
    let grade_year = ctx_i64(m, "grade_year")?;
    let grade = ctx_i64(m, "grade")?;
    let idx = StudentIndex::load(conn)?;
    let cohort = cohort_of(grade_year, grade);
    for (i, row) in rows.iter().enumerate() {
        let k = match read_row_key(row, m, grade) {
            Ok(k) => k,
            Err(e) => {
                res.skipped += 1;
                res.errors.push(format!("{}행: {e}", i + 2));
                continue;
            }
        };
        let Some(sid) = resolve_student(&idx, cohort, &k, res) else { continue };
        let Some(month) = cell_i64(row, val_col(m, "month")) else {
            res.skipped += 1;
            continue;
        };
        let late = cell_i64(row, val_col(m, "late")).unwrap_or(0);
        let early = cell_i64(row, val_col(m, "early_leave")).unwrap_or(0);
        let absent = cell_i64(row, val_col(m, "absent")).unwrap_or(0);
        let hw = cell_f64(row, val_col(m, "homework_rate"));
        upsert_count(
            conn,
            "SELECT 1 FROM Attendance WHERE student_id=?1 AND grade_year=?2 AND month=?3",
            params![sid, grade_year, month],
            "INSERT INTO Attendance (student_id, grade_year, month, late, early_leave, absent, homework_rate)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(student_id, grade_year, month) DO UPDATE SET
               late=excluded.late, early_leave=excluded.early_leave, absent=excluded.absent, homework_rate=excluded.homework_rate",
            params![sid, grade_year, month, late, early, absent, hw],
            res,
        )?;
    }
    Ok(())
}

fn upsert_count(
    conn: &Connection,
    exists_sql: &str,
    exists_params: &[&dyn rusqlite::ToSql],
    upsert_sql: &str,
    upsert_params: &[&dyn rusqlite::ToSql],
    res: &mut ImportResult,
) -> Result<(), String> {
    let exists: Option<i64> = conn
        .query_row(exists_sql, exists_params, |r| r.get(0))
        .optional()
        .map_err(|e| e.to_string())?;
    conn.execute(upsert_sql, upsert_params).map_err(|e| e.to_string())?;
    if exists.is_some() {
        res.updated += 1;
    } else {
        res.inserted += 1;
    }
    Ok(())
}

// ── 삭제 / 현황 / 로그 ───────────────────────────────────────

pub fn clear_dataset_impl(conn: &Connection, dataset: &str) -> Result<i64, String> {
    let sql = match dataset {
        "students" => "DELETE FROM Student",
        "mock" => "DELETE FROM MockScore",
        "school" => "DELETE FROM SchoolScore",
        "entry" => "DELETE FROM EntryScore; DELETE FROM EntrySubjectScore",
        "item_stat" => "DELETE FROM ItemStat",
        "item_response" => "DELETE FROM ItemResponse",
        "attendance" => "DELETE FROM Attendance",
        _ => return Err(format!("알 수 없는 데이터셋: {dataset}")),
    };
    let mut total = 0i64;
    for stmt in sql.split(';') {
        let s = stmt.trim();
        if s.is_empty() {
            continue;
        }
        total += conn.execute(s, []).map_err(|e| e.to_string())? as i64;
    }
    Ok(total)
}

fn count(conn: &Connection, sql: &str) -> Result<i64, String> {
    conn.query_row(sql, [], |r| r.get(0)).map_err(|e| e.to_string())
}

pub fn get_dataset_summary_impl(conn: &Connection) -> Result<DatasetSummary, String> {
    let mut s = DatasetSummary {
        students_total: count(conn, "SELECT COUNT(*) FROM Student")?,
        students_active: count(conn, "SELECT COUNT(*) FROM Student WHERE status='active'")?,
        students_excluded: count(conn, "SELECT COUNT(*) FROM Student WHERE status='excluded'")?,
        mock_rows: count(conn, "SELECT COUNT(*) FROM MockScore")?,
        school_rows: count(conn, "SELECT COUNT(*) FROM SchoolScore")?,
        entry_rows: count(conn, "SELECT COUNT(*) FROM EntryScore")?,
        item_stat_rows: count(conn, "SELECT COUNT(*) FROM ItemStat")?,
        item_response_rows: count(conn, "SELECT COUNT(*) FROM ItemResponse")?,
        attendance_rows: count(conn, "SELECT COUNT(*) FROM Attendance")?,
        ..Default::default()
    };
    let mut stmt = conn
        .prepare(
            "SELECT grade_year, grade, round, GROUP_CONCAT(DISTINCT subject), COUNT(DISTINCT student_id)
             FROM MockScore GROUP BY grade_year, grade, round ORDER BY grade_year, grade, round",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(RoundInfo {
                grade_year: r.get(0)?,
                grade: r.get(1)?,
                round: r.get(2)?,
                subjects: r.get::<_, Option<String>>(3)?.unwrap_or_default().split(',').map(|x| x.to_string()).filter(|x| !x.is_empty()).collect(),
                n_students: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    s.mock_rounds = rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT grade_year, grade, semester, GROUP_CONCAT(DISTINCT subject), COUNT(DISTINCT student_id)
             FROM SchoolScore GROUP BY grade_year, grade, semester ORDER BY grade_year, grade, semester",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SemesterInfo {
                grade_year: r.get(0)?,
                grade: r.get(1)?,
                semester: r.get(2)?,
                subjects: r.get::<_, Option<String>>(3)?.unwrap_or_default().split(',').map(|x| x.to_string()).filter(|x| !x.is_empty()).collect(),
                n_students: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    s.school_semesters = rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(s)
}

pub fn get_import_logs_impl(conn: &Connection) -> Result<Vec<ImportLogItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, dataset, file_name, row_count, inserted, updated, skipped, imported_at
             FROM ImportLog ORDER BY id DESC LIMIT 100",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ImportLogItem {
                id: r.get(0)?,
                dataset: r.get(1)?,
                file_name: r.get(2)?,
                row_count: r.get(3)?,
                inserted: r.get(4)?,
                updated: r.get(5)?,
                skipped: r.get(6)?,
                imported_at: r.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ── Tauri commands ───────────────────────────────────────────

#[tauri::command]
pub fn get_dataset_specs() -> Vec<DatasetSpec> {
    dataset_specs()
}

#[tauri::command]
pub fn suggest_mapping(dataset: String, headers: Vec<String>) -> Result<ImportMapping, String> {
    suggest_mapping_impl(&dataset, &headers)
}

#[tauri::command]
pub fn import_dataset(
    db: State<DbState>,
    dataset: String,
    rows: Vec<Row>,
    mapping: ImportMapping,
    file_name: String,
) -> Result<ImportResult, String> {
    let mut guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_mut().ok_or("DB not open")?;
    import_dataset_impl(conn, &dataset, &rows, &mapping, &file_name)
}

#[tauri::command]
pub fn clear_dataset(db: State<DbState>, dataset: String) -> Result<i64, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    clear_dataset_impl(conn, &dataset)
}

#[tauri::command]
pub fn get_dataset_summary(db: State<DbState>) -> Result<DatasetSummary, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    get_dataset_summary_impl(conn)
}

#[tauri::command]
pub fn get_import_logs(db: State<DbState>) -> Result<Vec<ImportLogItem>, String> {
    let guard = db.0.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not open")?;
    get_import_logs_impl(conn)
}
