<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { ChevronLeft, ChevronRight, Copy, FileSpreadsheet, Printer, ShieldOff, UserRound } from 'lucide-vue-next'
import { useContextStore } from '../stores/context'
import { useAnalysisStore } from '../stores/analysis'
import { useReportStore } from '../stores/report'
import { buildBundleWorkbook } from '../lib/schoolTemplate'
import GradeBadge from '../components/GradeBadge.vue'
import AchStack from '../components/AchStack.vue'
import PolyRadar from '../components/charts/PolyRadar.vue'
import Sparkline from '../components/charts/Sparkline.vue'
import Ruler from '../components/charts/Ruler.vue'
import HBars from '../components/charts/HBars.vue'

const emit = defineEmits(['navigate'])
const ctx = useContextStore()
const analysis = useAnalysisStore()
const reportStore = useReportStore()
const AXES = ['국어', '수학', '영어', '사회', '과학', '기타']
const msg = ref('')
const showPayload = ref(false)
const memoEdit = ref('')

async function load() {
  if (!ctx.studentId) return
  await analysis.fetchBundle(ctx.studentId, ctx.term, ctx.scope)
  memoEdit.value = analysis.bundle?.memo ?? ''
}
onMounted(async () => {
  if (!ctx.studentId && analysis.classReport?.rows?.length) ctx.studentId = analysis.classReport.rows[0].student_id
  await load()
})
watch(() => [ctx.studentId, ctx.term, ctx.scope], load)

const b = computed(() => analysis.bundle)
const m = computed(() => b.value?.metrics)
const fmt = (v, d = 2) => v == null ? '—' : Number(v).toFixed(d)
const classRows = computed(() => analysis.classReport?.rows ?? [])
const idx = computed(() => classRows.value.findIndex(r => r.student_id === ctx.studentId))
function step(d) { const r = classRows.value[idx.value + d]; if (r) ctx.studentId = r.student_id }

const latestMock = computed(() => (b.value?.mocks ?? []).filter(x => !x.raw_only).at(-1) ?? null)
const rawMock = computed(() => (b.value?.mocks ?? []).filter(x => x.raw_only).at(-1) ?? null)
const mockBars = computed(() => !latestMock.value ? [] : latestMock.value.subjects.filter(s => s[3] != null || s[4] != null).map(s => ({
  label: s[0], value: s[3] ?? (s[4] != null ? (1 - (s[4] - 1) / 8) * 100 : 0), max: 100,
  text: `${s[3] != null ? Number(s[3]).toFixed(1) : '—'}<br><span style="color:var(--tx-4)">${s[4] == null ? '—' : s[4] + '등급'}</span>`,
})))
const rawBars = computed(() => !rawMock.value ? [] : rawMock.value.subjects.filter(s => s[1] != null).map(s => ({ label: s[0], value: s[1], max: ['한국사', '통합사회', '통합과학'].includes(s[0]) ? 50 : 100, text: String(s[1]) })))
const subjLabel = (s) => s.credits != null ? `${s.subject}(${s.credits})` : s.subject
const distText = (d) => d.every(x => x == null) ? '' : 'ABCDE'.split('').map((L, i) => `${L}${d[i] == null ? '-' : d[i]}`).join(' ')

async function copyMemo() {
  try { await navigator.clipboard.writeText(memoEdit.value); msg.value = '메모를 복사했습니다.' } catch { msg.value = '복사할 수 없습니다.' }
  setTimeout(() => (msg.value = ''), 2000)
}
async function exportExcel() {
  if (!b.value) return
  try {
    const buf = await buildBundleWorkbook(b.value, memoEdit.value)
    const saved = await reportStore.saveBinary(buf, `상담리포트_${m.value.hakbun}_${b.value.term}.xlsx`)
    if (saved) msg.value = `저장했습니다: ${saved}`
  } catch (e) { msg.value = String(e) }
}
</script>

<template>
  <div class="section">
    <div class="section-header compact no-print">
      <div class="row">
        <button class="btn-ghost" :disabled="idx <= 0" @click="step(-1)"><ChevronLeft :size="14" /> 이전</button>
        <button class="btn-ghost" :disabled="idx < 0 || idx >= classRows.length - 1" @click="step(1)">다음 <ChevronRight :size="14" /></button>
        <span class="hint">{{ ctx.contextLabel }} · {{ b?.term_label ?? '' }} 기준 · 상단 검색(Ctrl+K)으로 다른 학생을 찾을 수 있습니다</span>
      </div>
      <div class="header-actions">
        <button class="btn-danger btn-sm" :disabled="!b" @click="window.print()"><Printer :size="14" /> PDF 저장</button>
        <button class="btn-add btn-sm" :disabled="!b" @click="exportExcel"><FileSpreadsheet :size="14" /> 엑셀 저장</button>
      </div>
    </div>

    <div class="section-body">
      <p v-if="msg" class="msg-success no-print">{{ msg }}</p>
      <p v-if="analysis.error" class="msg-error no-print">{{ analysis.error }}</p>
      <div v-if="!ctx.studentId" class="empty-state">
        <UserRound :size="40" color="var(--tx-3)" />
        <p class="empty-title">학생을 선택하세요</p>
        <p class="empty-desc">학급 화면에서 행을 누르거나, 상단 검색창에서 이름·학번을 입력하세요.</p>
        <button class="btn-soft" @click="emit('navigate', 'class')">학급 한눈에로</button>
      </div>

      <template v-else-if="b && m">
        <!-- 머리: 이름·태그·KPI -->
        <div class="panel head card-print">
          <div class="head-row">
            <div class="who">
              <div class="nm">{{ b.student.name || b.student.anon_id }}</div>
              <div class="meta"><b class="mono">{{ m.hakbun }}</b> · {{ b.student.grade }}학년 {{ b.student.class_no }}반 {{ b.student.number }}번 · {{ b.term_label }} 기준</div>
              <div class="tags"><span v-for="t in m.tags" :key="t.text" class="chip" :class="t.level">{{ t.text }}</span><span v-if="!m.tags.length" class="chip ok">특이 사항 없음</span></div>
            </div>
            <div class="kpis">
              <div class="k"><span class="kl">주요교과</span><span class="kv">{{ fmt(m.gpa_main) }}</span><span class="ks">국·수·영·사·과 등급평균</span></div>
              <div class="k"><span class="kl">전 과목</span><span class="kv">{{ fmt(m.gpa_all) }}</span><span class="ks">등급 산출 과목</span></div>
              <div class="k"><span class="kl">교내 위치</span><span class="kv">{{ m.pos_pct == null ? '—' : '상위 ' + m.pos_pct.toFixed(1) + '%' }}</span><span class="ks">{{ m.pos_rank ? `${b.student.grade}학년 ${m.pos_n}명 중 ${m.pos_rank}번째` : '' }}</span></div>
              <div class="k"><span class="kl">9등급 환산</span><span class="kv">{{ m.grade9_equiv ?? '—' }}</span><span class="ks">진학 참고용</span></div>
              <div class="k"><span class="kl">직전 대비</span><span class="kv" :class="{ good: m.pos_delta < 0, bad: m.pos_delta > 0 }">{{ m.pos_delta == null ? '—' : (m.pos_delta > 0 ? '▼' : '▲') + Math.abs(m.pos_delta).toFixed(0) + 'p' }}</span><span class="ks">교내 위치 변화</span></div>
            </div>
          </div>
        </div>

        <div class="grid2">
          <div class="panel">
            <div class="panel-head"><div><h3 class="panel-title">교과별 균형</h3><p class="panel-desc">이 학생 vs {{ b.student.grade }}학년 전체 평균</p></div></div>
            <PolyRadar :axes="AXES" :series="[{ name: `${b.student.grade}학년 평균`, values: b.radar_grade, color: 'var(--clr-note-text)', dashed: true }, { name: b.student.name || '학생', values: m.radar, fill: true, label: true }]" mode="grade" />
            <p class="note-text">{{ b.radar_note }}</p>
          </div>
          <div class="panel">
            <div class="panel-head"><div><h3 class="panel-title">교과별 추이</h3><p class="panel-desc">학기별 등급평균 (1등급이 위)</p></div></div>
            <div class="trends">
              <div v-for="t in b.trends" :key="t.area" class="trend">
                <div class="tl"><b>{{ t.area }}</b><span>{{ t.latest == null ? '—' : t.latest.toFixed(2) + '등급' }}<template v-if="t.change != null"> · {{ t.change < 0 ? '▲' : t.change > 0 ? '▼' : '—' }} {{ Math.abs(t.change).toFixed(2) }}</template></span></div>
                <Sparkline :points="t.points" />
              </div>
            </div>
            <div class="ach-line">성취도 <AchStack :counts="m.ach" /></div>
          </div>
        </div>

        <!-- 과목 표 -->
        <div class="panel">
          <div class="panel-head"><div><h3 class="panel-title">과목별 성적</h3><p class="panel-desc">{{ b.term_label }} · 편차 = 원점수 − 과목평균 · 분포 = 수강자 성취도 비율(%)</p></div></div>
          <div class="table-wrap">
            <table class="data-table sub">
              <thead><tr><th>학기</th><th>과목</th><th>교과·구분</th><th class="n">학점</th><th class="n">원점수</th><th class="n">과목평균</th><th class="n">편차</th><th>성취도</th><th>등급</th><th>분포 A~E</th><th class="n">수강자</th><th>위치·피드백</th></tr></thead>
              <tbody>
                <tr v-for="s in b.subjects" :key="s.term + s.subject" :class="{ dim: s.excluded_from_avg }">
                  <td class="muted">{{ s.term }}</td>
                  <td class="nm">{{ subjLabel(s) }}</td>
                  <td class="muted">{{ s.area }}<span v-if="s.kind && s.kind !== '미분류'"> · {{ s.kind }}</span></td>
                  <td class="n">{{ s.credits ?? '' }}</td>
                  <td class="n">{{ s.raw ?? '—' }}</td>
                  <td class="n">{{ s.subject_mean ?? '—' }}</td>
                  <td class="n" :class="{ good: s.deviation >= 8, bad: s.deviation <= -8 }">{{ s.deviation == null ? '—' : (s.deviation > 0 ? '+' : '') + s.deviation }}</td>
                  <td><GradeBadge :value="s.achievement" /></td>
                  <td><GradeBadge :value="s.grade5" :digits="0" /></td>
                  <td class="muted small">{{ distText(s.dist) }}</td>
                  <td class="n">{{ s.enrolled_n ?? '' }}</td>
                  <td class="fb"><b>{{ s.relative_label }}</b> {{ s.feedback }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div class="grid2">
          <div class="panel">
            <div class="panel-head"><div><h3 class="panel-title">내신 vs 모의고사</h3><p class="panel-desc">교내 내신 백분위와 최근 학력평가 전국 백분위 (오른쪽이 상위)</p></div></div>
            <template v-if="latestMock || b.in_pct != null">
              <Ruler :pins="[{ value: b.in_pct, label: b.in_pct == null ? '' : '내신 ' + Math.round(b.in_pct), color: 'var(--accent-hex)' }, { value: latestMock?.avg_pct, label: latestMock ? `모의 ${Math.round(latestMock.avg_pct)} (${latestMock.label})` : '', color: 'var(--clr-note-text)', below: true }]" />
              <p class="note-text">{{ b.mock_note }}</p>
              <template v-if="latestMock">
                <h4 class="h4">{{ latestMock.label }} 영역별 전국 위치 <small>막대가 길수록 전국 상위 · 영어·한국사는 절대평가라 등급만</small></h4>
                <HBars :rows="mockBars" />
              </template>
              <template v-if="rawMock">
                <h4 class="h4">{{ rawMock.label }} 가채점 <small>원점수만 집계된 단계 — 등급·백분위는 채점 뒤에</small></h4>
                <HBars :rows="rawBars" />
              </template>
              <p class="note-text small">학력평가 등급은 9등급, 내신 석차등급은 5등급입니다. 숫자를 직접 비교하지 마세요.</p>
            </template>
            <div v-else class="placeholder" style="min-height:90px">이 학생의 학력평가 자료가 없습니다.</div>
          </div>
          <div class="panel">
            <div class="panel-head"><div><h3 class="panel-title">입학 때와 견주면</h3><p class="panel-desc">고입내신 + 반배치고사 기준 위치 → 현재 교내 내신 위치</p></div></div>
            <template v-if="b.entry">
              <Ruler :pins="[{ value: b.entry.top_pct == null ? null : 100 - b.entry.top_pct, label: b.entry.top_pct == null ? '' : '입학 ' + Math.round(100 - b.entry.top_pct), color: 'var(--clr-note-text)', below: true }, { value: b.in_pct, label: b.in_pct == null ? '' : '현재 ' + Math.round(b.in_pct), color: 'var(--accent-hex)' }]" />
              <p class="note-text">{{ b.entry_note }}</p>
              <table class="data-table mini"><tbody>
                <tr><td>고입내신 석차백분율</td><td class="n">{{ b.entry.mid_school_score == null ? '—' : b.entry.mid_school_score.toFixed(1) + '%' }}</td><td>반배치 국어</td><td class="n">{{ b.entry.subjects['국어'] ?? '—' }}</td></tr>
                <tr><td>반배치 석차</td><td class="n">{{ b.entry.placement_rank == null ? '—' : `${b.entry.placement_rank}등 / ${b.entry.placement_n}명` }}</td><td>반배치 수학</td><td class="n">{{ b.entry.subjects['수학'] ?? '—' }}</td></tr>
                <tr><td>입학 상위</td><td class="n">{{ b.entry.top_pct == null ? '—' : b.entry.top_pct.toFixed(1) + '%' }}</td><td>반배치 영어</td><td class="n">{{ b.entry.subjects['영어'] ?? '—' }}</td></tr>
              </tbody></table>
            </template>
            <div v-else class="placeholder" style="min-height:90px">{{ b.entry_note }}</div>
          </div>
        </div>

        <!-- 진학 참고 -->
        <div class="panel">
          <div class="panel-head"><div><h3 class="panel-title">진학 참고 — 네 대학 성취도 산식</h3><p class="panel-desc">2028학년도 전형계획 가이드 산식을 그대로 적용. 최종 모집요강 확인 필수. 9등급 환산(연속값) {{ b.univ.grade9_cont ?? '—' }}</p></div><button class="btn-ghost no-print" @click="emit('navigate', 'admission')">진학 화면으로</button></div>
          <div class="ucards">
            <div class="uc"><b>강원대 · 직접 반영형</b><span class="uv">{{ b.univ.calc.kangwon ?? '—' }}<small>/500</small></span><span class="us">성취도 환산점수(학점 가중) · 반영 {{ b.univ.calc.kangwon_n }}과목{{ b.univ.kangwon_top != null ? ` · 교내 상위 ${b.univ.kangwon_top}%` : '' }}</span></div>
            <div class="uc"><b>한림대 · 보상형</b><span class="uv">{{ b.univ.calc.hallym ?? '—' }}<small>/60</small></span><span class="us">국수영사과 성취도 가산점 평균 · 이수 {{ b.univ.calc.hallym_cred }}학점{{ b.univ.hallym_top != null ? ` · 교내 상위 ${b.univ.hallym_top}%` : '' }}</span></div>
            <div class="uc"><b>경희대 · 우선 활용형</b><span class="uv">{{ b.univ.calc.kh_gain == null ? '—' : (b.univ.calc.kh_gain > 0 ? '−' : '') + Math.abs(b.univ.calc.kh_gain).toFixed(2) }}<small> 등급 개선</small></span><span class="us">{{ b.univ.calc.kh_before == null ? '' : `${b.univ.calc.kh_before} → ${b.univ.calc.kh_after}` }} · A 대체 {{ b.univ.calc.kh_swapped.length }}과목{{ b.univ.calc.kh_swapped.length ? ' (' + b.univ.calc.kh_swapped.join(', ') + ')' : '' }}{{ b.univ.calc.kh_excluded.length ? ' · 미반영 ' + b.univ.calc.kh_excluded.join(', ') : '' }}</span></div>
            <div class="uc"><b>동국대 · 결합형</b><span class="uv">{{ b.univ.calc.dongguk ?? '—' }}<small>/10</small></span><span class="us">상위 {{ b.univ.calc.dongguk_used.length }}과목 영역점수 평균 (후보 {{ b.univ.calc.dongguk_total }}){{ b.univ.dongguk_top != null ? ` · 교내 상위 ${b.univ.dongguk_top}%` : '' }}</span></div>
          </div>
        </div>

        <!-- 메모 -->
        <div class="panel no-print">
          <div class="panel-head"><div><h3 class="panel-title">상담 메모</h3><p class="panel-desc">수치에서 자동으로 만든 초안입니다. 고쳐서 복사하거나 엑셀에 함께 저장됩니다.</p></div>
            <div class="row"><button class="btn-ghost" @click="showPayload = !showPayload">해석용 JSON {{ showPayload ? '닫기' : '보기' }}</button><button class="btn-soft" @click="copyMemo"><Copy :size="14" /> 복사</button></div></div>
          <textarea v-model="memoEdit" class="memo" rows="14" />
          <pre v-if="showPayload" class="payload">{{ JSON.stringify(b.ai_payload, null, 1) }}</pre>
        </div>
        <div class="note no-print"><ShieldOff :size="14" /><div>{{ b.notes.join(' ') }} 잔차·회귀 예측값·확률은 이 화면에 없습니다. 해석용 JSON에는 이름·학번이 들어가지 않습니다 (AI 연동은 보류).</div></div>
      </template>
      <div v-else-if="analysis.loading" class="placeholder">계산 중…</div>
    </div>
  </div>
</template>

<style scoped>
.compact { padding: 12px 24px; align-items: center; }
.hint { font-size: 12px; color: var(--tx-4); margin-left: 6px; }
.btn-sm { padding: 7px 12px; font-size: 13px; display: inline-flex; align-items: center; gap: 6px; }
.section-body { padding: 14px 24px 40px; gap: 14px; }
.head-row { display: flex; gap: 18px; align-items: flex-start; flex-wrap: wrap; }
.who { flex: 1; min-width: 260px; }
.nm { font-size: 24px; font-weight: 800; color: var(--tx-1); letter-spacing: -0.02em; }
.meta { font-size: 13px; color: var(--tx-3); margin-top: 2px; }
.tags { margin-top: 8px; display: flex; flex-wrap: wrap; gap: 4px; }
.chip { display: inline-block; padding: 2px 9px; border-radius: 999px; font-size: 12px; border: 1px solid var(--bd-1); background: var(--bg-2); color: var(--tx-3); }
.chip.crit { background: var(--clr-red-bg); border-color: var(--clr-red-border); color: var(--clr-red-text); }
.chip.warn { background: var(--clr-warn-bg); border-color: var(--clr-warn-border); color: var(--clr-warn-text); }
.chip.ok { background: var(--clr-green-bg); border-color: var(--clr-green-border); color: var(--clr-green-text); }
.kpis { display: grid; grid-template-columns: repeat(5, minmax(96px, 1fr)); gap: 8px; }
.k { padding: 10px 12px; border: 1px solid var(--bd-1); border-radius: 12px; background: var(--bg-2); display: flex; flex-direction: column; gap: 2px; }
.kl { font-size: 11px; color: var(--tx-4); }
.kv { font-size: 20px; font-weight: 800; color: var(--tx-1); font-variant-numeric: tabular-nums; }
.ks { font-size: 10.5px; color: var(--tx-4); }
.good { color: var(--clr-green-text); } .bad { color: var(--clr-red-text); }
.grid2 { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
@media (max-width: 1200px) { .grid2 { grid-template-columns: 1fr; } .kpis { grid-template-columns: repeat(3, 1fr); } }
.note-text { font-size: 12.5px; color: var(--tx-3); line-height: 1.6; margin: 0; }
.note-text.small { font-size: 11.5px; color: var(--tx-4); }
.trends { display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; }
.trend { border: 1px solid var(--bd-1); border-radius: 10px; padding: 8px 10px 4px; background: var(--bg-2); }
.tl { display: flex; justify-content: space-between; font-size: 12px; color: var(--tx-3); margin-bottom: 2px; }
.tl b { color: var(--tx-1); }
.ach-line { display: flex; align-items: center; gap: 10px; font-size: 12.5px; color: var(--tx-3); }
.sub td { padding: 6px 8px; font-size: 12.5px; }
.sub th { padding: 7px 8px; }
.sub .nm { font-weight: 600; color: var(--tx-1); white-space: nowrap; }
.sub .fb { min-width: 260px; white-space: normal; font-size: 12px; color: var(--tx-3); line-height: 1.45; }
.sub .fb b { color: var(--tx-1); }
.sub tr.dim td { color: var(--tx-4); }
.small { font-size: 11px; white-space: nowrap; }
.muted { color: var(--tx-4); }
.n { text-align: right; white-space: nowrap; }
.h4 { font-size: 13px; font-weight: 700; color: var(--tx-1); margin: 10px 0 6px; }
.h4 small { font-weight: 400; color: var(--tx-4); font-size: 11px; margin-left: 6px; }
.mini td { padding: 5px 8px; font-size: 12.5px; }
.ucards { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; }
@media (max-width: 1200px) { .ucards { grid-template-columns: repeat(2, 1fr); } }
.uc { border: 1px solid var(--bd-1); border-radius: 12px; padding: 12px 14px; background: var(--bg-2); display: flex; flex-direction: column; gap: 4px; }
.uc b { font-size: 13px; color: var(--tx-1); }
.uv { font-size: 22px; font-weight: 800; color: var(--accent-text); }
.uv small { font-size: 12px; color: var(--tx-4); font-weight: 500; }
.us { font-size: 11.5px; color: var(--tx-3); line-height: 1.45; }
.memo { width: 100%; padding: 12px 14px; border: 1px solid var(--bd-1); border-radius: 12px; background: var(--bg-2); color: var(--tx-1); font-family: inherit; font-size: 13px; line-height: 1.6; resize: vertical; }
.payload { font-size: 11px; background: var(--bg-2); border: 1px solid var(--bd-1); border-radius: 10px; padding: 10px; max-height: 320px; overflow: auto; color: var(--tx-3); }
</style>
