<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { AlertTriangle, ArrowDown, ArrowUp } from 'lucide-vue-next'
import { useContextStore } from '../stores/context'
import { useAnalysisStore } from '../stores/analysis'
import GradeBadge from '../components/GradeBadge.vue'
import AchStack from '../components/AchStack.vue'
import PolyRadar from '../components/charts/PolyRadar.vue'

const emit = defineEmits(['navigate'])
const ctx = useContextStore()
const analysis = useAnalysisStore()
const sort = ref({ k: 'number', dir: 1 })
const AXES = ['국어', '수학', '영어', '사회', '과학', '기타']

async function load() {
  if (!ctx.grade || !ctx.classNo) return
  await analysis.fetchClassReport(ctx.gradeYear, ctx.grade, ctx.classNo, ctx.term, ctx.scope)
}
onMounted(load)
watch(() => [ctx.gradeYear, ctx.grade, ctx.classNo, ctx.term, ctx.scope], load)

const r = computed(() => analysis.classReport)
const rows = computed(() => {
  const list = [...(r.value?.rows ?? [])]
  const { k, dir } = sort.value
  const val = (m) => ({
    number: m.number, name: m.name, gpa_main: m.gpa_main ?? 99, gpa_all: m.gpa_all ?? 99, pos_pct: m.pos_pct ?? 999,
    a: -m.ach.a, e: -m.ach.e, delta: m.pos_delta ?? -999, entry: m.entry_move ?? -999,
  })[k]
  list.sort((a, b) => { const va = val(a), vb = val(b); return typeof va === 'string' ? dir * va.localeCompare(vb, 'ko') : dir * (va - vb) })
  return list
})
function setSort(k) { sort.value = sort.value.k === k ? { k, dir: -sort.value.dir } : { k, dir: 1 } }
const diff = computed(() => (r.value?.class_gpa != null && r.value?.grade_gpa != null) ? r.value.class_gpa - r.value.grade_gpa : null)
const fmt = (v, d = 2) => v == null ? '—' : Number(v).toFixed(d)
const cols = [
  ['number', '학번', 'n'], ['name', '이름', ''], ['gpa_main', '주요교과', 'n'], [null, '국·수·영·사·과·기타', ''], ['gpa_all', '전과목', 'n'],
  ['pos_pct', '교내 위치', 'n'], [null, '성취도 A→E', ''], ['a', 'A', 'n'], ['e', 'E', 'n'], ['delta', '직전 대비', 'n'], ['entry', '입학 대비', 'n'], [null, '눈여겨볼 점', ''],
]
</script>

<template>
  <div class="section">
    <div class="section-header compact">
      <div>
        <h2 class="section-title">학급 한눈에 <span class="ctxl">{{ ctx.contextLabel }} · {{ r?.term_label ?? '' }}</span></h2>
        <div class="section-desc"><p>행을 누르면 그 학생의 상담 리포트로 이동합니다. 등급 평균은 표시용, <b>교내 위치·직전 대비·입학 대비는 석차백분율 환산값</b>입니다.</p></div>
      </div>
    </div>
    <div class="section-body">
      <p v-if="analysis.error" class="msg-error">{{ analysis.error }}</p>
      <div v-if="!ctx.contexts.length" class="empty-state">
        <p class="empty-title">아직 학년 데이터가 없습니다</p>
        <p class="empty-desc">데이터 → 가져오기에서 명단과 나이스 교과학습발달상황 파일을 넣으면 여기에 학급이 나타납니다.</p>
        <button class="btn-add" @click="emit('navigate', 'data')">데이터 가져오기</button>
      </div>
      <template v-else-if="r">
        <div class="kpi-row">
          <div class="kpi"><span class="kpi-label">학급 인원</span><span class="kpi-value">{{ r.class_n }}<small>명</small></span><span class="kpi-sub">{{ r.grade }}학년 {{ r.grade_n }}명 중</span></div>
          <div class="kpi"><span class="kpi-label">학급 등급평균</span><span class="kpi-value">{{ fmt(r.class_gpa) }}</span><span class="kpi-sub">{{ r.scope === 'main' ? '국·수·영·사·과 학점 가중' : '등급 산출 전 과목' }} · 표시용</span></div>
          <div class="kpi" :class="{ 'kpi--warn': diff != null && diff > 0.2 }"><span class="kpi-label">학년 대비</span><span class="kpi-value">{{ diff == null ? '—' : (diff < 0 ? '−' : '+') + Math.abs(diff).toFixed(2) }}</span><span class="kpi-sub">{{ diff == null ? '' : diff < 0 ? '학년 평균보다 상위' : '학년 평균보다 하위' }}</span></div>
          <div class="kpi"><span class="kpi-label">학급 교내 위치 평균</span><span class="kpi-value">{{ r.class_pos_mean == null ? '—' : '상위 ' + fmt(r.class_pos_mean, 1) + '%' }}</span><span class="kpi-sub">석차백분율 환산</span></div>
          <div class="kpi" :class="{ 'kpi--warn': r.watch.length }"><span class="kpi-label">먼저 만나 볼 학생</span><span class="kpi-value">{{ r.watch.length }}<small>명</small></span><span class="kpi-sub">기준값에 걸린 학생</span></div>
        </div>

        <div class="table-wrap">
          <table class="data-table cls">
            <thead><tr>
              <th v-for="[k, l, c] in cols" :key="l" :class="[c, { sortable: k }]" @click="k && setSort(k)">{{ l }}<span v-if="sort.k === k" class="arw">{{ sort.dir > 0 ? '▲' : '▼' }}</span></th>
            </tr></thead>
            <tbody>
              <tr v-for="m in rows" :key="m.student_id" class="pick" @click="ctx.openStudent(m.student_id)">
                <td class="n mono">{{ m.hakbun }}</td>
                <td class="nm">{{ m.name || m.anon_id }}</td>
                <td class="n"><GradeBadge :value="m.gpa_main" /></td>
                <td><span class="strip"><GradeBadge v-for="(v, i) in m.radar" :key="i" :value="v" :digits="1" :title="AXES[i]" /></span></td>
                <td class="n">{{ fmt(m.gpa_all) }}</td>
                <td class="n">{{ m.pos_pct == null ? '—' : '상위 ' + m.pos_pct.toFixed(1) + '%' }}</td>
                <td><AchStack :counts="m.ach" mini /></td>
                <td class="n">{{ m.ach.a }}</td>
                <td class="n" :class="{ bad: m.ach.e >= 3 }">{{ m.ach.e }}</td>
                <td class="n" :class="{ good: m.pos_delta != null && m.pos_delta <= -5, bad: m.pos_delta != null && m.pos_delta >= 5 }">
                  <template v-if="m.pos_delta == null">—</template><template v-else><ArrowUp v-if="m.pos_delta < 0" :size="12" /><ArrowDown v-else-if="m.pos_delta > 0" :size="12" />{{ Math.abs(m.pos_delta).toFixed(0) }}p</template>
                </td>
                <td class="n" :class="{ good: m.entry_move >= 8, bad: m.entry_move <= -8 }" :title="m.entry_move == null ? '입학 성적 미연결' : `입학 상위 ${m.entry_top}% → 현재 상위 ${m.pos_pct}%`">
                  <template v-if="m.entry_move == null">—</template><template v-else><ArrowUp v-if="m.entry_move > 0" :size="12" /><ArrowDown v-else-if="m.entry_move < 0" :size="12" />{{ Math.abs(m.entry_move).toFixed(0) }}p</template>
                </td>
                <td><span v-for="t in m.tags" :key="t.text" class="chip" :class="t.level">{{ t.text }}</span><span v-if="!m.tags.length" class="muted">—</span></td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="panel-grid three">
          <div class="panel">
            <div class="panel-head"><div><h3 class="panel-title">교과별 균형</h3><p class="panel-desc">우리 반 vs 학년 전체 (등급평균, 표시용)</p></div></div>
            <PolyRadar :axes="AXES" :series="[{ name: '학년 전체', values: r.radar_grade, color: 'var(--clr-note-text)', dashed: true }, { name: `우리 반 (${r.class_no}반)`, values: r.radar_class, fill: true, label: true }]" mode="grade" />
          </div>
          <div class="panel">
            <div class="panel-head"><div><h3 class="panel-title">등급 분포</h3><p class="panel-desc">우리 반이 어디에 몰려 있는지 (반 명수 vs 학년 %)</p></div></div>
            <div class="dist">
              <div v-for="b in r.dist" :key="b.grade" class="drow">
                <span class="dl"><GradeBadge :value="b.grade" :digits="0" /> {{ b.grade }}등급대</span>
                <span class="dtrack">
                  <span class="dfill" :style="{ width: Math.min(100, b.class_pct / Math.max(1, ...r.dist.map(x => Math.max(x.class_pct, x.grade_pct))) * 100) + '%' }" />
                  <span class="dfill alt" :style="{ width: Math.min(100, b.grade_pct / Math.max(1, ...r.dist.map(x => Math.max(x.class_pct, x.grade_pct))) * 100) + '%' }" />
                </span>
                <span class="dv">{{ b.class_n }}명<br><small>{{ b.grade_pct }}%</small></span>
              </div>
            </div>
          </div>
          <div class="panel">
            <div class="panel-head"><div><h3 class="panel-title">먼저 만나 볼 학생</h3><p class="panel-desc">수치에서 자동 추출한 상담 우선순위</p></div></div>
            <div v-if="!r.watch.length" class="placeholder" style="min-height:80px">이 기준에 걸린 학생이 없습니다.</div>
            <div v-else class="watch">
              <button v-for="w in r.watch" :key="w.student_id" class="witem" @click="ctx.openStudent(w.student_id)">
                <GradeBadge :value="w.gpa_main" :digits="1" />
                <span><b>{{ w.number }}. {{ w.name }}</b><small>{{ w.reasons.join(' · ') }}</small></span>
              </button>
            </div>
          </div>
        </div>
        <div class="note"><AlertTriangle :size="14" /><div>{{ r.notes.join(' ') }} 반별·교사별 비교 집계는 제공하지 않습니다.</div></div>
      </template>
      <div v-else-if="analysis.loading" class="placeholder">계산 중…</div>
    </div>
  </div>
</template>

<style scoped>
.compact { padding: 18px 28px 12px; }
.ctxl { font-size: 13px; font-weight: 500; color: var(--tx-4); margin-left: 8px; }
.section-body { padding: 16px 28px 40px; }
.cls th.sortable { cursor: pointer; user-select: none; }
.cls th.sortable:hover { color: var(--accent-text); }
.arw { margin-left: 3px; font-size: 9px; }
.cls td { padding: 7px 10px; font-size: 13px; }
.cls th { padding: 8px 10px; }
.pick { cursor: pointer; }
.nm { font-weight: 600; color: var(--tx-1); white-space: nowrap; }
.strip { display: inline-flex; gap: 2px; }
.strip :deep(.gb) { min-width: 26px; padding: 1px 3px; font-size: 10.5px; }
.n { text-align: right; white-space: nowrap; }
.good { color: var(--clr-green-text); font-weight: 600; }
.bad { color: var(--clr-red-text); font-weight: 600; }
.muted { color: var(--tx-4); }
.chip { display: inline-block; padding: 1px 7px; margin: 1px 3px 1px 0; border-radius: 999px; font-size: 11px; border: 1px solid var(--bd-1); background: var(--bg-2); color: var(--tx-3); white-space: nowrap; }
.chip.crit { background: var(--clr-red-bg); border-color: var(--clr-red-border); color: var(--clr-red-text); }
.chip.warn { background: var(--clr-warn-bg); border-color: var(--clr-warn-border); color: var(--clr-warn-text); }
.chip.ok { background: var(--clr-green-bg); border-color: var(--clr-green-border); color: var(--clr-green-text); }
.three { grid-template-columns: repeat(3, minmax(0, 1fr)); }
@media (max-width: 1300px) { .three { grid-template-columns: 1fr; } }
.dist { display: flex; flex-direction: column; gap: 8px; }
.drow { display: grid; grid-template-columns: 96px 1fr 54px; gap: 10px; align-items: center; font-size: 12.5px; }
.dl { display: flex; align-items: center; gap: 6px; color: var(--tx-2); }
.dtrack { position: relative; height: 18px; background: var(--bg-2); border: 1px solid var(--bd-1); border-radius: 4px; overflow: hidden; }
.dfill { position: absolute; left: 0; top: 0; height: 9px; background: var(--accent-hex); }
.dfill.alt { top: 9px; background: var(--clr-note-text); opacity: 0.7; }
.dv { text-align: right; color: var(--tx-2); line-height: 1.15; }
.dv small { color: var(--clr-note-text); }
.watch { display: flex; flex-direction: column; gap: 8px; }
.witem { display: flex; align-items: flex-start; gap: 10px; padding: 9px 11px; border-radius: 10px; border: 1px solid var(--bd-1); background: var(--bg-2); cursor: pointer; text-align: left; }
.witem:hover { border-color: rgba(var(--accent-rgb), 0.5); }
.witem span { display: flex; flex-direction: column; gap: 2px; }
.witem b { font-size: 13px; color: var(--tx-1); }
.witem small { font-size: 11.5px; color: var(--tx-3); line-height: 1.4; }
</style>
