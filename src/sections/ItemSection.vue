<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useDatasetStore } from '../stores/dataset'
import { useConfigStore } from '../stores/configStore'
import FilterBar from '../components/FilterBar.vue'
import PanelCard from '../components/PanelCard.vue'

const dataset = useDatasetStore()
const config = useConfigStore()
const filters = ref({ year: '', round: '', subject: '' })

onMounted(() => Promise.all([dataset.refreshSummary(), config.loadThresholds()]))
watch(() => dataset.filterOptions, (o) => {
  filters.value = { year: o.years.at(-1) ?? '', round: o.rounds.at(-1) ?? '', subject: o.subjects[0] ?? '' }
}, { immediate: true })

const t = computed(() => config.thresholds ?? {})
const statSt = computed(() => dataset.hasItems ? 'pending' : 'nodata')
const respSt = computed(() => dataset.hasItemResponses ? 'pending' : 'nodata')
const bands = computed(() => [
  { label: '쉬움', rule: `전국 정답률 ${t.value.easy_item_national_rate ?? 70}% 이상`, hint: '실점 비중 높음 → 실수·시간관리형' },
  { label: '중간', rule: `${t.value.hard_item_national_rate ?? 30}~${t.value.easy_item_national_rate ?? 70}%` },
  { label: '어려움', rule: `${t.value.hard_item_national_rate ?? 30}% 이하`, hint: '득점 있음 → 상위 실력 보유' },
])
</script>

<template>
  <div class="section">
    <div class="section-header">
      <div>
        <h2 class="section-title">문항 분석</h2>
        <div class="section-desc">
          <p>L5. 문항 통계가 있으면 학교 단위 격차·오개념을, 학생별 응답까지 있으면 실수형/실력한계형을 분류합니다.</p>
          <p>백분위가 같아도 유형이 다르면 처방이 다릅니다.</p>
        </div>
      </div>
    </div>

    <div class="section-body">
      <FilterBar v-model="filters" :show="['year', 'round', 'subject']" />

      <div class="kpi-row">
        <div class="kpi"><span class="kpi-label">문항 수</span><span class="kpi-value kpi-value--muted">—</span></div>
        <div class="kpi"><span class="kpi-label">평균 격차 (교내−전국)</span><span class="kpi-value kpi-value--muted">—</span><span class="kpi-sub">%p</span></div>
        <div class="kpi"><span class="kpi-label">격차 ≤ {{ t.item_gap_report ?? -15 }}%p 문항</span><span class="kpi-value kpi-value--muted">—</span></div>
        <div class="kpi"><span class="kpi-label">공통 오개념 후보</span><span class="kpi-value kpi-value--muted">—</span><span class="kpi-sub">오답 선지 ≥ {{ t.misconception_choice_share ?? 40 }}%</span></div>
        <div class="kpi"><span class="kpi-label">시간 부족 의심</span><span class="kpi-value kpi-value--muted">—</span><span class="kpi-sub">후반부 정답률 급락</span></div>
      </div>

      <div class="panel-grid panel-grid--2">
        <PanelCard title="문항별 격차" desc="item_gap = 교내 정답률 − 전국 정답률. 문항 번호순 막대, 기준 이하 강조." tag="L5" :status="statSt" requires="문항 통계 데이터가 필요합니다." chart />
        <PanelCard title="단원 · 유형별 집계" desc="격차를 단원/유형으로 묶어 집중 구간을 보고합니다." tag="L5" :status="statSt" requires="문항 통계(단원·유형 컬럼)가 필요합니다." />
        <PanelCard title="공통 오개념 후보" desc="오답이 특정 선지에 기준 이상 몰린 문항 목록과 그 선지." tag="L5" :status="statSt" requires="문항 통계(선지별 응답 분포)가 필요합니다." />
        <PanelCard title="시간 부족 플래그" desc="후반부 문항 정답률이 전반부 대비 급락하는 학생·문항 구간." tag="L5" :status="respSt" requires="문항 응답(학생별) 데이터가 필요합니다." chart />
      </div>

      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title">난이도대별 프로파일</h3><p class="panel-desc">전국 정답률 기준 3구간. 학생별로 어느 구간에서 실점하는지에 따라 유형을 분류합니다.</p></div></div>
        <div class="bands">
          <div v-for="b in bands" :key="b.label" class="band">
            <div class="band-label">{{ b.label }}</div>
            <div class="band-rule">{{ b.rule }}</div>
            <div class="band-val muted">—</div>
            <div v-if="b.hint" class="band-hint">{{ b.hint }}</div>
          </div>
        </div>
        <div class="table-wrap">
          <table class="data-table">
            <thead><tr><th>익명 ID</th><th>반</th><th>번호</th><th class="num">백분위</th><th class="num">쉬운 문항 실점 비중</th><th class="num">어려운 문항 득점</th><th>유형</th><th>처방 방향</th></tr></thead>
            <tbody>
              <tr><td colspan="8" class="muted" style="text-align:center; padding:24px">
                <template v-if="!dataset.hasItemResponses">학생별 문항 응답이 없어 유형 판정 불가입니다.</template>
                <template v-else>데이터는 준비되었습니다. 계산 엔진(2단계)에서 채워집니다.</template>
              </td></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bands { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
.band { padding: 12px 14px; border-radius: 12px; border: 1px solid var(--bd-1); background: var(--bg-2); display: flex; flex-direction: column; gap: 3px; }
.band-label { font-size: 14px; font-weight: 700; color: var(--tx-1); }
.band-rule { font-size: 12px; color: var(--tx-3); }
.band-val { font-size: 20px; font-weight: 700; margin-top: 4px; }
.band-hint { font-size: 12px; color: var(--accent-text); }
</style>
