<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { AlertTriangle } from 'lucide-vue-next'
import { useDatasetStore } from '../stores/dataset'
import { useConfigStore } from '../stores/configStore'
import FilterBar from '../components/FilterBar.vue'
import PanelCard from '../components/PanelCard.vue'

const dataset = useDatasetStore()
const config = useConfigStore()
const filters = ref({ year: '', grade: '', round: '', subject: '' })
const method = ref('equipercentile')
const binSize = ref(10)

onMounted(() => Promise.all([dataset.refreshSummary(), config.loadThresholds()]))
watch(() => dataset.filterOptions, (o) => {
  filters.value = { year: o.years.at(-1) ?? '', grade: o.grades[0] ?? '', round: o.rounds[0] ?? '', subject: o.subjects[0] ?? '' }
}, { immediate: true })

const st = computed(() => dataset.hasMock ? 'pending' : 'nodata')
const bins = computed(() => {
  const out = []
  for (let lo = 0; lo < 100; lo += binSize.value) out.push([lo, Math.min(100, lo + binSize.value)])
  return out
})
</script>

<template>
  <div class="section">
    <div class="section-header">
      <div>
        <h2 class="section-title">교내 ↔ 전국 환율표</h2>
        <div class="section-desc">
          <p>L2. 교내 석차백분율 구간이 전국 백분위로는 어디쯤인지 보여 주는 대응표입니다. 과목별·회차별로 따로 만듭니다.</p>
          <p>내신·반배치처럼 교내 서열만 있는 지표의 전국 위치는 <b>점이 아니라 구간(사분위)</b>으로만 말합니다.</p>
        </div>
      </div>
    </div>

    <div class="section-body">
      <FilterBar v-model="filters">
        <div class="filter-field">
          <span class="filter-label">방법</span>
          <div class="seg">
            <button :class="{ on: method === 'equipercentile' }" @click="method = 'equipercentile'">등백분위 (권장)</button>
            <button :class="{ on: method === 'parametric' }" @click="method = 'parametric'">모수적 (mu + sigma·z)</button>
          </div>
        </div>
        <div class="filter-field">
          <span class="filter-label">구간 폭</span>
          <div class="seg">
            <button :class="{ on: binSize === 5 }" @click="binSize = 5">5%</button>
            <button :class="{ on: binSize === 10 }" @click="binSize = 10">10%</button>
          </div>
        </div>
      </FilterBar>

      <div class="panel">
        <div class="panel-head">
          <div>
            <h3 class="panel-title">대응표 <span class="badge badge--muted">{{ method === 'equipercentile' ? '방법2 · 등백분위' : '방법1 · 모수적' }}</span></h3>
            <p class="panel-desc">N ≥ {{ config.thresholds?.min_n_equipercentile ?? 200 }}이면 등백분위가 기본값. 두 방법의 차이를 마지막 열에 보고합니다.</p>
          </div>
        </div>
        <div class="table-wrap">
          <table class="data-table">
            <thead>
              <tr>
                <th>교내 석차백분율</th>
                <th class="num">N</th>
                <th class="num">전국 백분위 중앙값</th>
                <th class="num">Q1</th>
                <th class="num">Q3</th>
                <th class="num">모수적 추정</th>
                <th class="num">두 방법 차이</th>
                <th class="num">밀리는 폭</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="[lo, hi] in bins" :key="lo">
                <td>상위 {{ lo }}~{{ hi }}%</td>
                <td class="num muted">—</td><td class="num muted">—</td><td class="num muted">—</td><td class="num muted">—</td><td class="num muted">—</td><td class="num muted">—</td><td class="num muted">—</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div v-if="st === 'nodata'" class="placeholder" style="min-height:70px">모의고사(전국 백분위) 데이터가 들어오면 이 표가 채워집니다.</div>
        <div v-else class="placeholder" style="min-height:70px">데이터는 준비되었습니다. 계산 엔진(2단계)에서 채워집니다.</div>
      </div>

      <div class="panel-grid panel-grid--2">
        <PanelCard title="교내 → 전국 대응 곡선" desc="가로축 교내 석차백분율, 세로축 전국 백분위. 등백분위(점·사분위 띠)와 모수적 직선을 겹쳐 그립니다." :status="st" requires="모의고사 데이터가 필요합니다." chart />
        <PanelCard title="구간별 밀리는 폭" desc="같은 교내 순위라도 전국에서 밀리는 정도가 구간마다 다릅니다. sigma < 1이면 상위권 손해가 큽니다." :status="st" requires="모의고사 데이터가 필요합니다." chart />
      </div>

      <div class="note note--warn">
        <AlertTriangle :size="15" />
        <div>상수 오프셋 보정("교내 등급 −1 = 전국 등급")은 하지 않습니다. 등급끼리의 연산은 이 프로그램 어디에도 없습니다.</div>
      </div>
    </div>
  </div>
</template>
