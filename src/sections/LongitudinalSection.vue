<script setup>
import { computed, onMounted } from 'vue'
import { AlertTriangle } from 'lucide-vue-next'
import { useDatasetStore } from '../stores/dataset'
import { useConfigStore } from '../stores/configStore'
import PanelCard from '../components/PanelCard.vue'

const dataset = useDatasetStore()
const config = useConfigStore()
onMounted(() => Promise.all([dataset.refreshSummary(), config.loadThresholds()]))

/** 고1 3월과 고3 9월이 모두 있는 코호트가 있어야 계산 가능 */
const cohorts = computed(() => {
  const rounds = dataset.summary?.mock_rounds ?? []
  const byCohort = {}
  for (const r of rounds) {
    const c = r.grade_year - r.grade + 1
    byCohort[c] ??= { cohort: c, g1_3: false, g3_9: false, rounds: 0 }
    byCohort[c].rounds++
    if (r.grade === 1 && r.round === 3) byCohort[c].g1_3 = true
    if (r.grade === 3 && r.round === 9) byCohort[c].g3_9 = true
  }
  return Object.values(byCohort).sort((a, b) => a.cohort - b.cohort)
})
const ready = computed(() => cohorts.value.some(c => c.g1_3 && c.g3_9))
const st = computed(() => ready.value ? 'pending' : 'nodata')
const REQ = '고1 3월과 고3 9월 학평이 모두 있는 과거 코호트가 필요합니다.'
const bins = ['상위 ~10%', '10~25%', '25~50%', '50~75%', '75%~']
const cellMin = computed(() => config.thresholds?.transition_cell_min ?? 10)
</script>

<template>
  <div class="section">
    <div class="section-header">
      <div>
        <h2 class="section-title">종단 분석</h2>
        <div class="section-desc">
          <p>L6. 과거 코호트가 있을 때 고1 3월 위치가 고3 9월에 어디로 갔는지(전이행렬)와, 상승군·하락군을 고1 시점에 가른 지표를 찾습니다.</p>
          <p>셀 인원이 {{ cellMin }}명 미만이면 비율을 표시하지 않고 "표본 부족"으로 둡니다.</p>
        </div>
      </div>
    </div>

    <div class="section-body">
      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title">코호트 준비 상태</h3><p class="panel-desc">모의고사 회차에서 입학연도별로 자동 판정합니다.</p></div></div>
        <div v-if="!cohorts.length" class="placeholder" style="min-height:80px">모의고사 데이터가 없습니다.</div>
        <div v-else class="table-wrap">
          <table class="data-table">
            <thead><tr><th>입학연도</th><th class="num">보유 회차</th><th>고1 3월</th><th>고3 9월</th><th>종단 분석</th></tr></thead>
            <tbody>
              <tr v-for="c in cohorts" :key="c.cohort">
                <td>{{ c.cohort }}</td><td class="num">{{ c.rounds }}</td>
                <td><span class="badge" :class="c.g1_3 ? 'badge--green' : 'badge--muted'">{{ c.g1_3 ? '있음' : '없음' }}</span></td>
                <td><span class="badge" :class="c.g3_9 ? 'badge--green' : 'badge--muted'">{{ c.g3_9 ? '있음' : '없음' }}</span></td>
                <td><span class="badge" :class="c.g1_3 && c.g3_9 ? 'badge--accent' : 'badge--muted'">{{ c.g1_3 && c.g3_9 ? '가능' : '불가' }}</span></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="panel">
        <div class="panel-head">
          <div>
            <h3 class="panel-title">전이행렬 <span class="badge badge--muted">고1 3월 → 고3 9월</span></h3>
            <p class="panel-desc">행 = 고1 3월 백분위 구간, 각 행은 조건부 분포(%). 등급 이동과 백분위 변화폭을 함께 표시합니다 (평균으로의 회귀 때문).</p>
          </div>
        </div>
        <div class="table-wrap">
          <table class="data-table">
            <thead><tr><th>고1 3월 \ 고3 9월</th><th v-for="b in bins" :key="b" class="num">{{ b }}</th><th class="num">N</th><th class="num">평균 Δ백분위</th></tr></thead>
            <tbody>
              <tr v-for="b in bins" :key="b">
                <td>{{ b }}</td>
                <td v-for="b2 in bins" :key="b2" class="num muted">—</td>
                <td class="num muted">—</td><td class="num muted">—</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="placeholder" style="min-height:60px">{{ ready ? '데이터는 준비되었습니다. 계산 엔진(2단계)에서 채워집니다.' : REQ }}</div>
      </div>

      <div class="panel-grid panel-grid--2">
        <PanelCard title="상승군 vs 하락군의 고1 시점 특성" desc="과목균형 / 변동성 / consistency / slope / 출결을 비교해 유의하게 갈리는 변수를 '갈림길 지표'로 보고합니다. 효과크기와 신뢰구간 병기." tag="L6" :status="st" :requires="REQ" />
        <PanelCard title="구간별 이동 흐름" desc="고1 3월 구간에서 고3 9월 구간으로 흐르는 인원을 띠로 그립니다." tag="L6" :status="st" :requires="REQ" chart />
      </div>

      <div class="note note--warn">
        <AlertTriangle :size="15" />
        <div>고3은 재수생 유입으로 백분위가 구조적으로 밀립니다. 전이는 회차 내 상대 이동으로 계산하며, "학교 교육의 효과로 상승했다"는 식의 인과 문장은 만들지 않습니다.</div>
      </div>
    </div>
  </div>
</template>
