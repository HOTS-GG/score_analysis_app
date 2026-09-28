//! 학년 단위 데이터 적재 + 공통 유틸. 모든 분석 모듈이 이 구조체 위에서 계산한다.
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;

pub const MAIN_AREAS: [&str; 5] = ["국어", "수학", "영어", "사회", "과학"];
pub const RADAR_AXES: [&str; 6] = ["국어", "수학", "영어", "사회", "과학", "기타"];

/// 나이스 교과명 → 표준 교과
pub fn normalize_area(s: &str) -> String {
    let t = s.trim().replace(' ', "");
    if t.is_empty() {
        return String::new();
    }
    if t.starts_with("사회") {
        return "사회".into();
    }
    if t.starts_with("기술") || t.contains("정보") {
        return "기술·가정/정보".into();
    }
    if t.starts_with("제2외국어") || t.contains("한문") {
        return "제2외국어/한문".into();
    }
    t
}

pub fn is_main_area(area: &str) -> bool {
    MAIN_AREAS.contains(&area)
}

/// 공통과목 판정 (경희대·동국대 산식용)
pub fn is_common_subject(subject: &str, kind: Option<&str>) -> bool {
    if kind == Some("공통") {
        return true;
    }
    let s = subject.replace(' ', "");
    s.starts_with("공통") || s.starts_with("한국사") || s.starts_with("통합사회") || s.starts_with("통합과학") || s.starts_with("과학탐구실험")
}

#[derive(Clone, Debug, Serialize)]
pub struct StudentBase {
    pub id: i64,
    pub anon_id: String,
    pub name: String,
    pub grade: i64,
    pub class_no: i64,
    pub number: i64,
    pub status: String,
}

impl StudentBase {
    pub fn hakbun(&self) -> String {
        format!("{}{:02}{:02}", self.grade, self.class_no, self.number)
    }
}

/// 내신 한 행 (SchoolScore)
#[derive(Clone, Debug, Serialize)]
pub struct Rec {
    pub grade_year: i64,
    pub grade: i64,
    pub semester: i64,
    pub term: String, // "1-1"
    pub subject: String,
    pub area: String,
    pub kind: String,
    pub credits: f64,
    pub raw: Option<f64>,
    pub subject_mean: Option<f64>,
    pub subject_sd: Option<f64>,
    pub achievement: Option<String>,
    pub ach_scale: i64,
    pub grade5: Option<i64>,
    pub rank: Option<i64>,
    pub enrolled_n: Option<i64>,
    pub dist: [Option<f64>; 5],
    pub note: Option<String>,
}

impl Rec {
    pub fn in_term(&self, term: &str) -> bool {
        term == "ALL" || self.term == term
    }
    pub fn in_scope(&self, scope: &str) -> bool {
        scope == "all" || is_main_area(&self.area)
    }
    pub fn deviation(&self) -> Option<f64> {
        match (self.raw, self.subject_mean) {
            (Some(r), Some(m)) => Some(r - m),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct EntryInfo {
    pub mid_school_score: Option<f64>,
    pub placement_rank: Option<f64>,
    pub placement_n: Option<i64>,
    pub subjects: HashMap<String, f64>,
    /// 입학 당시 상위 % (작을수록 상위)
    pub top_pct: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MockRec {
    pub grade_year: i64,
    pub grade: i64,
    pub round: i64,
    pub subject: String,
    pub raw: Option<f64>,
    pub std: Option<f64>,
    pub pct: Option<f64>,
    pub grade9: Option<i64>,
    pub absent: bool,
}

pub struct GradeData {
    pub grade_year: i64,
    pub grade: i64,
    pub students: Vec<StudentBase>,
    pub recs: HashMap<i64, Vec<Rec>>,
    pub entry: HashMap<i64, EntryInfo>,
    pub mock: HashMap<i64, Vec<MockRec>>,
    /// 학년 안에 존재하는 학기 키, 오름차순 ("1-1","1-2","2-1")
    pub terms: Vec<String>,
}

impl GradeData {
    pub fn recs_of(&self, sid: i64) -> &[Rec] {
        self.recs.get(&sid).map(|v| v.as_slice()).unwrap_or(&[])
    }
    pub fn student(&self, sid: i64) -> Option<&StudentBase> {
        self.students.iter().find(|s| s.id == sid)
    }
    /// 직전 학기 쌍 (cur, prev). term == "ALL" 이면 마지막 두 학기.
    pub fn prev_term(&self, term: &str) -> Option<(String, String)> {
        if self.terms.len() < 2 {
            return None;
        }
        if term == "ALL" {
            let n = self.terms.len();
            return Some((self.terms[n - 1].clone(), self.terms[n - 2].clone()));
        }
        let i = self.terms.iter().position(|t| t == term)?;
        if i == 0 {
            None
        } else {
            Some((self.terms[i].clone(), self.terms[i - 1].clone()))
        }
    }
}

pub fn term_label(t: &str) -> String {
    if t == "ALL" {
        "전체 누적".into()
    } else {
        match t.split_once('-') {
            Some((g, s)) => format!("{g}학년 {s}학기"),
            None => t.to_string(),
        }
    }
}

/// 학년 데이터 전체 적재 (활성 학생만). grade_year·grade 는 "현재" 학년 기준이며, 이전 학년 내신도 함께 싣는다.
pub fn load_grade(conn: &Connection, grade_year: i64, grade: i64) -> Result<GradeData, String> {
    let cohort = grade_year - grade + 1;
    let mut stmt = conn
        .prepare("SELECT id, anon_id, name, grade, class_no, number, status FROM Student WHERE cohort = ?1 AND status = 'active' ORDER BY class_no, number")
        .map_err(|e| e.to_string())?;
    let students: Vec<StudentBase> = stmt
        .query_map([cohort], |r| {
            Ok(StudentBase { id: r.get(0)?, anon_id: r.get(1)?, name: r.get(2)?, grade: r.get(3)?, class_no: r.get(4)?, number: r.get(5)?, status: r.get(6)? })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let mut recs: HashMap<i64, Vec<Rec>> = HashMap::new();
    let mut terms: Vec<String> = Vec::new();
    {
        let mut st = conn
            .prepare(
                "SELECT s.student_id, s.grade_year, s.grade, s.semester, s.subject, COALESCE(s.area, c.area, ''), COALESCE(s.kind, c.kind, '미분류'),
                        COALESCE(s.credits, 0), s.raw, s.subject_mean, s.subject_sd, s.achievement, COALESCE(s.ach_scale, CASE WHEN s.grade5 IS NULL THEN 3 ELSE 5 END),
                        s.grade5, s.rank, s.enrolled_n, s.dist_a, s.dist_b, s.dist_c, s.dist_d, s.dist_e, s.note
                 FROM SchoolScore s JOIN Student st ON st.id = s.student_id LEFT JOIN SubjectCatalog c ON c.subject = s.subject
                 WHERE st.cohort = ?1 AND st.status = 'active' ORDER BY s.grade, s.semester, s.id",
            )
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([cohort], |r| {
                let g: i64 = r.get(2)?;
                let sem: i64 = r.get(3)?;
                Ok((
                    r.get::<_, i64>(0)?,
                    Rec {
                        grade_year: r.get(1)?,
                        grade: g,
                        semester: sem,
                        term: format!("{g}-{sem}"),
                        subject: r.get(4)?,
                        area: r.get(5)?,
                        kind: r.get(6)?,
                        credits: r.get::<_, f64>(7)?,
                        raw: r.get(8)?,
                        subject_mean: r.get(9)?,
                        subject_sd: r.get(10)?,
                        achievement: r.get(11)?,
                        ach_scale: r.get(12)?,
                        grade5: r.get(13)?,
                        rank: r.get(14)?,
                        enrolled_n: r.get(15)?,
                        dist: [r.get(16)?, r.get(17)?, r.get(18)?, r.get(19)?, r.get(20)?],
                        note: r.get(21)?,
                    },
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (sid, rec) = row.map_err(|e| e.to_string())?;
            if !terms.contains(&rec.term) {
                terms.push(rec.term.clone());
            }
            recs.entry(sid).or_default().push(rec);
        }
    }
    terms.sort();

    let mut entry: HashMap<i64, EntryInfo> = HashMap::new();
    {
        let mut st = conn
            .prepare("SELECT e.student_id, e.mid_school_score, e.placement_rank, e.placement_n FROM EntryScore e JOIN Student s ON s.id = e.student_id WHERE s.cohort = ?1")
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([cohort], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<f64>>(1)?, r.get::<_, Option<f64>>(2)?, r.get::<_, Option<i64>>(3)?)))
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (sid, mid, rank, n) = row.map_err(|e| e.to_string())?;
            let top = match (rank, n) {
                (Some(rk), Some(nn)) if nn > 0 => Some(rk / nn as f64 * 100.0),
                _ => mid, // 고입내신 석차백분율 (이미 %)
            };
            entry.insert(sid, EntryInfo { mid_school_score: mid, placement_rank: rank, placement_n: n, subjects: HashMap::new(), top_pct: top });
        }
        let mut st2 = conn
            .prepare("SELECT e.student_id, e.subject, e.raw FROM EntrySubjectScore e JOIN Student s ON s.id = e.student_id WHERE s.cohort = ?1")
            .map_err(|e| e.to_string())?;
        let rows = st2
            .query_map([cohort], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<f64>>(2)?)))
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (sid, subj, raw) = row.map_err(|e| e.to_string())?;
            if let (Some(e), Some(v)) = (entry.get_mut(&sid), raw) {
                e.subjects.insert(subj, v);
            }
        }
    }

    let mut mock: HashMap<i64, Vec<MockRec>> = HashMap::new();
    {
        let mut st = conn
            .prepare(
                "SELECT m.student_id, m.grade_year, m.grade, m.round, m.subject, m.raw, m.std, m.pct, m.grade9, m.absent
                 FROM MockScore m JOIN Student s ON s.id = m.student_id WHERE s.cohort = ?1 ORDER BY m.grade_year, m.round",
            )
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([cohort], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    MockRec { grade_year: r.get(1)?, grade: r.get(2)?, round: r.get(3)?, subject: r.get(4)?, raw: r.get(5)?, std: r.get(6)?, pct: r.get(7)?, grade9: r.get(8)?, absent: r.get::<_, i64>(9)? != 0 },
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (sid, m) = row.map_err(|e| e.to_string())?;
            mock.entry(sid).or_default().push(m);
        }
    }

    Ok(GradeData { grade_year, grade, students, recs, entry, mock, terms })
}

/// 학년도·학년 목록 (내신 또는 모의고사가 있는 활성 코호트)
pub fn available_contexts(conn: &Connection) -> Result<Vec<(i64, i64, Vec<i64>)>, String> {
    let mut stmt = conn
        .prepare("SELECT cohort, grade, GROUP_CONCAT(DISTINCT class_no) FROM Student WHERE status='active' GROUP BY cohort, grade ORDER BY grade")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, Option<String>>(2)?.unwrap_or_default())))
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        let (cohort, grade, classes) = row.map_err(|e| e.to_string())?;
        let mut cl: Vec<i64> = classes.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        cl.sort();
        out.push((cohort + grade - 1, grade, cl));
    }
    Ok(out)
}

pub fn load_catalog(conn: &Connection) -> Result<Vec<(String, Option<String>, String)>, String> {
    let mut stmt = conn.prepare("SELECT subject, area, kind FROM SubjectCatalog ORDER BY subject").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

pub fn upsert_catalog(conn: &Connection, items: &[(String, Option<String>, String)]) -> Result<usize, String> {
    let mut n = 0;
    for (subject, area, kind) in items {
        conn.execute(
            "INSERT INTO SubjectCatalog (subject, area, kind) VALUES (?1, ?2, ?3) ON CONFLICT(subject) DO UPDATE SET area=excluded.area, kind=excluded.kind",
            params![subject.trim(), area, kind.trim()],
        )
        .map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok(n)
}
