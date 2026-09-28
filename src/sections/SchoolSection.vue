<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { AlertTriangle } from 'lucide-vue-next'
import { useDatasetStore } from '../stores/dataset'
import FilterBar from '../components/FilterBar.vue'
import PanelCard from '../components/PanelCard.vue'

const dataset = useDatasetStore()
const filters = ref({ year: '', grade: '', round: '', subject: '' })
const population = ref('all') // all | sincere

onMounted(() => dataset.refreshSummary())
watch(() => dataset.filterOptions, (o) => {
  filters.value = { year: o.years.at(-1) ?? '', grade: o.grades[0] ?? '', round: o.rounds[0] ?? '', subject: o.subjects[0] ?? '' }
}, { immediate: true })

const st = computed(() => dataset.hasMock ? 'pending' : 'nodata')
const REQ = '모의고사(전국 백분위) 데이터가 필요합니다.'

const kpis = [
  { label: 'N (응시자)', key: 'n' },
  { label: 'mu · 학교의 전국 레벨', key: 'mu', hint: 'mean(Φ⁻¹(pct/100))' },
  { label: 'sigma · 밀집도', key: 'sigma', hint: 'sd(z_nat). <1이면 상위권 손해 큼' },
  { label: '왜도 (skew)', key: 'skew', hint: '|skew| ≥ 0.5 → 정규 가정 경고' },
  { label: '1~2등급 비율', key: 'top' },
  { label: '4등급 이하 비율', key: 'bottom' },
]
</script>

<template>
  <div class="section">
    <div class="section-header">
      <div>
        <h2 class="section-title">학교 리포트</h2>
        <div class="section-desc">
          <p>L0·L7. 모의고사 전국 백분위만 사용해 학년 전체의 전국 좌표(mu)와 밀집도(sigma)를 회차별·과목별로 봅니다.</p>
          <p>mu 상승 = 학년 전체가 전국 대비 상승. sigma 축소 = 하위권이 따라붙음, 확대 = 양극화.</p>
        </div>
      </div>
    </div>

    <div class="section-body">
      <FilterBar v-model="filters">
        <div class="filter-field">
          <span class="filter-label">모집단</span>
          <div class="seg">
            <button :class="{ on: population === 'all' }" @click="population = 'all'">전체</button>
            <button :class="{ on: population === 'sincere' }" @click="population = 'sincere'">성실 응시자만</button>
          </div>
        </div>
      </FilterBar>

      <div class="kpi-row">
        <div v-for="k in kpis" :key="k.key" class="kpi">
          <span class="kpi-label">{{ k.label }}</span>
          <span class="kpi-value kpi-value--muted">—</span>
          <span v-if="k.hint" class="kpi-sub">{{ k.hint }}</span>
        </div>
      </div>

      <div class="panel-grid panel-grid--2">
        <PanelCard title="mu · sigma 시계열" desc="3→6→9월 및 전년 동회차 대비 변화. 과목별 선 그래프." tag="L7" :status="st" :requires="REQ" chart />
        <PanelCard title="전국 z 분포" desc="학교 z_nat 히스토그램과 정규곡선. 왜도가 크면 환율표에서 등백분위 방식을 씁니다." tag="L0" :status="st" :requires="REQ" chart />
        <PanelCard title="등급 분포 변화" desc="회차별 1~2등급 비율, 4등급 이하 비율 (표시용 9등급 변환)." tag="L7" :status="st" :requires="REQ" />
        <PanelCard title="잔차 분포의 꼬리" desc="부가가치 잔차 분포의 양쪽 꼬리 변화. 회귀 가드(N≥100, r≥0.4) 통과 시에만." tag="L4·L7" :status="dataset.hasMock && dataset.hasEntry ? 'pending' : 'nodata'" requires="모의고사 + 입학 시점(반배치/고입내신) 데이터가 필요합니다." chart />
        <PanelCard title="문항 격차 상위 목록" desc="교내 정답률 − 전국 정답률이 가장 낮은 문항·단원. 기준값 이하 집중 구간 보고." tag="L5" :status="dataset.hasItems ? 'pending' : 'nodata'" requires="문항 통계 데이터가 필요합니다." span />
      </div>

      <div class="note">
        <AlertTriangle :size="15" />
        <div>이 화면은 학교 단위 성과를 보는 곳입니다. 반별·교사별 비교 집계는 제공하지 않습니다.</div>
      </div>
    </div>
  </div>
</template>
