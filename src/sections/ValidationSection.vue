<script setup>
import { computed, onMounted } from 'vue'
import { CheckCircle2, ShieldCheck } from 'lucide-vue-next'
import { useDatasetStore } from '../stores/dataset'
import { useConfigStore } from '../stores/configStore'
import PanelCard from '../components/PanelCard.vue'

const dataset = useDatasetStore()
const config = useConfigStore()
onMounted(() => Promise.all([dataset.refreshSummary(), config.loadThresholds()]))

const st = computed(() => dataset.hasMock ? 'pending' : 'nodata')
const minN = computed(() => config.thresholds?.min_n_warn ?? 30)

const unitTests = [
  { name: '등급-백분위 변환 경계값', desc: '9등급제 4/11/…/96, 5등급제 10/34/66/90 누적 경계', status: 'pass' },
  { name: '동점 순위 처리 (RANK.AVG)', desc: '동점은 평균 순위', status: 'todo' },
  { name: '결측 전파', desc: '미응시가 0으로 바뀌지 않고 결측으로 남는지', status: 'pass' },
  { name: '백분위 클리핑 경계값', desc: '0 → 0.5, 100 → 99.5', status: 'todo' },
  { name: '기준값 검증', desc: '등급컷 개수·순서, 클리핑 범위', status: 'pass' },
  { name: '가져오기 upsert', desc: '같은 키 재가져오기 시 갱신, 미매칭 학생 보고', status: 'pass' },
]
</script>

<template>
  <div class="section">
    <div class="section-header">
      <div>
        <h2 class="section-title">품질 검증</h2>
        <div class="section-desc">
          <p>스펙 7장. 계산이 안정적인지, 표본이 충분한지, 전년 회귀식이 올해를 얼마나 맞히는지 확인합니다.</p>
          <p>모든 지표에는 산출 N이 붙고, N &lt; {{ minN }}이면 경고가 표시됩니다.</p>
        </div>
      </div>
    </div>

    <div class="section-body">
      <div class="panel-grid panel-grid--2">
        <PanelCard title="홀수반 / 짝수반 안정성" desc="두 반집단으로 나눠 mu, sigma, 회귀계수를 각각 산출해 차이를 봅니다. 차이가 크면 표본 의존이 큰 것." tag="7장" :status="st" requires="모의고사 데이터가 필요합니다.">
        </PanelCard>
        <PanelCard title="전년도 회귀식 백테스트" desc="전년도 회귀식으로 올해를 예측한 뒤 MAE와 ±10 백분위 이내 적중률을 보고합니다." tag="7장" :status="dataset.hasMock && dataset.hasEntry ? 'pending' : 'nodata'" requires="두 학년도 이상의 모의고사 + 입학 시점 데이터가 필요합니다." />
        <PanelCard title="N 경고 목록" :desc="`N < ${minN}인 지표·구간 목록. 과목·회차·구간별.`" tag="7장" :status="st" requires="모의고사 데이터가 필요합니다." />
        <PanelCard title="효과크기 · 신뢰구간" desc="'유의미하다'만 쓰지 않습니다. 회귀 r², SEE, 95% 신뢰구간을 항상 함께 출력합니다." tag="6장" :status="st" requires="모의고사 데이터가 필요합니다." />
      </div>

      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title"><ShieldCheck :size="16" /> 단위 테스트</h3><p class="panel-desc">Rust 백엔드에서 자동 실행되는 테스트 목록입니다. 'todo'는 계산 엔진과 함께 추가됩니다.</p></div></div>
        <div class="table-wrap">
          <table class="data-table">
            <thead><tr><th>테스트</th><th>내용</th><th>상태</th></tr></thead>
            <tbody>
              <tr v-for="u in unitTests" :key="u.name">
                <td>{{ u.name }}</td><td class="muted">{{ u.desc }}</td>
                <td><span class="badge" :class="u.status === 'pass' ? 'badge--green' : 'badge--muted'"><CheckCircle2 v-if="u.status === 'pass'" :size="12" /> {{ u.status === 'pass' ? '통과' : '예정' }}</span></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
