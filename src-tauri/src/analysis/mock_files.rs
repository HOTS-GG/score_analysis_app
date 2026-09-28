//! 모의고사(학력평가) 파일 자동 인식 가져오기.
//! 두 가지 익숙한 양식을 매핑 없이 읽는다.
//!   (A) 학력평가 성적 종합 분석 파일의 회차 시트: 머리글에 "학번·이름"과 "국어|표준점수·백분위·등급" 같은 열. (xlsb 는 xlsx 로 저장해서 넣는다)
//!   (B) 학급 가채점 시트: 학번·이름·국어·수학·영어·한국사·통사·통과(·합계) 원점수.
use crate::commands::dataset::normalize_subject;
use crate::engine::parse_f64;
use crate::types::ImportResult;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct MockRow {
    pub grade: i64,
    pub class_no: i64,
    pub number: i64,
    pub name: String,
    pub subject: String,
    pub raw: Option<f64>,
    pub std: Option<f64>,
    pub pct: Option<f64>,
    pub grade9: Option<i64>,
    pub absent: bool,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct MockParsed {
    /// "analysis" (종합분석 시트) | "raw" (가채점)
    pub layout: String,
    pub grade: Option<i64>,
    pub round: Option<i64>,
    pub students: usize,
    pub subjects: Vec<String>,
    pub rows: Vec<MockRow>,
    pub warnings: Vec<String>,
}

fn cell(grid: &[Vec<Value>], r: usize, c: usize) -> String {
    grid.get(r)
        .and_then(|row| row.get(c))
        .map(|v| match v {
            Value::Null => String::new(),
            Value::String(s) => s.trim().to_string(),
            Value::Number(n) => n.to_string(),
            other => other.to_string(),
        })
        .unwrap_or_default()
}

const ABSENT: [&str; 6] = ["미응시", "결시", "병결", "도움반", "학적 변동", "미응시(도움반)"];
fn is_absent(s: &str) -> bool {
    ABSENT.iter().any(|t| s.contains(t))
}
fn is_pending(s: &str) -> bool {
    s.contains("취합중") || s.contains('~')
}

fn parse_hakbun(s: &str) -> Option<(i64, i64, i64)> {
    let t = s.trim().trim_end_matches(".0");
    if t.len() != 5 || !t.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((t[..1].parse().ok()?, t[1..3].parse().ok()?, t[3..5].parse().ok()?))
}

/// 머리글 "국어\n표준점수" → (국어, std). 집계 열(평균·순위·석차·괄호)은 None.
fn header_field(h: &str) -> Option<(String, &'static str)> {
    let t = h.replace('\n', " ").replace('|', " ");
    let t = t.trim();
    if t.is_empty() || t.contains('(') || t.contains("평균") || t.contains("순위") || t.contains("석차") || t.contains("동석차") {
        return None;
    }
    let field = if t.contains("표준점수") {
        "std"
    } else if t.contains("백분위") {
        "pct"
    } else if t.contains("원점수") {
        "raw"
    } else if t.contains("등급") {
        "grade9"
    } else {
        return None;
    };
    let subj = t.replace("표준점수", "").replace("백분위", "").replace("원점수", "").replace("등급", "").replace('/', "").trim().to_string();
    let subj = subj.replace("한문", "");
    let subj = normalize_subject(subj.trim());
    if subj.is_empty() || subj.contains("국영수") || subj.contains("국수") {
        return None;
    }
    Some((subj, field))
}

fn find_header(grid: &[Vec<Value>]) -> Option<usize> {
    (0..grid.len().min(12)).find(|&r| {
        let row: Vec<String> = (0..grid[r].len()).map(|c| cell(grid, r, c)).collect();
        row.iter().any(|x| x.replace(' ', "") == "학번") && row.iter().any(|x| x.replace(' ', "") == "이름")
    })
}

fn detect_round(grid: &[Vec<Value>]) -> Option<i64> {
    for r in 0..grid.len().min(6) {
        for c in 0..grid[r].len().min(12) {
            let s = cell(grid, r, c);
            if let Some(i) = s.find("월") {
                let digits: String = s[..i].chars().rev().take_while(|ch| ch.is_ascii_digit()).collect::<String>().chars().rev().collect();
                if let Ok(m) = digits.parse::<i64>() {
                    if matches!(m, 3 | 5 | 6 | 7 | 9 | 10 | 11) {
                        return Some(m);
                    }
                }
            }
        }
    }
    None
}

pub fn parse_mock_grid(grid: &[Vec<Value>]) -> Result<MockParsed, String> {
    let hr = find_header(grid).ok_or("'학번'과 '이름' 머리글이 있는 행을 찾지 못했습니다. 학력평가 종합분석 시트(xlsx로 저장) 또는 가채점 시트인지 확인하세요.")?;
    let ncol = grid[hr].len();
    let headers: Vec<String> = (0..ncol).map(|c| cell(grid, hr, c)).collect();
    let hakbun_cols: Vec<usize> = headers.iter().enumerate().filter(|(_, h)| h.replace(' ', "") == "학번").map(|(i, _)| i).collect();
    let name_col = headers.iter().position(|h| h.replace(' ', "") == "이름").unwrap();
    let fields: Vec<(usize, String, &str)> = headers.iter().enumerate().filter_map(|(i, h)| header_field(h).map(|(s, f)| (i, s, f))).collect();
    let (layout, fields) = if fields.iter().any(|f| f.2 == "std" || f.2 == "pct") {
        ("analysis", fields)
    } else {
        // 가채점: 과목명만 있는 열을 원점수로
        let raw_fields: Vec<(usize, String, &str)> = headers
            .iter()
            .enumerate()
            .filter_map(|(i, h)| {
                let s = normalize_subject(h);
                if matches!(s.as_str(), "국어" | "수학" | "영어" | "한국사" | "통합사회" | "통합과학" | "제2외국어") { Some((i, s, "raw")) } else { None }
            })
            .collect();
        ("raw", raw_fields)
    };
    if fields.is_empty() {
        return Err("과목 열을 찾지 못했습니다.".into());
    }
    let mut out = MockParsed { layout: layout.into(), round: detect_round(grid), ..Default::default() };
    let mut students = std::collections::HashSet::new();
    for r in hr + 1..grid.len() {
        let mut key = None;
        for &hc in &hakbun_cols {
            if let Some(k) = parse_hakbun(&cell(grid, r, hc)) {
                key = Some(k);
                break;
            }
        }
        let Some((g, c, n)) = key else { continue };
        let name = cell(grid, r, name_col);
        if name.is_empty() || name == "0" {
            continue;
        }
        out.grade.get_or_insert(g);
        students.insert((c, n));
        // 과목별로 모아서 한 행씩
        let mut per: std::collections::BTreeMap<String, MockRow> = std::collections::BTreeMap::new();
        let mut row_absent = false;
        let mut row_pending = false;
        for (i, subj, f) in &fields {
            let v = cell(grid, r, *i);
            if is_absent(&v) {
                row_absent = true;
            }
            if is_pending(&v) {
                row_pending = true;
            }
            let e = per.entry(subj.clone()).or_insert_with(|| MockRow { grade: g, class_no: c, number: n, name: name.clone(), subject: subj.clone(), raw: None, std: None, pct: None, grade9: None, absent: false });
            let x = if v.starts_with("0x") { None } else { parse_f64(&v) };
            match *f {
                "raw" => e.raw = x,
                "std" => e.std = x,
                "pct" => e.pct = x,
                _ => e.grade9 = x.map(|y| y.round() as i64),
            }
            if is_absent(&v) {
                e.absent = true;
            }
        }
        if row_pending {
            out.warnings.push(format!("{c}반 {n}번 {name}: 점수 미확정(취합중) — 건너뜀"));
            continue;
        }
        for (_, mut m) in per {
            let has = m.raw.is_some() || m.std.is_some() || m.pct.is_some() || m.grade9.is_some();
            if !has && !(m.absent || row_absent) {
                continue;
            }
            // 선택 응시 과목(제2외국어)은 값이 있을 때만 기록
            if !has && m.subject == "제2외국어" {
                continue;
            }
            if let Some(p) = m.pct {
                if !(0.0..=100.0).contains(&p) {
                    out.warnings.push(format!("{c}반 {n}번 {}: 백분위 범위 밖({p})", m.subject));
                    continue;
                }
            }
            if !has {
                m.absent = true;
            }
            if !out.subjects.contains(&m.subject) {
                out.subjects.push(m.subject.clone());
            }
            out.rows.push(m);
        }
    }
    out.students = students.len();
    if out.rows.is_empty() {
        return Err("학생 성적 행을 찾지 못했습니다 (학번이 5자리 숫자인지 확인).".into());
    }
    Ok(out)
}

pub fn import_mock_parsed(conn: &mut Connection, p: &MockParsed, grade_year: i64, round: i64, file_name: &str, create_missing: bool) -> Result<ImportResult, String> {
    if !matches!(round, 3 | 5 | 6 | 7 | 9 | 10 | 11) {
        return Err("회차(월)는 3·5·6·7·9·10·11 중 하나여야 합니다.".into());
    }
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut res = ImportResult { row_count: p.rows.len() as i64, ..Default::default() };
    let mut unmatched = std::collections::BTreeSet::new();
    for m in &p.rows {
        let cohort = grade_year - m.grade + 1;
        let sid: Option<i64> = tx
            .query_row("SELECT id FROM Student WHERE cohort = ?1 AND class_no = ?2 AND number = ?3", params![cohort, m.class_no, m.number], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        let sid = match sid {
            Some(id) => id,
            None if create_missing => {
                let anon = crate::commands::student::next_anon_id(&tx)?;
                tx.execute("INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![anon, cohort, m.grade, m.class_no, m.number, m.name]).map_err(|e| e.to_string())?;
                tx.last_insert_rowid()
            }
            None => {
                unmatched.insert(format!("{}학년 {}반 {}번 {}", m.grade, m.class_no, m.number, m.name));
                res.skipped += 1;
                continue;
            }
        };
        let exists: Option<i64> = tx
            .query_row("SELECT 1 FROM MockScore WHERE student_id=?1 AND grade_year=?2 AND round=?3 AND subject=?4", params![sid, grade_year, round, m.subject], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO MockScore (student_id, grade_year, grade, round, subject, raw, std, pct, grade9, absent)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(student_id, grade_year, round, subject) DO UPDATE SET grade=excluded.grade,
               raw=COALESCE(excluded.raw, raw), std=COALESCE(excluded.std, std), pct=COALESCE(excluded.pct, pct), grade9=COALESCE(excluded.grade9, grade9), absent=excluded.absent",
            params![sid, grade_year, m.grade, round, m.subject, m.raw, m.std, m.pct, m.grade9, m.absent as i64],
        )
        .map_err(|e| e.to_string())?;
        if exists.is_some() { res.updated += 1 } else { res.inserted += 1 }
    }
    res.unmatched = unmatched.into_iter().collect();
    res.errors = p.warnings.clone();
    tx.execute(
        "INSERT INTO ImportLog (dataset, file_name, row_count, inserted, updated, skipped, mapping_json) VALUES ('mock', ?1, ?2, ?3, ?4, ?5, ?6)",
        params![file_name, res.row_count, res.inserted, res.updated, res.skipped, format!("{{\"layout\":\"{}\",\"grade_year\":{grade_year},\"round\":{round}}}", p.layout)],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(res)
}
