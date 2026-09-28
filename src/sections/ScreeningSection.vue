<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { AlertTriangle, Siren } from 'lucide-vue-next'
import { useDatasetStore } from '../stores/dataset'
import { useConfigStore } from '../stores/configStore'
import FilterBar from '../components/FilterBar.vue'

const emit = defineEmits(['navigate'])
const dataset = useDatasetStore()
const config = useConfigStore()
const filters = ref({ year: '', grade: '', round: '' })
const population = ref('all')
const activeFlags = ref(new Set())

onMounted(() => Promise.all([dataset.refreshSummary(), config.loadThresholds()]))
watch(() => dataset.filterOptions, (o) => {
  filters.value = { year: o.years.at(-1) ?? '', grade: o.grades[0] ?? '', round: o.rounds.at(-1) ?? '' }
}, { immediate: true })

const t = computed(() => config.thresholds ?? {})

/** 스펙 4장 플래그 12개. 각 플래그가 필요한 데이터를 함께 표시. */
const flags = computed(() => [
  { id: 'consecutive_drop', label: '연속 하락', rule: `Δ(3→6) ≤ ${t.value.consecutive_drop_delta} AND Δ(6→9) ≤ ${t.value.consecutive_drop_delta}`, needs: '모의고사 3회', ok: dataset.mockRoundCount >= 3, color: 'red' },
  { id: 'sharp_drop', label: '급락', rule: `어느 구간이든 Δ ≤ ${t.value.sharp_drop_delta}`, needs: '모의고사 2회', ok: dataset.mockRoundCount >= 2, color: 'red' },
  { id: 'subject_gap', label: '과목 불균형', rule: `subject_gap ≥ ${t.value.subject_gap_min}`, needs: '모의고사 1회', ok: dataset.hasMock, color: 'warn' },
  { id: 'volatility', label: '기복', rule: `volatility ≥ ${t.value.volatility_min}`, needs: '모의고사 3회', ok: dataset.mockRoundCount >= 3, color: 'warn' },
  { id: 'inconsistency', label: '유형 불일치', rule: `|consistency| ≥ ${t.value.consistency_abs_min} (교내 석차백분율 차)`, needs: '모의고사 + 내신', ok: dataset.hasMock && dataset.hasSchool, color: 'purple' },
  { id: 'undervalued_growth', label: '저평가 성장', rule: `z_resid ≥ +${t.value.z_resid_growth} 이 ${t.value.z_resid_growth_count}회 이상`, needs: '모의고사 + 입학 시점', ok: dataset.hasMock && dataset.hasEntry, color: 'green' },
  { id: 'quiet_drop', label: '조용한 하락 ★', rule: `z_resid ≤ ${t.value.z_resid_quiet_drop} 인데 등급 유지`, needs: '모의고사 + 입학 시점', ok: dataset.hasMock && dataset.hasEntry, color: 'red' },
  { id: 'adaptation_fail', label: '적응 실패', rule: `반배치 상위 ${t.value.adaptation_entry_top_pct}% → 9월 교내 하위 ${t.value.adaptation_sept_bottom_pct}%`, needs: '입학 시점 + 9월 모의고사', ok: dataset.hasEntry && dataset.hasMock, color: 'red' },
  { id: 'group_illusion', label: '집단 착시 ★', rule: `성취도 ${(t.value.group_illusion_achievements ?? []).join('~')} 인데 내신 ${(t.value.group_illusion_grades ?? []).join('~')}등급`, needs: '내신(성취도+등급)', ok: dataset.hasSchool, color: 'red' },
  { id: 'cutline', label: '경계선', rule: `cutline_margin ≤ ${t.value.cutline_margin_max}`, needs: '모의고사 1회', ok: dataset.hasMock, color: 'warn' },
  { id: 'careless', label: '실수형', rule: `전국 정답률 ${t.value.easy_item_national_rate}%↑ 문항 실점이 총 실점의 ${t.value.careless_easy_item_share}% 이상`, needs: '문항 응답(학생별)', ok: dataset.hasItemResponses, color: 'purple' },
  { id: 'insincere', label: '성실도 의심', rule: `무응답 ≥ ${t.value.sincerity_blank_ratio}% 등 전처리 3) 조건`, needs: '모의고사(무응답 수) 또는 문항 응답', ok: dataset.hasMock, color: 'muted' },
])

function toggleFlag(id) {
  const s = new Set(activeFlags.value)
  s.has(id) ? s.delete(id) : s.add(id)
  activeFlags.value = s
}
</script>

<template>
  <div class="section">
    <div class="section-header">
      <div>
        <h2 class="section-title">학년 스크리닝 명단</h2>
        <div class="section-desc">
          <p>담임이 <b>누구를 먼저 볼지</b> 정하는 화면입니다. 플래그를 눌러 해당 학생만 거릅니다. 기준값은 설정에서 바꿉니다.</p>
          <p>이 화면의 잔차(z_resid)·회귀 예측값은 교사용입니다. 학생 카드와 학생·학부모용 출력에는 나오지 않습니다.</p>
        </div>
      </div>
      <div class="header-actions">
        <button class="btn-ghost" @click="emit('navigate', 'settings')">기준값 조정</button>
      </div>
    </div>

    <div class="section-body">
      <FilterBar v-model="filters" :show="['year', 'grade', 'round']">
        <div class="filter-field">
          <span class="filter-label">모집단</span>
          <div class="seg">
            <button :class="{ on: population === 'all' }" @click="population = 'all'">전체</button>
            <button :class="{ on: population === 'sincere' }" @click="population = 'sincere'">성실 응시자만</button>
          </div>
        </div>
      </FilterBar>

      <!-- 플래그 칩 -->
      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title"><Siren :size="16" /> 플래그</h3><p class="panel-desc">칩을 누르면 명단이 해당 플래그로 걸러집니다. 회색 칩은 필요한 데이터가 아직 없는 플래그입니다.</p></div></div>
        <div class="chips">
          <button
            v-for="f in flags" :key="f.id" class="chip" :class="[`chip--${f.color}`, { on: activeFlags.has(f.id), off: !f.ok }]"
            :title="`${f.rule}\n필요 데이터: ${f.needs}`" @click="f.ok && toggleFlag(f.id)"
          >
            <span class="chip-label">{{ f.label }}</span>
            <span class="chip-count">{{ f.ok ? '—' : '데이터 없음' }}</span>
          </button>
        </div>
        <div class="rules">
          <div v-for="f in flags" :key="f.id" class="rule" :class="{ dim: !f.ok }">
            <span class="rule-name">{{ f.label }}</span>
            <span class="rule-text">{{ f.rule }}</span>
            <span class="badge" :class="f.ok ? 'badge--green' : 'badge--muted'">{{ f.needs }}</span>
          </div>
        </div>
      </div>

      <!-- 명단 -->
      <div class="panel">
        <div class="panel-head">
          <div>
            <h3 class="panel-title">명단 <span class="badge badge--muted">{{ activeFlags.size ? [...activeFlags].length + '개 플래그 필터' : '전체' }}</span></h3>
            <p class="panel-desc">클릭하면 학생 카드로 이동합니다. 정렬 기본값은 플래그 개수 내림차순.</p>
          </div>
        </div>
        <div class="table-wrap">
          <table class="data-table">
            <thead>
              <tr>
                <th>익명 ID</th><th>반</th><th>번호</th><th>플래그</th>
                <th class="num">3월</th><th class="num">6월</th><th class="num">9월</th>
                <th class="num">Δ(3→9)</th><th class="num">기울기</th><th class="num">과목격차</th><th>최저 과목</th>
                <th class="num">consistency</th><th class="num">z_resid ×3</th><th>4분면</th>
              </tr>
            </thead>
            <tbody>
              <tr><td colspan="14" class="muted" style="text-align:center; padding: 28px">
                <template v-if="!dataset.hasMock">모의고사 데이터가 들어오면 명단이 나타납니다.</template>
                <template v-else>데이터는 준비되었습니다. 계산 엔진(2단계)에서 채워집니다.</template>
              </td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="note note--warn">
        <AlertTriangle :size="15" />
        <div>1회 잔차로 판정하지 않습니다. 세 잔차의 부호가 일관될 때만 "성장/이탈"로 표시됩니다. 반별 평균 잔차 순위 같은 학급 평가 집계는 제공하지 않습니다.</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.chips { display: flex; flex-wrap: wrap; gap: 8px; }
.chip { display: inline-flex; flex-direction: column; align-items: flex-start; gap: 2px; padding: 8px 12px; border-radius: 12px; border: 1px solid var(--bd-1); background: var(--bg-2); cursor: pointer; min-width: 110px; text-align: left; transition: transform 0.1s; }
.chip:hover:not(.off) { transform: translateY(-1px); }
.chip.off { opacity: 0.45; cursor: not-allowed; }
.chip-label { font-size: 13px; font-weight: 700; color: var(--tx-1); }
.chip-count { font-size: 12px; color: var(--tx-4); }
.chip--red.on { border-color: var(--clr-red-border); background: var(--clr-red-bg); }
.chip--warn.on { border-color: var(--clr-warn-border); background: var(--clr-warn-bg); }
.chip--purple.on { border-color: var(--clr-purple-border); background: var(--clr-purple-bg); }
.chip--green.on { border-color: var(--clr-green-border); background: var(--clr-green-bg); }
.chip--muted.on { border-color: var(--bd-2); background: var(--bg-hover); }
.rules { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px 14px; }
.rule { display: grid; grid-template-columns: 110px 1fr auto; align-items: center; gap: 10px; font-size: 12px; padding: 6px 0; border-bottom: 1px dashed var(--bd-1); }
.rule.dim { opacity: 0.55; }
.rule-name { font-weight: 700; color: var(--tx-2); }
.rule-text { color: var(--tx-3); }
@media (max-width: 1100px) { .rules { grid-template-columns: 1fr; } }
</style>
