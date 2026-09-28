/**
 * 브라우저 미리보기용 분석 mock. Rust 커맨드와 같은 형태의 응답을 만든다 (수치는 예시).
 * 실제 계산 로직은 Rust(src-tauri/src/analysis)에만 있다.
 */
const AXES = ['국어', '수학', '영어', '사회', '과학', '기타']
const SUBJ = {
  '1-1': [['공통국어1', '국어', '공통', 4], ['공통수학1', '수학', '공통', 4], ['공통영어1', '영어', '공통', 4], ['한국사1', '사회', '공통', 3], ['통합사회1', '사회', '공통', 4], ['통합과학1', '과학', '공통', 4], ['체육1', '체육', '일반', 2, 3]],
  '1-2': [['공통국어2', '국어', '공통', 4], ['공통수학2', '수학', '공통', 4], ['공통영어2', '영어', '공통', 4], ['한국사2', '사회', '공통', 3], ['통합사회2', '사회', '공통', 4], ['통합과학2', '과학', '공통', 4], ['체육2', '체육', '일반', 2, 3]],
  '2-1': [['문학', '국어', '일반', 4], ['대수', '수학', '일반', 4], ['영어Ⅰ', '영어', '일반', 4], ['세계시민과 지리', '사회', '일반', 3], ['사회문제 탐구', '사회', '융합', 3, 0], ['정보', '기술·가정/정보', '일반', 3], ['일본어', '제2외국어/한문', '일반', 3], ['스포츠 생활1', '체육', '일반', 2, 3]],
}
const NAMES = ['한가람', '윤슬기', '도하람', '문빛나', '박서준', '이도윤', '정하늘', '최민준']
const seed = (n) => { let x = Math.sin(n * 9301 + 49297) * 233280; return x - Math.floor(x) }

function makeStudents() {
  const out = []
  let id = 1
  for (const cls of [1, 2]) for (let n = 1; n <= 8; n++) {
    const ability = 0.15 + 0.7 * seed(id * 7) // 0~1, 낮을수록 상위
    out.push({ id, anon_id: `S${String(id).padStart(5, '0')}`, name: NAMES[n - 1] + (cls === 2 ? '2' : ''), grade: 2, class_no: cls, number: n, status: 'active', ability })
    id++
  }
  return out
}
export const MOCK_STUDENTS = makeStudents()

function recsOf(s) {
  const recs = []
  for (const [term, list] of Object.entries(SUBJ)) {
    const drift = term === '2-1' ? (seed(s.id * 3) - 0.5) * 0.3 : 0
    list.forEach(([subject, area, kind, credits, scale = 5], i) => {
      const p = Math.min(0.98, Math.max(0.02, s.ability + drift + (seed(s.id * 31 + i * 7 + term.length) - 0.5) * 0.35))
      const grade5 = scale === 5 ? (p <= 0.1 ? 1 : p <= 0.34 ? 2 : p <= 0.66 ? 3 : p <= 0.9 ? 4 : 5) : null
      const achIdx = scale === 3 ? Math.min(2, Math.floor(p * 3)) : Math.min(4, Math.floor(p * 5))
      const raw = scale === 5 ? Math.round(95 - p * 60 + (seed(s.id + i) - 0.5) * 10) : null
      recs.push({ term, subject, area, kind, credits, raw, subject_mean: scale === 5 ? 62.5 : null, achievement: 'ABCDE'[achIdx], ach_scale: scale === 0 ? 5 : scale, grade5: scale === 0 ? null : grade5,
        dist: scale === 5 || scale === 0 ? [8, 22, 30, 24, 16] : [null, null, null, null, null], enrolled_n: scale === 5 ? 258 : null, note: null })
    })
  }
  return recs
}
const wavg = (list) => { let w = 0, s = 0; for (const [v, c] of list) { w += c; s += v * c } return w ? s / w : null }
const gpa = (recs, term, scope) => wavg(recs.filter(r => (term === 'ALL' || r.term === term) && (scope === 'all' || ['국어', '수학', '영어', '사회', '과학'].includes(r.area)) && r.grade5 != null).map(r => [r.grade5, r.credits]))
const mid = { 1: 5, 2: 22, 3: 50, 4: 78, 5: 95 }
const posRaw = (recs, term) => wavg(recs.filter(r => (term === 'ALL' || r.term === term) && ['국어', '수학', '영어', '사회', '과학'].includes(r.area) && r.grade5 != null).map(r => [mid[r.grade5], r.credits]))
const round = (v, d = 2) => v == null ? null : Math.round(v * 10 ** d) / 10 ** d

function metricsFor(s, all, term, scope) {
  const recs = recsOf(s)
  const ranks = all.map(x => ({ id: x.id, p: posRaw(recsOf(x), term) })).filter(x => x.p != null).sort((a, b) => a.p - b.p)
  const idx = ranks.findIndex(x => x.id === s.id)
  const pos_pct = idx < 0 ? null : round((idx + 1) / ranks.length * 100, 1)
  const areas = Object.fromEntries(['국어', '수학', '영어', '사회', '과학'].map(a => [a, round(wavg(recs.filter(r => (term === 'ALL' || r.term === term) && r.area === a && r.grade5 != null).map(r => [r.grade5, r.credits])))]))
  const radar = AXES.map(a => a === '기타' ? round(wavg(recs.filter(r => (term === 'ALL' || r.term === term) && !['국어', '수학', '영어', '사회', '과학'].includes(r.area) && r.grade5 != null).map(r => [r.grade5, r.credits]))) : areas[a])
  const ach = { a: 0, b: 0, c: 0, d: 0, e: 0 }
  recs.filter(r => term === 'ALL' || r.term === term).forEach(r => { ach[r.achievement.toLowerCase()]++ })
  const entry_top = round(10 + s.ability * 80, 1)
  const mock_pct = round(100 - (s.ability * 100 + (seed(s.id * 5) - 0.5) * 40), 1)
  const pos_delta = round((seed(s.id * 11) - 0.5) * 30, 1)
  const tags = []
  if (pos_delta >= 10) tags.push({ text: `직전 학기보다 교내 위치 ${pos_delta.toFixed(0)}p 하락`, level: 'crit' })
  if (ach.e >= 3) tags.push({ text: `E ${ach.e}과목`, level: 'crit' })
  if (pos_pct != null && 100 - mock_pct - pos_pct >= 20) tags.push({ text: '내신 대비 모의 약함', level: 'warn' })
  if (pos_pct != null && pos_pct - (100 - mock_pct) >= 20) tags.push({ text: '모의 대비 내신 약함', level: 'warn' })
  if (pos_delta <= -10) tags.push({ text: `직전 학기보다 교내 위치 ${Math.abs(pos_delta).toFixed(0)}p 상승`, level: 'ok' })
  return {
    student_id: s.id, anon_id: s.anon_id, name: s.name, hakbun: `${s.grade}${String(s.class_no).padStart(2, '0')}${String(s.number).padStart(2, '0')}`, class_no: s.class_no, number: s.number,
    gpa_main: round(gpa(recs, term, 'main')), gpa_all: round(gpa(recs, term, 'all')), pos_pct, pos_rank: idx < 0 ? null : idx + 1, pos_n: ranks.length,
    grade9_equiv: pos_pct == null ? null : [4, 11, 23, 40, 60, 77, 89, 96, 100].findIndex(c => pos_pct <= c) + 1,
    ach, radar, areas, pos_delta, entry_move: pos_pct == null ? null : round(entry_top - pos_pct, 1), entry_top, mock_pct, mock_round: '6월', tags, graded_n: recs.filter(r => r.grade5 != null).length,
  }
}

export const mockAnalysis = {
  get_contexts() { return [{ grade_year: 2026, grade: 2, classes: [1, 2], terms: ['1-1', '1-2', '2-1'], students: 16 }] },
  search_student({ q }) { return MOCK_STUDENTS.filter(s => s.name.includes(q) || `${s.grade}${String(s.class_no).padStart(2, '0')}${String(s.number).padStart(2, '0')}`.includes(q)).slice(0, 20) },
  get_class_report({ classNo, term, scope }) {
    const all = MOCK_STUDENTS.map(s => metricsFor(s, MOCK_STUDENTS, term, scope))
    const rows = all.filter(m => m.class_no === classNo)
    const avg = (l, k) => { const v = l.map(x => x[k]).filter(x => x != null); return v.length ? round(v.reduce((a, b) => a + b, 0) / v.length) : null }
    const bucket = (l) => { const b = [0, 0, 0, 0, 0]; l.forEach(m => { if (m.gpa_main != null) b[Math.min(4, Math.max(0, Math.round(m.gpa_main) - 1))]++ }); return b }
    const cb = bucket(rows), gb = bucket(all)
    const radarAvg = (l) => AXES.map((_, i) => { const v = l.map(m => m.radar[i]).filter(x => x != null); return v.length ? round(v.reduce((a, b) => a + b, 0) / v.length) : null })
    return {
      grade_year: 2026, grade: 2, class_no: classNo, term, term_label: term === 'ALL' ? '전체 누적' : term.replace('-', '학년 ') + '학기', scope, terms: ['1-1', '1-2', '2-1'], prev_term: ['2-1', '1-2'],
      class_n: rows.length, grade_n: all.length, class_gpa: avg(rows, 'gpa_main'), grade_gpa: avg(all, 'gpa_main'), class_pos_mean: avg(rows, 'pos_pct'), rows,
      dist: cb.map((n, i) => ({ grade: i + 1, class_n: n, class_pct: round(n / Math.max(1, rows.length) * 100, 1), grade_n: gb[i], grade_pct: round(gb[i] / all.length * 100, 1) })),
      radar_class: radarAvg(rows), radar_grade: radarAvg(all),
      watch: rows.map(m => ({ student_id: m.student_id, name: m.name, number: m.number, gpa_main: m.gpa_main, reasons: m.tags.filter(t => t.level !== 'ok').map(t => t.text) })).filter(w => w.reasons.length).slice(0, 8),
      notes: ['등급 평균은 학점 가중 표시용 값입니다. 교내 위치는 석차백분율 환산(5등급 누적 컷 구간 중앙값)으로 계산했습니다.'],
    }
  },
  get_student_bundle({ studentId, term, scope }) {
    const s = MOCK_STUDENTS.find(x => x.id === studentId)
    if (!s) throw '학생을 찾을 수 없습니다.'
    const all = MOCK_STUDENTS.map(x => metricsFor(x, MOCK_STUDENTS, term, scope))
    const m = all.find(x => x.student_id === studentId)
    const recs = recsOf(s).filter(r => term === 'ALL' || r.term === term)
    const subjects = recs.map(r => ({
      subject: r.subject, credits: r.credits, raw: r.raw, achievement: r.achievement, grade5: r.grade5, grade9_equiv: r.grade5 ? Math.min(9, r.grade5 * 2 - 1) : null, subject_mean: r.subject_mean, subject_sd: null, enrolled_n: r.enrolled_n,
      z: null, rank: r.grade5 ? Math.round(mid[r.grade5] / 100 * 258) : null, rank_estimated: true, top_pct: r.grade5 ? mid[r.grade5] : null,
      relative_label: r.grade5 ? `${r.grade5}등급 구간 / 상위 약 ${mid[r.grade5]}%` : '정보 없음',
      feedback: r.grade5 ? `과목평균보다 ${(r.raw - r.subject_mean).toFixed(1)}점 ${r.raw >= r.subject_mean ? '높습니다' : '낮습니다'}. 성취도 ${r.achievement} — 수강자 중 A 비율 8.0%. 현재 ${r.grade5}등급 (석차·표준편차가 없어 등급 안 세부 위치는 추정하지 않습니다).` : (r.ach_scale === 3 ? '3단계 성취도(A~C) 과목이므로 등급 평균에서 제외했습니다.' : '성취도(A~E)만 있는 과목이므로 등급 평균에서 제외했습니다.'),
      excluded_from_avg: r.grade5 == null, radar: r.grade5 ? 100 - mid[r.grade5] : null, area: r.area, kind: r.kind, term: r.term, ach_scale: r.ach_scale, deviation: r.raw != null ? round(r.raw - r.subject_mean, 1) : null, dist: r.dist,
    }))
    const radarAvg = AXES.map((_, i) => { const v = all.map(x => x.radar[i]).filter(x => x != null); return v.length ? round(v.reduce((a, b) => a + b, 0) / v.length) : null })
    const trends = ['국어', '수학', '영어', '사회', '과학'].map(a => { const points = ['1-1', '1-2', '2-1'].map(t => ({ term: t, gpa: round(wavg(recsOf(s).filter(r => r.term === t && r.area === a && r.grade5 != null).map(r => [r.grade5, r.credits]))) })); const seen = points.map(p => p.gpa).filter(x => x != null); return { area: a, points, latest: seen.at(-1) ?? null, change: seen.length >= 2 ? round(seen.at(-1) - seen[0]) : null } })
    const mk = (round_, pct) => ({ grade_year: 2026, grade: 2, round: round_, label: `2학년 ${round_}월`, avg_pct: pct, raw_only: false, subjects: [['국어', null, 110, pct + 3, 4, false], ['수학', null, 105, pct - 4, 5, false], ['영어', null, null, null, 3, false], ['통합사회', null, 52, pct + 6, 4, false], ['통합과학', null, 48, pct - 5, 5, false], ['한국사', null, null, null, 2, false]] })
    return {
      student: { ...s }, grade_year: 2026, term, term_label: term === 'ALL' ? '전체 누적' : term.replace('-', '학년 ') + '학기', scope, terms: ['1-1', '1-2', '2-1'], metrics: m, radar_grade: radarAvg,
      radar_note: '가장 강한 교과는 영어(2.10), 가장 약한 교과는 수학(3.40)입니다. 차이가 뚜렷한 편입니다.', subjects, trends,
      mocks: [mk(3, m.mock_pct - 5), mk(6, m.mock_pct)], in_pct: m.pos_pct == null ? null : round(100 - m.pos_pct, 1),
      mock_note: '교내 내신이 전국 모의고사보다 앞섭니다. 수시 교과·종합이 유리하고, 수능 최저 충족 여부를 확인해야 합니다.',
      entry: { mid_school_score: 21.3, placement_rank: 46, placement_n: 262, subjects: { 국어: 68, 수학: 96, 영어: 88 }, top_pct: m.entry_top }, entry_note: '입학 때 위치를 그대로 지키고 있습니다.',
      univ: { calc: { kangwon: 476.4, kangwon_n: 12, kangwon_cnt: { A: 3, B: 4, C: 3, D: 2 }, hallym: 41.7, hallym_n: 9, hallym_cred: 33, kh_before: 2.61, kh_after: 2.44, kh_gain: 0.17, kh_swapped: ['세계시민과 지리'], kh_excluded: ['사회문제 탐구'], dongguk: 7.4, dongguk_used: [], dongguk_total: 8 }, kangwon_top: 38.5, hallym_top: 41.2, dongguk_top: 36.9, grade9_cont: 4.12 },
      memo: `[${m.hakbun} ${s.name} · 2학년 ${s.class_no}반 ${s.number}번 · 전체 누적 기준]\n\n· 성적 위치 : 주요교과 ${m.gpa_main}등급, 전 과목 ${m.gpa_all}등급\n· 성취도 : A ${m.ach.a} / B ${m.ach.b} / C ${m.ach.c} / D ${m.ach.d} / E ${m.ach.e} 과목\n\n[상담에서 확인할 것]\n1. 위 수치 중 학생 본인이 예상과 다르다고 느끼는 부분은?\n\n[담임 의견]\n`,
      ai_payload: { 학년: 2, 내신: { 주요교과_5등급평균: m.gpa_main, 교내_상위백분율: m.pos_pct } }, notes: ['등급 평균은 표시용 단순(학점 가중) 평균입니다.'],
    }
  },
  get_subject_options() {
    return Object.entries(SUBJ).flatMap(([term, list]) => list.map(([subject, area, kind, , scale = 5]) => ({ term, subject, area, kind, n: 16, ach_scale: scale || 5 })))
  },
  get_subject_report({ term, subject, classNo }) {
    const def = SUBJ[term]?.find(x => x[0] === subject)
    if (!def) return null
    const rows = MOCK_STUDENTS.map(s => ({ s, r: recsOf(s).find(r => r.term === term && r.subject === subject) })).filter(x => x.r)
    const gc = [0, 0, 0, 0, 0], ac = [0, 0, 0, 0, 0]
    rows.forEach(({ r }) => { if (r.grade5) gc[r.grade5 - 1]++; ac['ABCDE'.indexOf(r.achievement)]++ })
    const byClass = [1, 2].map(c => { const l = rows.filter(x => x.s.class_no === c); const rr = l.map(x => x.r.raw).filter(x => x != null); return { class_no: c, n: l.length, raw_mean: rr.length ? round(rr.reduce((a, b) => a + b, 0) / rr.length, 1) : null, a_rate: round(l.filter(x => x.r.achievement === 'A').length / l.length * 100, 1), gpa: round(wavg(l.filter(x => x.r.grade5).map(x => [x.r.grade5, 1]))) } })
    return { term, term_label: term.replace('-', '학년 ') + '학기', subject, area: def[1], kind: def[2], ach_scale: def[4] ?? 5, n: rows.length, credits: def[3], enrolled_n: 258, subject_mean: 62.5, raw_mean: 61.2, raw_sd: 18.4, dist_official: [8, 22, 30, 24, 16], ach_counts: ac, grade_counts: gc, by_class: byClass,
      students: rows.filter(x => !classNo || x.s.class_no === classNo).map(({ s, r }) => ({ student_id: s.id, name: s.name, class_no: s.class_no, number: s.number, raw: r.raw, deviation: r.raw != null ? round(r.raw - 62.5, 1) : null, achievement: r.achievement, grade5: r.grade5 })).sort((a, b) => (b.raw ?? -1) - (a.raw ?? -1)) }
  },
  get_admission_aggregate() {
    return [
      { univ: '강원대학교(춘천)', adm_type: '학생부위주(교과)', track: '자연', years: [2023, 2024, 2025], apply_n: 61, pass_n: 22, pass_rate: 36.1, apply_min: 2.1, apply_max: 5.4, pass_min: 2.1, pass_median: 3.3, pass_max: 4.0 },
      { univ: '한림대학교(춘천)', adm_type: '학생부위주(종합)', track: '자연', years: [2023, 2024, 2025], apply_n: 44, pass_n: 15, pass_rate: 34.1, apply_min: 2.4, apply_max: 6.1, pass_min: 2.6, pass_median: 3.9, pass_max: 4.8 },
      { univ: '동국대학교(서울)', adm_type: '학생부위주(종합)', track: '인문', years: [2024, 2025], apply_n: 9, pass_n: 2, pass_rate: 22.2, apply_min: 1.9, apply_max: 3.6, pass_min: 1.9, pass_median: 2.2, pass_max: 2.5 },
    ]
  },
  get_regional_admission() {
    return [
      { track: '자연', category: '교과', univ: '가천대', adm_name: '지역균형', apply_n: 168, apply_best: 3.04, apply_worst: 7.32, pass_n: 8, pass_best: 3.17, pass_worst: 5.1, pass_rate: 4.76, cut70: null },
      { track: '자연', category: '교과', univ: '강원대(춘천)', adm_name: '일반전형', apply_n: 402, apply_best: 1.5, apply_worst: 6.9, pass_n: 130, pass_best: 1.5, pass_worst: 4.3, pass_rate: 32.3, cut70: 3.8 },
      { track: '인문', category: '종합', univ: '동국대', adm_name: 'Do Dream', apply_n: 40, apply_best: 1.8, apply_worst: 4.9, pass_n: 5, pass_best: 1.9, pass_worst: 3.1, pass_rate: 12.5, cut70: null },
    ]
  },
  _refs: null,
  get_univ_reference() {
    if (!this._refs) this._refs = [['서울권', '서울대', '인문/자연', '지역균형(교과)', 1.5], ['서울권', '연세대', '인문/자연', '추천형(교과)', 1.6], ['강원권', '강원대(춘천)', '인문/자연', '일반전형(교과)', 3.6], ['강원권', '한림대', '인문/자연', '학생부교과(일반)', 3.7]].map((r, i) => ({ id: i + 1, region: r[0], univ: r[1], unit: r[2], adm_type: r[3], cut: r[4], note: '예시값 — 학교 입결로 교체' }))
    return this._refs
  },
  set_univ_reference({ id, cut, note }) { const r = this.get_univ_reference().find(x => x.id === id); if (r) { r.cut = cut; r.note = note } },
  get_subject_catalog() { return [['공통국어1', null, '공통'], ['문학', null, '일반'], ['사회문제 탐구', null, '융합']] },
  set_subject_catalog({ items }) { return items.length },
  parse_neis({ grid }) { const cls = grid.flat().map(String).find(v => /(\d)학년\s*(\d+)반/.test(v)); const m = cls ? cls.match(/(\d)학년\s*(\d+)반/) : null; if (!m) throw '반 정보를 찾을 수 없습니다.'; return { grade: +m[1], class_no: +m[2], rows: grid.filter(r => r[5] && r[6]).map(() => ({})), students: 28, warnings: [] } },
  parse_mock_file({ grid }) {
    const hr = grid.findIndex(r => r.some(c => String(c).replace(/\s/g, '') === '학번') && r.some(c => String(c).replace(/\s/g, '') === '이름'))
    if (hr < 0) throw '학번·이름 머리글을 찾지 못했습니다.'
    const hasStd = grid[hr].some(c => String(c).includes('표준점수'))
    const rows = grid.slice(hr + 1).filter(r => /^\d{5}$/.test(String(r.find(c => /^\d{5}$/.test(String(c))) ?? '')))
    return { layout: hasStd ? 'analysis' : 'raw', grade: rows.length ? Number(String(rows[0].find(c => /^\d{5}$/.test(String(c))))[0]) : null, round: null, students: rows.length, subjects: hasStd ? ['국어', '수학', '영어'] : ['국어', '수학', '영어', '한국사', '통합사회', '통합과학'], rows: [], warnings: [] }
  },
  import_mock_file({ grid }) { return { row_count: grid.length, inserted: grid.length * 3, updated: 0, skipped: 0, unmatched: [], errors: [] } },
  import_neis({ grid }) { return { row_count: grid.length, inserted: grid.length, updated: 0, skipped: 0, unmatched: [], errors: [] } },
}
