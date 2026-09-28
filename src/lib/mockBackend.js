/**
 * 브라우저 단독 실행용 mock 백엔드 (화면 확인 전용).
 * Rust 커맨드와 같은 이름·형태로 응답한다. 실제 계산 로직은 넣지 않는다.
 */
import { mockAnalysis, MOCK_STUDENTS } from './mockAnalysis'

const THRESHOLD_DEFAULTS = {
  consecutive_drop_delta: -7, sharp_drop_delta: -15, subject_gap_min: 30, volatility_min: 15,
  consistency_abs_min: 20, z_resid_growth: 1, z_resid_growth_count: 2, z_resid_quiet_drop: -1,
  adaptation_entry_top_pct: 30, adaptation_sept_bottom_pct: 50,
  group_illusion_achievements: ['C', 'D', 'E'], group_illusion_grades: [1, 2],
  cutline_margin_max: 3, pos_delta_warn: 10, e_count_warn: 3, entry_move_warn: 20, area_gap_warn: 1.2, careless_easy_item_share: 40, easy_item_national_rate: 70, hard_item_national_rate: 30,
  item_gap_report: -15, misconception_choice_share: 40, sincerity_blank_ratio: 20, sincerity_same_choice_run: 5,
  min_n_regression: 100, min_r_regression: 0.4, min_n_equipercentile: 200, min_n_warn: 30,
  transition_cell_min: 10, skew_warn: 0.5, pct_clip_low: 0.5, pct_clip_high: 99.5, top_tier_pct: 96,
  grade5_cuts: [10, 34, 66, 90, 100], grade9_cuts: [4, 11, 23, 40, 60, 77, 89, 96, 100],
}

const f = (key, label, required, kind, hint = '') => ({ key, label, required, kind, hint })
const studentKeys = () => [
  f('class_no', '반', true, 'int'), f('number', '번호', true, 'int'),
  f('name', '이름', false, 'text', '매칭 확인용'), f('grade', '학년', false, 'int', '비우면 공통값 사용'),
]
const SPECS = [
  { id: 'students', label: '학생 명단', desc: '학년·반·번호·이름. 익명 ID는 자동 부여됩니다.',
    key_fields: [...studentKeys(), f('cohort', '입학연도', false, 'int', '비우면 공통값 사용')],
    context_fields: [f('cohort', '입학연도', true, 'int', '예: 2026'), f('grade', '학년', true, 'int', '1~3')],
    subject_fields: [], value_fields: [], supports_subjects: false },
  { id: 'mock', label: '모의고사(학력평가)', desc: '회차 × 과목별 원점수·표준점수·전국 백분위·등급. 결시는 빈칸으로 두세요 (0 아님).',
    key_fields: studentKeys(),
    context_fields: [f('grade_year', '학년도', true, 'int', '예: 2026'), f('grade', '학년', true, 'int', '1~3'), f('round', '회차(월)', true, 'int', '3 / 6 / 9 / 11')],
    subject_fields: [f('raw', '원점수', false, 'float'), f('std', '표준점수', false, 'float', '상위권 추적에 필요'), f('pct', '전국 백분위', false, 'float', '핵심 지표'), f('grade9', '등급(9등급)', false, 'int', '표시용'), f('absent', '결시 여부', false, 'text'), f('blank_n', '무응답 문항 수', false, 'int')],
    value_fields: [f('subject', '과목', true, 'text', '세로형일 때 과목 컬럼')], supports_subjects: true },
  { id: 'school', label: '내신', desc: '학기 × 과목별 성취도·등급(5등급)·원점수·과목평균·표준편차·수강자 수.',
    key_fields: studentKeys(),
    context_fields: [f('grade_year', '학년도', true, 'int'), f('grade', '학년', true, 'int'), f('semester', '학기', true, 'int', '1 / 2')],
    subject_fields: [f('achievement', '성취도(A~E)', false, 'text'), f('grade5', '등급(5등급)', false, 'int'), f('raw', '원점수', false, 'float'), f('subject_mean', '과목 평균', false, 'float'), f('subject_sd', '표준편차', false, 'float'), f('enrolled_n', '수강자 수', false, 'int'), f('written', '지필 점수', false, 'float'), f('performance', '수행 점수', false, 'float')],
    value_fields: [f('subject', '과목', true, 'text')], supports_subjects: true },
  { id: 'entry', label: '입학 시점(고입내신·반배치)', desc: '고입내신 환산점수, 반배치고사 교내 석차와 과목별 원점수.',
    key_fields: studentKeys(), context_fields: [f('grade', '학년', true, 'int')],
    subject_fields: [f('raw', '반배치 원점수', false, 'float')],
    value_fields: [f('mid_school_score', '고입내신 환산점수', false, 'float'), f('placement_rank', '반배치 교내 석차', false, 'float'), f('placement_n', '반배치 응시 인원', false, 'int')],
    supports_subjects: true },
  { id: 'item_stat', label: '문항 통계', desc: '문항별 교내 정답률·전국 정답률·선지별 응답 분포.',
    key_fields: [], context_fields: [f('grade_year', '학년도', true, 'int'), f('round', '회차(월)', true, 'int'), f('subject', '과목', true, 'text')],
    subject_fields: [], value_fields: [f('item_no', '문항 번호', true, 'int'), f('unit', '단원', false, 'text'), f('item_type', '유형', false, 'text'), f('our_correct_rate', '교내 정답률(%)', false, 'float'), f('national_correct_rate', '전국 정답률(%)', false, 'float'), f('answer_distribution', '선지별 응답 분포', false, 'text')],
    supports_subjects: false },
  { id: 'item_response', label: '문항 응답(학생별)', desc: '학생 × 문항 정오답 (세로형).',
    key_fields: studentKeys(), context_fields: [f('grade_year', '학년도', true, 'int'), f('grade', '학년', true, 'int'), f('round', '회차(월)', true, 'int'), f('subject', '과목', true, 'text')],
    subject_fields: [], value_fields: [f('item_no', '문항 번호', true, 'int'), f('correct', '정답 여부', false, 'text'), f('chosen', '선택 선지', false, 'text')], supports_subjects: false },
  { id: 'attendance', label: '출결·과제', desc: '월별 지각·조퇴·결석 횟수, 과제 제출률.',
    key_fields: studentKeys(), context_fields: [f('grade_year', '학년도', true, 'int'), f('grade', '학년', true, 'int')],
    subject_fields: [], value_fields: [f('month', '월', true, 'int'), f('late', '지각', false, 'int'), f('early_leave', '조퇴', false, 'int'), f('absent', '결석', false, 'int'), f('homework_rate', '과제 제출률(%)', false, 'float')], supports_subjects: false },
]

const state = {
  open: false, path: '', config: {}, globalConfig: {}, thresholds: { ...THRESHOLD_DEFAULTS },
  students: [], nextId: 1, logs: [], exclusion: [], summary: null,
  rows: { mock: 0, school: 0, entry: 0, item_stat: 0, item_response: 0, attendance: 0 },
  rounds: [], semesters: [],
}

function seedDemo() {
  state.students = MOCK_STUDENTS.map(s => ({ id: s.id, anon_id: s.anon_id, cohort: 2025, grade: s.grade, class_no: s.class_no, number: s.number, name: s.name, status: 'active', exclude_reason: null }))
  state.nextId = MOCK_STUDENTS.length + 1
  state.rows.school = 120
  state.semesters = [{ grade_year: 2025, grade: 1, semester: 1, subjects: ['공통국어1', '공통수학1'], n_students: 16 }, { grade_year: 2026, grade: 2, semester: 1, subjects: ['문학', '대수', '영어Ⅰ'], n_students: 16 }]
}

/** 브라우저 미리보기용 카드 (Rust 결과와 같은 형태의 예시값) */
function demoReport(st, gradeYear, semester) {
  const S = (subject, credits, raw, achievement, grade5, grade9_equiv, subject_mean, subject_sd, z, rank, enrolled_n, top_pct, relative_label, feedback) =>
    ({ subject, credits, raw, achievement, grade5, grade9_equiv, subject_mean, subject_sd, enrolled_n, z, rank, rank_estimated: false, top_pct, relative_label, feedback, excluded_from_avg: grade5 === null, radar: 100 - top_pct })
  const subjects = [
    S('문학', 4, 86.77, 'B', 2, 3, 67.2, 17.8, 1.1, 34, 258, 13.2, '평균보다 높음 / 상위 13.2%', '현재 2등급 내 8/62 위치입니다. 3.2점 정도 올리면 1등급 진입권에 가까워집니다 (정규 근사 추정).'),
    S('대수', 4, 87.19, 'A', 1, 1, 54.9, 23.4, 1.38, 10, 258, 3.9, '평균보다 높음 / 상위 3.9%', '현재 1등급 내 10/26 위치입니다. 현재 1등급 구간입니다.'),
    S('영어Ⅰ', 4, 94.36, 'A', 1, 2, 50.7, 25.4, 1.72, 13, 258, 5.0, '매우 우수 / 상위 5.0%', '현재 1등급 내 13/26 위치입니다. 현재 1등급 구간입니다.'),
    S('현대사회와 윤리', 3, 96.22, 'A', 1, 2, 69.9, 21.9, 1.2, 7, 103, 6.8, '평균보다 높음 / 상위 6.8%', '현재 1등급 하단에 있어 성적이 조금만 낮아져도 등급 하락 위험이 있습니다. 현재 1등급 구간입니다.'),
    S('경제', 3, 96.01, 'A', 1, 2, 73.9, 17.9, 1.24, 3, 45, 6.7, '평균보다 높음 / 상위 6.7%', '현재 1등급 하단에 있어 성적이 조금만 낮아져도 등급 하락 위험이 있습니다. 현재 1등급 구간입니다.'),
    S('사회문제 탐구', 3, 100, 'A', null, null, 97.3, 5.7, 0.47, 1, 80, 1.3, '평균권 / 상위 1.3%', '성취도(A~E)만 있는 과목이므로 등급 평균에서 제외했습니다. 수강자 중 상위 1.3% 부근입니다.'),
    S('정보', 3, 90.88, 'A', 1, 2, 70.4, 16.7, 1.23, 19, 258, 7.4, '평균보다 높음 / 상위 7.4%', '현재 1등급 하단에 있어 성적이 조금만 낮아져도 등급 하락 위험이 있습니다. 현재 1등급 구간입니다.'),
    S('일본어', 3, 97.7, 'A', 1, 1, 77.9, 12.7, 1.56, 5, 130, 3.8, '매우 우수 / 상위 3.8%', '현재 1등급 내 5/13 위치입니다. 현재 1등급 구간입니다.'),
    S('스포츠 생활1', 2, 92, 'A', null, null, 84.8, 15.8, 0.46, 104, 258, 40.3, '평균권 / 상위 40.3%', '성취도(A~E)만 있는 과목이므로 등급 평균에서 제외했습니다. 수강자 중 상위 40.3% 부근입니다.'),
  ]
  return { student_id: st.id, anon_id: st.anon_id, name: st.name, grade: st.grade, class_no: st.class_no, number: st.number, grade_year: gradeYear, semester, avg_grade5: 1.14, avg_grade9: 1.86, n_graded: 7, subjects, notes: ['평균 등급은 표시용 단순 평균입니다. 위치 판단은 석차백분율과 z점수로 합니다.'] }
}

function summary() {
  return {
    students_total: state.students.length,
    students_active: state.students.filter(s => s.status === 'active').length,
    students_excluded: state.students.filter(s => s.status === 'excluded').length,
    mock_rows: state.rows.mock, school_rows: state.rows.school, entry_rows: state.rows.entry,
    item_stat_rows: state.rows.item_stat, item_response_rows: state.rows.item_response, attendance_rows: state.rows.attendance,
    mock_rounds: state.rounds, school_semesters: state.semesters,
  }
}

const handlers = {
  new_project({ path }) { Object.assign(state, { open: true, path, config: {}, thresholds: { ...THRESHOLD_DEFAULTS }, logs: [], exclusion: [], rounds: [], semesters: [] }); state.rows = { mock: 0, school: 0, entry: 0, item_stat: 0, item_response: 0, attendance: 0 }; state.students = []; state.nextId = 1 },
  open_project({ path }) { state.open = true; state.path = path; if (!state.students.length) seedDemo() },
  backup_project() {}, migrate_schema() {}, check_and_update_app_version() { return null },
  get_config({ key }) { return state.config[key] ?? null },
  set_config({ key, value }) { state.config[key] = value },
  delete_config({ key }) { delete state.config[key] },
  get_global_config({ key }) { return state.globalConfig[key] ?? null },
  set_global_config({ key, value }) { state.globalConfig[key] = value },
  get_thresholds() { return { ...state.thresholds } },
  set_thresholds({ thresholds }) { state.thresholds = { ...thresholds }; return { ...state.thresholds } },
  reset_thresholds() { state.thresholds = { ...THRESHOLD_DEFAULTS }; return { ...state.thresholds } },
  get_students() { return [...state.students].sort((a, b) => a.cohort - b.cohort || a.grade - b.grade || a.class_no - b.class_no || a.number - b.number) },
  create_student({ student }) {
    if (state.students.some(s => s.cohort === student.cohort && s.class_no === student.class_no && s.number === student.number)) throw '같은 입학연도·반·번호의 학생이 이미 있습니다.'
    const id = state.nextId++
    const item = { id, anon_id: `S${String(id).padStart(5, '0')}`, ...student, status: 'active', exclude_reason: null }
    state.students.push(item); return item
  },
  update_student({ id, student }) { const s = state.students.find(x => x.id === id); if (!s) throw '학생을 찾을 수 없습니다.'; Object.assign(s, student) },
  delete_student({ id }) { state.students = state.students.filter(x => x.id !== id) },
  bulk_upsert_students({ students }) { let inserted = 0, updated = 0; for (const s of students) { const ex = state.students.find(x => x.cohort === s.cohort && x.class_no === s.class_no && x.number === s.number); if (ex) { Object.assign(ex, s); updated++ } else { handlers.create_student({ student: s }); inserted++ } } return { inserted, updated } },
  set_student_status({ id, status, reason }) {
    const s = state.students.find(x => x.id === id); if (!s) throw '학생을 찾을 수 없습니다.'
    if (status === 'excluded' && !reason) throw '제외 사유를 입력해야 합니다.'
    s.status = status; s.exclude_reason = status === 'excluded' ? reason : null
    state.exclusion.unshift({ id: state.exclusion.length + 1, student_id: id, anon_id: s.anon_id, action: status === 'excluded' ? 'exclude' : 'restore', reason: reason ?? null, logged_at: new Date().toISOString().slice(0, 19).replace('T', ' ') })
  },
  get_exclusion_log() { return [...state.exclusion] },
  get_dataset_specs() { return SPECS },
  suggest_mapping({ dataset, headers }) {
    const spec = SPECS.find(s => s.id === dataset)
    const find = (kws) => headers.find(h => kws.some(k => h.replace(/\s/g, '').includes(k))) ?? null
    const keys = {}
    for (const kf of spec.key_fields) keys[kf.key] = { class_no: find(['반']), number: find(['번호']), name: find(['이름', '성명']), grade: find(['학년']), cohort: find(['입학']) }[kf.key] ?? null
    const subjects = []
    if (spec.supports_subjects) {
      for (const subj of ['국어', '수학', '영어', '한국사', '통합사회', '통합과학']) {
        const cols = headers.filter(h => h.includes(subj))
        if (!cols.length) continue
        const columns = {}
        for (const sf of spec.subject_fields) {
          const kw = { raw: '원점수', std: '표준', pct: '백분위', grade9: '등급', grade5: '등급', achievement: '성취', subject_mean: '평균', subject_sd: '편차', enrolled_n: '수강' }[sf.key]
          columns[sf.key] = cols.find(c => kw && c.includes(kw)) ?? null
        }
        subjects.push({ subject: subj, columns })
      }
    }
    return { keys, context: {}, subjects, values: {}, mode: 'wide', create_missing: false }
  },
  import_dataset({ dataset, rows, mapping, fileName }) {
    const n = rows.length
    let inserted = 0, updated = 0, skipped = 0
    if (dataset === 'students') {
      for (const r of rows) {
        const cn = Number(r[mapping.keys.class_no]), no = Number(r[mapping.keys.number])
        if (!cn || !no) { skipped++; continue }
        const st = { cohort: Number(mapping.context.cohort), grade: Number(mapping.context.grade), class_no: cn, number: no, name: mapping.keys.name ? String(r[mapping.keys.name] ?? '') : '' }
        const res = handlers.bulk_upsert_students({ students: [st] }); inserted += res.inserted; updated += res.updated
      }
    } else {
      const per = mapping.mode === 'wide' && mapping.subjects?.length ? mapping.subjects.length : 1
      inserted = n * per; state.rows[dataset] = (state.rows[dataset] ?? 0) + inserted
      if (dataset === 'mock') state.rounds.push({ grade_year: Number(mapping.context.grade_year), grade: Number(mapping.context.grade), round: Number(mapping.context.round), subjects: mapping.subjects.map(s => s.subject), n_students: n })
      if (dataset === 'school') state.semesters.push({ grade_year: Number(mapping.context.grade_year), grade: Number(mapping.context.grade), semester: Number(mapping.context.semester), subjects: mapping.subjects.map(s => s.subject), n_students: n })
    }
    state.logs.unshift({ id: state.logs.length + 1, dataset, file_name: fileName, row_count: n, inserted, updated, skipped, imported_at: new Date().toISOString().slice(0, 19).replace('T', ' ') })
    return { row_count: n, inserted, updated, skipped, unmatched: [], errors: [] }
  },
  clear_dataset({ dataset }) { if (dataset === 'students') { const c = state.students.length; state.students = []; return c } const c = state.rows[dataset] ?? 0; state.rows[dataset] = 0; if (dataset === 'mock') state.rounds = []; if (dataset === 'school') state.semesters = []; return c },
  get_dataset_summary() { return summary() },
  get_import_logs() { return [...state.logs] },
  get_school_report({ studentId, gradeYear, semester }) {
    const st = state.students.find(s => s.id === studentId); if (!st) throw '학생을 찾을 수 없습니다.'
    return demoReport(st, gradeYear, semester)
  },
  get_class_school_reports({ gradeYear, semester, grade, classNo }) {
    return state.students.filter(s => s.grade === grade && s.class_no === classNo && s.status === 'active').map(s => demoReport(s, gradeYear, semester))
  },
  write_bytes_file() { return null },
}

export async function mockInvoke(cmd, args = {}) {
  const h = handlers[cmd] ?? (typeof mockAnalysis[cmd] === 'function' ? mockAnalysis[cmd].bind(mockAnalysis) : null)
  if (!h) throw new Error(`mock: 알 수 없는 커맨드 ${cmd}`)
  await new Promise(r => setTimeout(r, 30))
  return h(args)
}
