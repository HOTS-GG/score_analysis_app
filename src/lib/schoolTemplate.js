import { Workbook } from 'exceljs'

/** 담임용 내신 입력 양식(세로형). 앱 가져오기 화면의 '내신' 세로형 매핑이 그대로 인식한다. */
export const SCHOOL_TEMPLATE_HEADERS = ['학년', '반', '번호', '이름', '과목', '단위수', '원점수', '성취도', '5등급', '과목평균', '표준편차', '수강자수', '석차']

export async function buildSchoolTemplate({ grade = 1, classNo = 1, students = [] } = {}) {
  const wb = new Workbook()
  const ws = wb.addWorksheet('내신입력')
  ws.addRow(SCHOOL_TEMPLATE_HEADERS)
  ws.getRow(1).font = { bold: true }
  const example = [
    [grade, classNo, 1, '예시학생', '문학', 4, 86.77, 'B', 2, 67.2, 17.8, 258, 34],
    [grade, classNo, 1, '예시학생', '대수', 4, 87.19, 'A', 1, 54.9, 23.4, 258, 10],
    [grade, classNo, 1, '예시학생', '사회문제 탐구', 3, 100, 'A', '', 97.3, 5.7, 80, 1],
  ]
  if (students.length) {
    for (const s of students) ws.addRow([grade, classNo, s.number, s.name, '', '', '', '', '', '', '', '', ''])
  } else {
    for (const r of example) ws.addRow(r)
  }
  ws.columns.forEach((c, i) => { c.width = i === 3 || i === 4 ? 16 : 10 })
  const guide = wb.addWorksheet('작성요령')
  ;[
    ['한 행 = 학생 1명 × 과목 1개 (세로형). 과목 수만큼 행을 추가하세요.'],
    ['성취도만 있는 과목(A·B·C 과목, 체육·예술 등)은 5등급 칸을 비우세요. 등급 평균에서 자동 제외됩니다.'],
    ['석차가 없으면 비워도 됩니다. 과목평균·표준편차·수강자수로 위치를 추정하고 "추정" 표시가 붙습니다.'],
    ['미응시·미이수는 원점수를 빈칸으로 두세요. 0점으로 쓰지 마세요.'],
    ['과목명 뒤에 (4) 처럼 단위수를 붙여도 인식합니다.'],
    ['이름은 명단 매칭 확인용이며 분석 테이블에는 저장되지 않습니다.'],
  ].forEach(r => guide.addRow(r))
  guide.getColumn(1).width = 90
  return await wb.xlsx.writeBuffer()
}

/** 반 전체 카드 → 엑셀 (학생별 시트 + 요약 시트) */
export async function buildReportsWorkbook(reports) {
  const wb = new Workbook()
  const sum = wb.addWorksheet('요약')
  sum.addRow(['학년', '반', '번호', '이름', '5등급제 평균', '9등급 환산 평균', '등급 과목 수'])
  sum.getRow(1).font = { bold: true }
  for (const r of reports) sum.addRow([r.grade, r.class_no, r.number, r.name || r.anon_id, r.avg_grade5 ?? '', r.avg_grade9 ?? '', r.n_graded])
  for (const r of reports) {
    const name = `${r.number}번 ${r.name || r.anon_id}`.replace(/[\\/?*[\]:]/g, ' ').slice(0, 28)
    const ws = wb.addWorksheet(name)
    ws.addRow([`${r.grade}학년 ${r.class_no}반 ${r.number}번 ${r.name || r.anon_id} 학생 개인 분석 (${r.grade_year}학년도 ${r.semester}학기)`])
    ws.addRow(['5등급제 평균 등급', r.avg_grade5 ?? '-', '9등급제 환산 평균 등급', r.avg_grade9 ?? '-'])
    ws.addRow([])
    ws.addRow(['과목', '단위수', '원점수', '성취도', '현재 5등급', '9등급 환산', '과목 평균', '표준편차', 'Z점수', '석차', '수강자수', '상위 %', '상대위치', '피드백'])
    ws.getRow(4).font = { bold: true }
    for (const s of r.subjects) {
      ws.addRow([s.subject, s.credits ?? '', s.raw ?? '', s.achievement ?? '', s.grade5 ?? (s.achievement ?? ''), s.grade9_equiv ?? '', s.subject_mean ?? '', s.subject_sd ?? '', s.z ?? '',
        s.rank != null ? (s.rank_estimated ? `${s.rank}(추정)` : s.rank) : '', s.enrolled_n ?? '', s.top_pct ?? '', s.relative_label, s.feedback])
    }
    ws.columns.forEach((c, i) => { c.width = i === 0 ? 18 : i === 13 ? 80 : i === 12 ? 22 : 10 })
  }
  return await wb.xlsx.writeBuffer()
}

/** 상담 리포트(번들) → 엑셀 */
export async function buildBundleWorkbook(b, memo) {
  const wb = new Workbook()
  const m = b.metrics
  const ws = wb.addWorksheet('상담 리포트')
  ws.addRow([`${m.hakbun} ${b.student.name || b.student.anon_id} · ${b.student.grade}학년 ${b.student.class_no}반 ${b.student.number}번 · ${b.term_label} 기준`])
  ws.getRow(1).font = { bold: true, size: 13 }
  ws.addRow(['주요교과 등급평균', m.gpa_main ?? '', '전 과목', m.gpa_all ?? '', '교내 위치(상위%)', m.pos_pct ?? '', '9등급 환산', m.grade9_equiv ?? '', '직전 대비(p)', m.pos_delta ?? '', '입학 대비(p)', m.entry_move ?? ''])
  ws.addRow(['눈여겨볼 점', m.tags.map(t => t.text).join(' · ')])
  ws.addRow([])
  ws.addRow(['학기', '과목', '교과', '구분', '학점', '원점수', '과목평균', '편차', '성취도', '등급', 'A%', 'B%', 'C%', 'D%', 'E%', '수강자', '위치', '피드백'])
  ws.getRow(5).font = { bold: true }
  for (const s of b.subjects) ws.addRow([s.term, s.subject, s.area, s.kind, s.credits ?? '', s.raw ?? '', s.subject_mean ?? '', s.deviation ?? '', s.achievement ?? '', s.grade5 ?? '', ...s.dist.map(x => x ?? ''), s.enrolled_n ?? '', s.relative_label, s.feedback])
  ws.addRow([])
  ws.addRow(['교과별 추이'])
  for (const t of b.trends) ws.addRow([t.area, ...t.points.map(p => `${p.term}: ${p.gpa ?? '-'}`)])
  ws.addRow([])
  for (const mk of b.mocks) {
    ws.addRow([`${mk.label} 학력평가${mk.raw_only ? ' (가채점)' : ''}`, '과목', '원점수', '표준점수', '백분위', '등급'])
    for (const s of mk.subjects) ws.addRow(['', s[0], s[1] ?? '', s[2] ?? '', s[3] ?? '', s[4] ?? ''])
  }
  ws.addRow([])
  ws.addRow(['진학 참고', '강원대 환산', b.univ.calc.kangwon ?? '', '한림대 가산', b.univ.calc.hallym ?? '', '경희대 개선', b.univ.calc.kh_gain ?? '', '동국대', b.univ.calc.dongguk ?? ''])
  ws.columns.forEach((c, i) => { c.width = i === 1 ? 18 : i === 17 ? 70 : 11 })
  const memoWs = wb.addWorksheet('상담 메모')
  for (const line of String(memo ?? b.memo).split('\n')) memoWs.addRow([line])
  memoWs.getColumn(1).width = 100
  return await wb.xlsx.writeBuffer()
}
