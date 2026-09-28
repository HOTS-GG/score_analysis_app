//! 나이스 「학교생활기록부 교과학습발달상황」 반별 출력물(xlsx) 파서 + 가져오기.
//! 레이아웃: 상단 제목 블록("N학년 M반"), 반복되는 머리글 행, 학생 첫 행에만 번호·성명, 학기는 바뀔 때만 표기.
//! 5단계 블록: 번호|성명|학년|학기|교과|과목|학점|원점수/과목평균|성취도|분포비율|석차등급|수강자수|비고
//! 3단계·이수 블록: 번호|성명|학년|학기|교과|과목|학점|성취도(또는 이수여부)|비고
use crate::analysis::data::normalize_area;
use crate::engine::parse_f64;
use crate::types::ImportResult;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct NeisRow {
    pub class_no: i64,
    pub number: i64,
    pub name: String,
    pub grade: i64,
    pub semester: i64,
    pub area: String,
    pub subject: String,
    pub credits: f64,
    pub raw: Option<f64>,
    pub subject_mean: Option<f64>,
    pub achievement: Option<String>,
    pub ach_scale: i64, // 5 | 3 | 0(이수)
    pub dist: [Option<f64>; 5],
    pub grade5: Option<i64>,
    pub enrolled_n: Option<i64>,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct NeisParsed {
    pub grade: i64,
    pub class_no: i64,
    pub rows: Vec<NeisRow>,
    pub students: usize,
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

fn is_blank(s: &str) -> bool {
    s.is_empty() || s == "·" || s == "nan" || s == "NaN"
}

fn parse_int(s: &str) -> Option<i64> {
    parse_f64(s).map(|v| v.round() as i64)
}

/// "N학년 M반" 찾기
pub fn find_class(grid: &[Vec<Value>]) -> Option<(i64, i64)> {
    for r in 0..grid.len().min(6) {
        for c in 0..grid[r].len().min(6) {
            let s = cell(grid, r, c).replace(' ', "");
            if let Some(gi) = s.find("학년") {
                if let Some(ci) = s.find('반') {
                    let g: Option<i64> = s[..gi].chars().rev().take_while(|ch| ch.is_ascii_digit()).collect::<String>().chars().rev().collect::<String>().parse().ok();
                    let cl: Option<i64> = s[gi + "학년".len()..ci].parse().ok();
                    if let (Some(g), Some(cl)) = (g, cl) {
                        return Some((g, cl));
                    }
                }
            }
        }
    }
    None
}

/// "A(3.0) B(13.6) C(24.9)D(23.8) E(34.7)" → [3.0, 13.6, 24.9, 23.8, 34.7]
pub fn parse_dist(s: &str) -> [Option<f64>; 5] {
    let mut out = [None; 5];
    for (i, letter) in ['A', 'B', 'C', 'D', 'E'].iter().enumerate() {
        if let Some(pos) = s.find(&format!("{letter}(")) {
            let rest = &s[pos + 2..];
            if let Some(end) = rest.find(')') {
                out[i] = rest[..end].trim().parse::<f64>().ok();
            }
        }
    }
    out
}

/// "64/63.8" → (Some(64), Some(63.8))
fn parse_raw_mean(s: &str) -> (Option<f64>, Option<f64>) {
    match s.split_once('/') {
        Some((a, b)) => (parse_f64(a), parse_f64(b)),
        None => (parse_f64(s), None),
    }
}

pub fn parse_grid(grid: &[Vec<Value>]) -> Result<NeisParsed, String> {
    let (grade_now, class_no) = find_class(grid).ok_or("반 정보('N학년 M반')를 찾을 수 없습니다. 나이스 교과학습발달상황 출력물이 맞는지 확인하세요.")?;
    let mut out = NeisParsed { grade: grade_now, class_no, ..Default::default() };
    let mut layout: Option<&str> = None; // "five" | "three"
    let mut number: Option<i64> = None;
    let mut name = String::new();
    let mut grade: Option<i64> = None;
    let mut semester: Option<i64> = None;
    let mut students = std::collections::HashSet::new();

    for r in 0..grid.len() {
        let c0 = cell(grid, r, 0).replace(' ', "");
        let c5 = cell(grid, r, 5);
        let c7 = cell(grid, r, 7);
        if c0 == "번호" {
            layout = Some(if c7.contains("원점수") { "five" } else { "three" });
            continue;
        }
        if c0.starts_with("이수학점") || layout.is_none() || is_blank(&c5) {
            continue;
        }
        // 학생 식별 (첫 행에만)
        let n0 = cell(grid, r, 0);
        if let Some(n) = parse_int(&n0) {
            number = Some(n);
            name = cell(grid, r, 1);
            grade = None;
            semester = None;
        }
        let Some(num) = number else { continue };
        if let Some(g) = parse_int(&cell(grid, r, 2)) {
            grade = Some(g);
        }
        if let Some(s) = parse_int(&cell(grid, r, 3)) {
            semester = Some(s);
        }
        let (Some(g), Some(sem)) = (grade, semester) else {
            out.warnings.push(format!("{}행: 학년/학기를 알 수 없어 건너뜀 ({c5})", r + 1));
            continue;
        };
        let area = normalize_area(&cell(grid, r, 4));
        let credits = parse_f64(&cell(grid, r, 6)).unwrap_or(0.0);
        let row = if layout == Some("five") {
            let (raw, mean) = parse_raw_mean(&c7);
            let ach = cell(grid, r, 8);
            let note = cell(grid, r, 12);
            let ach_v = if ach.len() == 1 && ('A'..='E').contains(&ach.chars().next().unwrap()) { Some(ach.clone()) } else { None };
            let scale = if ach_v.is_some() { 5 } else { 0 };
            NeisRow {
                class_no,
                number: num,
                name: name.clone(),
                grade: g,
                semester: sem,
                area,
                subject: c5.clone(),
                credits,
                raw,
                subject_mean: mean,
                achievement: ach_v,
                ach_scale: scale,
                dist: parse_dist(&cell(grid, r, 9)),
                grade5: parse_int(&cell(grid, r, 10)),
                enrolled_n: parse_int(&cell(grid, r, 11)),
                note: if is_blank(&note) { None } else { Some(note) },
            }
        } else {
            let ach = c7.clone();
            let note = cell(grid, r, 8);
            let (ach_v, scale) = match ach.as_str() {
                "A" | "B" | "C" => (Some(ach.clone()), 3),
                "P" | "이수" => (Some("P".to_string()), 0),
                _ => (None, 0),
            };
            NeisRow {
                class_no,
                number: num,
                name: name.clone(),
                grade: g,
                semester: sem,
                area,
                subject: c5.clone(),
                credits,
                raw: None,
                subject_mean: None,
                achievement: ach_v,
                ach_scale: scale,
                dist: [None; 5],
                grade5: None,
                enrolled_n: None,
                note: if is_blank(&note) { None } else { Some(note) },
            }
        };
        students.insert(num);
        out.rows.push(row);
    }
    out.students = students.len();
    if out.rows.is_empty() {
        return Err("성적 행을 하나도 찾지 못했습니다.".into());
    }
    Ok(out)
}

/// 파싱 결과를 DB에 저장. grade_year = 파일의 현재 학년 기준 학년도. 이전 학년 기록은 학년 차이만큼 학년도를 뺀다.
pub fn import_parsed(conn: &mut Connection, p: &NeisParsed, grade_year: i64, file_name: &str, create_missing: bool) -> Result<ImportResult, String> {
    let cohort = grade_year - p.grade + 1;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut res = ImportResult { row_count: p.rows.len() as i64, ..Default::default() };
    let mut unmatched = std::collections::BTreeSet::new();
    for row in &p.rows {
        let sid: Option<i64> = tx
            .query_row("SELECT id FROM Student WHERE cohort = ?1 AND class_no = ?2 AND number = ?3", params![cohort, row.class_no, row.number], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        let sid = match sid {
            Some(id) => id,
            None if create_missing => {
                let anon = crate::commands::student::next_anon_id(&tx)?;
                tx.execute(
                    "INSERT INTO Student (anon_id, cohort, grade, class_no, number, name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![anon, cohort, p.grade, row.class_no, row.number, row.name],
                )
                .map_err(|e| e.to_string())?;
                tx.last_insert_rowid()
            }
            None => {
                unmatched.insert(format!("{}학년 {}반 {}번 {}", p.grade, row.class_no, row.number, row.name));
                res.skipped += 1;
                continue;
            }
        };
        let gy = grade_year - (p.grade - row.grade);
        let exists: Option<i64> = tx
            .query_row("SELECT 1 FROM SchoolScore WHERE student_id=?1 AND grade_year=?2 AND semester=?3 AND subject=?4", params![sid, gy, row.semester, row.subject], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO SchoolScore (student_id, grade_year, grade, semester, subject, achievement, grade5, raw, subject_mean, enrolled_n, credits, area, kind, ach_scale, dist_a, dist_b, dist_c, dist_d, dist_e, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, (SELECT kind FROM SubjectCatalog WHERE subject = ?5), ?13, ?14, ?15, ?16, ?17, ?18, ?19)
             ON CONFLICT(student_id, grade_year, semester, subject) DO UPDATE SET
               grade=excluded.grade, achievement=excluded.achievement, grade5=excluded.grade5, raw=excluded.raw, subject_mean=excluded.subject_mean,
               enrolled_n=excluded.enrolled_n, credits=excluded.credits, area=excluded.area, kind=COALESCE(excluded.kind, kind), ach_scale=excluded.ach_scale,
               dist_a=excluded.dist_a, dist_b=excluded.dist_b, dist_c=excluded.dist_c, dist_d=excluded.dist_d, dist_e=excluded.dist_e, note=excluded.note",
            params![
                sid, gy, row.grade, row.semester, row.subject, row.achievement, row.grade5, row.raw, row.subject_mean, row.enrolled_n, row.credits as i64,
                row.area, row.ach_scale, row.dist[0], row.dist[1], row.dist[2], row.dist[3], row.dist[4], row.note
            ],
        )
        .map_err(|e| e.to_string())?;
        if exists.is_some() { res.updated += 1 } else { res.inserted += 1 }
    }
    res.unmatched = unmatched.into_iter().collect();
    res.errors = p.warnings.clone();
    tx.execute(
        "INSERT INTO ImportLog (dataset, file_name, row_count, inserted, updated, skipped, mapping_json) VALUES ('neis_school', ?1, ?2, ?3, ?4, ?5, ?6)",
        params![file_name, res.row_count, res.inserted, res.updated, res.skipped, format!("{{\"grade\":{},\"class_no\":{},\"grade_year\":{}}}", p.grade, p.class_no, grade_year)],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(res)
}
