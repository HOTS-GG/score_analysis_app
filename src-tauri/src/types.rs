use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── 학생 ─────────────────────────────────────────────────────
#[derive(Serialize, Clone, Debug)]
pub struct StudentItem {
    pub id: i64,
    pub anon_id: String,
    pub cohort: i64,
    pub grade: i64,
    pub class_no: i64,
    pub number: i64,
    pub name: String,
    pub status: String,
    pub exclude_reason: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct StudentInput {
    pub cohort: i64,
    pub grade: i64,
    pub class_no: i64,
    pub number: i64,
    #[serde(default)]
    pub name: String,
}

#[derive(Serialize, Default, Debug)]
pub struct BulkUpsertResult {
    pub inserted: i64,
    pub updated: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct ExclusionLogItem {
    pub id: i64,
    pub student_id: i64,
    pub anon_id: String,
    pub action: String,
    pub reason: Option<String>,
    pub logged_at: String,
}

// ── 데이터셋 / 가져오기 ───────────────────────────────────────
/// 표준 필드 정의 (컬럼 매핑 UI가 이 정보를 그대로 그린다)
#[derive(Serialize, Clone, Debug)]
pub struct FieldSpec {
    pub key: String,
    pub label: String,
    pub required: bool,
    pub kind: String, // int | float | text
    pub hint: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct DatasetSpec {
    pub id: String,
    pub label: String,
    pub desc: String,
    /// 행마다 반복되는 학생 식별 필드
    pub key_fields: Vec<FieldSpec>,
    /// 파일 전체에 공통으로 적용되는 값 (학년도, 회차 등)
    pub context_fields: Vec<FieldSpec>,
    /// 과목 블록(가로형)에서 과목마다 반복되는 필드
    pub subject_fields: Vec<FieldSpec>,
    /// 과목 블록 없이 행 단위로 들어오는 값 필드
    pub value_fields: Vec<FieldSpec>,
    pub supports_subjects: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct SubjectMapping {
    pub subject: String,
    #[serde(default)]
    pub columns: HashMap<String, Option<String>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct ImportMapping {
    /// 표준 key → 원본 컬럼명
    #[serde(default)]
    pub keys: HashMap<String, Option<String>>,
    /// context key → 값
    #[serde(default)]
    pub context: HashMap<String, serde_json::Value>,
    /// 가로형 과목 블록
    #[serde(default)]
    pub subjects: Vec<SubjectMapping>,
    /// 세로형: 표준 value key → 원본 컬럼명 (subject 컬럼 포함)
    #[serde(default)]
    pub values: HashMap<String, Option<String>>,
    /// wide | long
    #[serde(default = "default_mode")]
    pub mode: String,
    /// 명단에 없는 학생을 자동 생성할지
    #[serde(default)]
    pub create_missing: bool,
}

fn default_mode() -> String {
    "wide".to_string()
}

#[derive(Serialize, Default, Debug)]
pub struct ImportResult {
    pub row_count: i64,
    pub inserted: i64,
    pub updated: i64,
    pub skipped: i64,
    pub unmatched: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ImportLogItem {
    pub id: i64,
    pub dataset: String,
    pub file_name: String,
    pub row_count: i64,
    pub inserted: i64,
    pub updated: i64,
    pub skipped: i64,
    pub imported_at: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct RoundInfo {
    pub grade_year: i64,
    pub grade: i64,
    pub round: i64,
    pub subjects: Vec<String>,
    pub n_students: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct SemesterInfo {
    pub grade_year: i64,
    pub grade: i64,
    pub semester: i64,
    pub subjects: Vec<String>,
    pub n_students: i64,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct DatasetSummary {
    pub students_total: i64,
    pub students_active: i64,
    pub students_excluded: i64,
    pub mock_rows: i64,
    pub school_rows: i64,
    pub entry_rows: i64,
    pub item_stat_rows: i64,
    pub item_response_rows: i64,
    pub attendance_rows: i64,
    pub mock_rounds: Vec<RoundInfo>,
    pub school_semesters: Vec<SemesterInfo>,
}
