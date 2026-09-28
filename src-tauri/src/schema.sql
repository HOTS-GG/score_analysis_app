-- ================================================================
-- 설정
-- ================================================================
CREATE TABLE IF NOT EXISTS APP_CONFIGS
(
    config_key   TEXT PRIMARY KEY,
    config_value TEXT NOT NULL
);

-- ================================================================
-- 학생 (실명은 이 테이블에만. 분석 테이블은 id/anon_id 로만 참조)
-- ================================================================
CREATE TABLE IF NOT EXISTS Student
(
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    anon_id        TEXT    NOT NULL UNIQUE,
    cohort         INTEGER NOT NULL,                  -- 입학연도
    grade          INTEGER NOT NULL,                  -- 현재 학년
    class_no       INTEGER NOT NULL,                  -- 반
    number         INTEGER NOT NULL,                  -- 번호
    name           TEXT    NOT NULL DEFAULT '',
    status         TEXT    NOT NULL DEFAULT 'active', -- active | excluded
    exclude_reason TEXT,
    UNIQUE (cohort, class_no, number),
    CHECK (grade >= 1 AND class_no >= 1 AND number >= 1)
);

-- ================================================================
-- 모의고사(학력평가) — 전국 준거
-- ================================================================
CREATE TABLE IF NOT EXISTS MockScore
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES Student (id) ON DELETE CASCADE,
    grade_year INTEGER NOT NULL,                      -- 학년도
    grade      INTEGER NOT NULL,                      -- 학년
    round      INTEGER NOT NULL,                      -- 3 / 6 / 9 / 11
    subject    TEXT    NOT NULL,
    raw        REAL,
    std        REAL,
    pct        REAL,
    grade9     INTEGER,
    absent     INTEGER NOT NULL DEFAULT 0,
    blank_n    INTEGER,
    UNIQUE (student_id, grade_year, round, subject)
);
CREATE INDEX IF NOT EXISTS idx_mock_round ON MockScore (grade_year, round, subject);

-- ================================================================
-- 내신 — 교내 준거
-- ================================================================
CREATE TABLE IF NOT EXISTS SchoolScore
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id   INTEGER NOT NULL REFERENCES Student (id) ON DELETE CASCADE,
    grade_year   INTEGER NOT NULL,
    grade        INTEGER NOT NULL,
    semester     INTEGER NOT NULL,                    -- 1 / 2
    subject      TEXT    NOT NULL,
    achievement  TEXT,                                -- A~E
    grade5       INTEGER,
    raw          REAL,
    subject_mean REAL,
    subject_sd   REAL,
    enrolled_n   INTEGER,
    written      REAL,
    performance  REAL,
    rank         INTEGER,                            -- 석차 (수강자 중)
    credits      INTEGER,                            -- 단위수(학점)
    area         TEXT,                               -- 교과 (국어/수학/영어/사회/과학/체육/예술/기술·가정/정보/제2외국어/한문/교양)
    kind         TEXT,                               -- 편제 구분 (공통/일반/진로/융합/공동)
    ach_scale    INTEGER,                            -- 성취도 단계 (5 | 3 | 0=이수)
    dist_a       REAL, dist_b REAL, dist_c REAL, dist_d REAL, dist_e REAL,   -- 성취도별 분포비율 %
    note         TEXT,                               -- 비고 (학교 밖 교육 등)
    UNIQUE (student_id, grade_year, semester, subject)
);

-- 과목 편제 (학교 교육과정 편제표)
CREATE TABLE IF NOT EXISTS SubjectCatalog
(
    subject TEXT PRIMARY KEY,
    area    TEXT,
    kind    TEXT NOT NULL                             -- 공통 | 일반 | 진로 | 융합
);

-- ================================================================
-- 대입 결과 (졸업생. 이름은 저장하지 않는다)
-- ================================================================
CREATE TABLE IF NOT EXISTS AdmissionResult
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    adm_year    INTEGER NOT NULL,                     -- 대입 학년도 (2025 = 2025학년도 대입)
    class_no    INTEGER, number INTEGER,
    region      TEXT, univ TEXT NOT NULL,
    adm_type    TEXT,                                 -- 학생부위주(교과) / 학생부위주(종합) / 논술위주 / 실기·실적위주 ...
    adm_name    TEXT, detail TEXT,
    track       TEXT,                                 -- 인문 / 자연 / 예체능
    unit        TEXT, quota INTEGER, select_type TEXT, min_req TEXT,
    stage1      TEXT, final TEXT,                     -- 합격 / 충원합격 / 불합격
    registered  TEXT,
    grade_all   REAL, grade_main REAL, grade_kem REAL,  -- 전교과 / 국영수사·과 / 국영수 (9등급제 시절 값)
    grade_kor   REAL, grade_eng REAL, grade_math REAL, grade_soc REAL, grade_sci REAL,
    csat_kor    INTEGER, csat_math INTEGER, csat_eng INTEGER, csat_t1 INTEGER, csat_t2 INTEGER,
    csat_note   TEXT
);
CREATE INDEX IF NOT EXISTS idx_adm_univ ON AdmissionResult (univ, adm_type, track);

-- 강원 지역 대학별 전형별 지원·합격 범위 (교육청 공유 자료)
CREATE TABLE IF NOT EXISTS RegionalAdmission
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    source      TEXT,
    track       TEXT NOT NULL,                        -- 인문 / 자연
    category    TEXT NOT NULL,                        -- 교과 / 종합
    univ        TEXT NOT NULL, adm_name TEXT,
    apply_n     INTEGER, apply_best REAL, apply_worst REAL,
    pass_n      INTEGER, pass_best REAL, pass_worst REAL,
    pass_rate   REAL, cut70 REAL
);

-- 참고선 (편집 가능)
CREATE TABLE IF NOT EXISTS UnivReference
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    region     TEXT NOT NULL, univ TEXT NOT NULL, unit TEXT, adm_type TEXT,
    cut        REAL, note TEXT, sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_school_sem ON SchoolScore (grade_year, semester, subject);

-- ================================================================
-- 입학 시점 (고입내신 / 반배치고사)
-- ================================================================
CREATE TABLE IF NOT EXISTS EntryScore
(
    student_id       INTEGER PRIMARY KEY REFERENCES Student (id) ON DELETE CASCADE,
    mid_school_score REAL,
    placement_rank   REAL,
    placement_n      INTEGER
);

CREATE TABLE IF NOT EXISTS EntrySubjectScore
(
    student_id INTEGER NOT NULL REFERENCES Student (id) ON DELETE CASCADE,
    subject    TEXT    NOT NULL,
    raw        REAL,
    PRIMARY KEY (student_id, subject)
);

-- ================================================================
-- 문항 단위
-- ================================================================
CREATE TABLE IF NOT EXISTS ItemStat
(
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    grade_year            INTEGER NOT NULL,
    round                 INTEGER NOT NULL,
    subject               TEXT    NOT NULL,
    item_no               INTEGER NOT NULL,
    unit                  TEXT,
    item_type             TEXT,
    our_correct_rate      REAL,
    national_correct_rate REAL,
    answer_distribution   TEXT,                       -- JSON
    UNIQUE (grade_year, round, subject, item_no)
);

CREATE TABLE IF NOT EXISTS ItemResponse
(
    student_id INTEGER NOT NULL REFERENCES Student (id) ON DELETE CASCADE,
    grade_year INTEGER NOT NULL,
    round      INTEGER NOT NULL,
    subject    TEXT    NOT NULL,
    item_no    INTEGER NOT NULL,
    correct    INTEGER,
    chosen     TEXT,
    PRIMARY KEY (student_id, grade_year, round, subject, item_no)
);

-- ================================================================
-- 보조 (출결 / 과제)
-- ================================================================
CREATE TABLE IF NOT EXISTS Attendance
(
    student_id    INTEGER NOT NULL REFERENCES Student (id) ON DELETE CASCADE,
    grade_year    INTEGER NOT NULL,
    month         INTEGER NOT NULL,
    late          INTEGER NOT NULL DEFAULT 0,
    early_leave   INTEGER NOT NULL DEFAULT 0,
    absent        INTEGER NOT NULL DEFAULT 0,
    homework_rate REAL,
    PRIMARY KEY (student_id, grade_year, month)
);

-- ================================================================
-- 로그
-- ================================================================
CREATE TABLE IF NOT EXISTS ImportLog
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    dataset      TEXT    NOT NULL,
    file_name    TEXT    NOT NULL DEFAULT '',
    row_count    INTEGER NOT NULL DEFAULT 0,
    inserted     INTEGER NOT NULL DEFAULT 0,
    updated      INTEGER NOT NULL DEFAULT 0,
    skipped      INTEGER NOT NULL DEFAULT 0,
    mapping_json TEXT    NOT NULL DEFAULT '{}',
    imported_at  TEXT    NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 제외자 로그 (생존 편향 기록)
CREATE TABLE IF NOT EXISTS ExclusionLog
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES Student (id) ON DELETE CASCADE,
    action     TEXT    NOT NULL,                      -- exclude | restore
    reason     TEXT,
    logged_at  TEXT    NOT NULL DEFAULT (datetime('now', 'localtime'))
);
