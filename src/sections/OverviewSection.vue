<script setup>
import { computed, onMounted } from 'vue'
import { ChevronRight, Download, IdCard, ListChecks, Siren, Upload, Users } from 'lucide-vue-next'
import { useDatasetStore } from '../stores/dataset'

const emit = defineEmits(['navigate'])
const dataset = useDatasetStore()
onMounted(() => dataset.refreshSummary())

const s = computed(() => dataset.summary)

const steps = [
  { num: 1, icon: Users, title: '학생 명단', desc: '학년·반·번호를 등록하거나 엑셀로 가져옵니다. 분석에 쓰이는 익명 ID가 자동 부여됩니다.', section: 'student', c: 'var(--step1-c)', bg: 'var(--step1-bg)', bd: 'var(--step1-bd)' },
  { num: 2, icon: Download, title: '데이터 가져오기', desc: '모의고사·내신·입학·문항·출결 파일을 넣고 컬럼 매핑을 확인합니다. 파일 양식이 달라도 됩니다.', section: 'import', c: 'var(--step2-c)', bg: 'var(--step2-bg)', bd: 'var(--step2-bd)' },
  { num: 3, icon: ListChecks, title: '데이터 현황·전처리', desc: '결측·응시자·성실도 플래그·제외자 로그를 확인합니다. 이 단계를 건너뛰면 하위권 해석이 틀립니다.', section: 'preprocess', c: 'var(--step3-c)', bg: 'var(--step3-bg)', bd: 'var(--step3-bd)' },
  { num: 4, icon: Siren, title: '스크리닝 명단 · 학생 카드', desc: '플래그별 학생 목록으로 누구를 먼저 볼지 정하고, 학생 카드로 상담합니다.', section: 'screening', c: 'var(--step4-c)', bg: 'var(--step4-bg)', bd: 'var(--step4-bd)' },
  { num: 5, icon: Upload, title: '내보내기', desc: '학교 리포트, 스크리닝 명단, 학생 카드, 원자료+파생지표를 파일로 저장합니다.', section: 'export', c: 'var(--step5-c)', bg: 'var(--step5-bg)', bd: 'var(--step5-bd)' },
]

const kpis = computed(() => [
  { label: '학생 (활성/제외)', value: s.value ? `${s.value.students_active} / ${s.value.students_excluded}` : '—' },
  { label: '모의고사 회차', value: s.value ? String(s.value.mock_rounds.length) : '—', sub: s.value ? `${s.value.mock_rows}행` : '' },
  { label: '내신 학기', value: s.value ? String(s.value.school_semesters.length) : '—', sub: s.value ? `${s.value.school_rows}행` : '' },
  { label: '입학 데이터', value: s.value ? String(s.value.entry_rows) : '—', sub: '명' },
  { label: '문항 통계 / 응답', value: s.value ? `${s.value.item_stat_rows} / ${s.value.item_response_rows}` : '—' },
  { label: '출결', value: s.value ? String(s.value.attendance_rows) : '—', sub: '학생·월 행' },
])

const rules = [
  ['준거 비대칭', '모의고사는 전국 준거, 내신·반배치는 교내 준거. 서로 비교할 때는 교내 석차백분율로 통일하고, 전국 좌표가 필요한 계산에는 모의고사만 씁니다.'],
  ['등급 연산 금지', '5등급제(내신)와 9등급제(학평)는 칸 수가 달라 비교할 수 없습니다. 등급은 표시용이며, 모든 연산은 백분위·석차백분율로 합니다.'],
  ['성장 측정', '개인의 절대적 성장은 모의고사 전국 백분위(또는 표준점수)로만 잽니다. 내신 등급 시계열, 원점수 회차 비교는 성장이 아닙니다.'],
  ['교사용 / 학생용 분리', '잔차·회귀 예측값·전이행렬 확률은 교사용 스크리닝 지표입니다. 학생 카드에는 절대 나오지 않으며, 학생의 비교 대상은 "3월의 자기 자신"입니다.'],
]
</script>

<template>
  <div class="section">
    <div class="section-body">
      <div class="hero">
        <div class="hero-eyebrow"><span class="eyebrow-badge">성적분석</span> 학생 성장 읽기 도우미</div>
        <h1 class="hero-title">데이터를 모으고, 플래그로 거르고,<br>학생 한 명씩 읽습니다</h1>
        <p class="hero-sub">아래 순서대로 진행하면 학년 스크리닝 명단과 학생별 상담 카드가 만들어집니다.</p>
      </div>

      <div class="kpi-row">
        <div v-for="k in kpis" :key="k.label" class="kpi">
          <span class="kpi-label">{{ k.label }}</span>
          <span class="kpi-value" :class="{ 'kpi-value--muted': k.value === '—' }">{{ k.value }}</span>
          <span v-if="k.sub" class="kpi-sub">{{ k.sub }}</span>
        </div>
      </div>

      <div class="steps">
        <div
          v-for="step in steps" :key="step.num" class="step-card" :class="{ 'step-card--wide': step.num === 5 }"
          :style="{ '--c': step.c, '--bg': step.bg, '--bd': step.bd }" @click="emit('navigate', step.section)"
        >
          <div class="step-top">
            <div class="step-num">{{ step.num }}</div>
            <component :is="step.icon" :size="26" class="step-icon" />
            <span class="step-name">{{ step.title }}</span>
          </div>
          <div class="step-desc">{{ step.desc }}</div>
          <button class="step-btn" @click.stop="emit('navigate', step.section)">이동하기 <ChevronRight :size="15" /></button>
        </div>
      </div>

      <div class="panel">
        <div class="panel-head">
          <div>
            <h3 class="panel-title">이 프로그램이 지키는 규칙</h3>
            <p class="panel-desc">어기면 결과가 전부 무효가 되는 도메인 전제입니다. 화면 곳곳의 경고와 잠금은 여기서 나옵니다.</p>
          </div>
        </div>
        <div class="rules">
          <div v-for="[t, d] in rules" :key="t" class="rule">
            <div class="rule-title">{{ t }}</div>
            <div class="rule-desc">{{ d }}</div>
          </div>
        </div>
      </div>

      <div class="quad-panel panel">
        <div class="panel-head">
          <div>
            <h3 class="panel-title"><IdCard :size="16" /> 학생 카드의 핵심: 4분면</h3>
            <p class="panel-desc">가로축 = 내신 교내 석차백분율, 세로축 = 모의고사 전국 백분위. 각 학생 점 위에 3→9월 변화를 화살표로 표시합니다.</p>
          </div>
        </div>
        <div class="quad">
          <div class="q q3"><b>③ 내신↓ 모평↑</b><span>할 줄 아는데 안 함. 개입 효과 최대</span></div>
          <div class="q q1"><b>① 내신↑ 모평↑</b><span>선택지 넓음</span></div>
          <div class="q q4"><b>④ 내신↓ 모평↓</b><span>기초 결손. 절대 수준부터</span></div>
          <div class="q q2"><b>② 내신↑ 모평↓</b><span>성실하나 사고력 한계. 성취도 확인 필수</span></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hero { padding: 8px 0 4px; }
.hero-eyebrow { display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--tx-3); margin-bottom: 10px; }
.eyebrow-badge { padding: 2px 8px; border-radius: 6px; background: rgba(var(--accent-rgb), 0.15); color: var(--accent-text); font-weight: 600; font-size: 12px; }
.hero-title { font-size: 26px; font-weight: 700; color: var(--tx-1); line-height: 1.35; margin: 0 0 10px; letter-spacing: -0.02em; }
.hero-sub { font-size: 14px; color: var(--tx-3); margin: 0; line-height: 1.6; }
.steps { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
.step-card { --c: var(--accent-text); --bg: var(--bg-1); --bd: var(--bd-1); background: var(--bg); border: 1px solid var(--bd); border-radius: 16px; padding: 18px 20px; cursor: pointer; display: flex; flex-direction: column; gap: 10px; transition: transform 0.12s, box-shadow 0.12s; }
.step-card:hover { transform: translateY(-2px); box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18); }
.step-card--wide { grid-column: 1 / -1; }
.step-top { display: flex; align-items: center; gap: 10px; }
.step-num { width: 26px; height: 26px; border-radius: 8px; background: var(--c); color: #fff; font-weight: 700; font-size: 13px; display: flex; align-items: center; justify-content: center; }
.step-icon { color: var(--c); }
.step-name { font-size: 16px; font-weight: 700; color: var(--tx-1); }
.step-desc { font-size: 13px; color: var(--tx-3); line-height: 1.6; }
.step-btn { align-self: flex-start; display: inline-flex; align-items: center; gap: 4px; padding: 6px 12px; border-radius: 8px; border: 1px solid var(--bd); background: none; color: var(--c); font-size: 13px; font-weight: 600; cursor: pointer; }
.rules { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.rule { padding: 12px 14px; border-radius: 12px; background: var(--bg-2); border: 1px solid var(--bd-1); }
.rule-title { font-size: 13px; font-weight: 700; color: var(--accent-text); margin-bottom: 4px; }
.rule-desc { font-size: 13px; color: var(--tx-3); line-height: 1.6; }
.quad { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
.q { padding: 14px; border-radius: 12px; border: 1px solid var(--bd-1); display: flex; flex-direction: column; gap: 4px; font-size: 13px; color: var(--tx-3); }
.q b { color: var(--tx-1); font-size: 14px; }
.q1 { background: var(--clr-green-bg); border-color: var(--clr-green-border); }
.q2 { background: var(--clr-warn-bg); border-color: var(--clr-warn-border); }
.q3 { background: rgba(var(--accent-rgb), 0.1); border-color: rgba(var(--accent-rgb), 0.3); }
.q4 { background: var(--clr-red-bg); border-color: var(--clr-red-border); }
@media (max-width: 1000px) { .steps, .rules { grid-template-columns: 1fr; } }
</style>
