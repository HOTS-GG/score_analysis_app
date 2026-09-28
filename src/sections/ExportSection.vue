<script setup>
import { onMounted } from 'vue'
import { FileJson, FileSpreadsheet, FileText, IdCard, Lock } from 'lucide-vue-next'
import { useDatasetStore } from '../stores/dataset'

const dataset = useDatasetStore()
onMounted(() => dataset.refreshSummary())

const outputs = [
  { icon: FileText, title: '학교 리포트', fmt: 'HTML', desc: 'mu/sigma 시계열, 환율표(과목별), 등급 분포, 문항 격차 상위 목록.', audience: '교사·관리자' },
  { icon: FileSpreadsheet, title: '학년 스크리닝 명단', fmt: 'xlsx', desc: '플래그별 학생 목록. 담임이 누구를 먼저 볼지 정하는 용도.', audience: '담임 (교사용)' },
  { icon: IdCard, title: '학생 카드 묶음', fmt: 'HTML (인쇄용)', desc: '학생별 한 장. 잔차·예측값·확률은 포함되지 않습니다.', audience: '학생·학부모' },
  { icon: FileSpreadsheet, title: '원자료 + 파생지표', fmt: 'xlsx', desc: '표준 테이블과 L1~L4 파생지표 전체. 익명 ID 기준, 이름 매핑 시트 별도.', audience: '분석 담당' },
  { icon: FileJson, title: '해석용 JSON', fmt: 'json', desc: '학생 지표를 B(해석용) 프롬프트 입력 형식으로 내보냅니다. AI 연동은 보류 상태입니다.', audience: '해석 프롬프트' },
]
</script>

<template>
  <div class="section">
    <div class="section-header">
      <div>
        <h2 class="section-title">내보내기</h2>
        <div class="section-desc">
          <p>출력물 4종 + 해석용 JSON. 학생·학부모용 출력에는 교사용 지표가 들어가지 않도록 출력 단계에서 한 번 더 걸러집니다.</p>
        </div>
      </div>
    </div>

    <div class="section-body">
      <div class="cards">
        <div v-for="o in outputs" :key="o.title" class="panel out">
          <div class="out-head">
            <component :is="o.icon" :size="22" class="out-icon" />
            <div>
              <div class="out-title">{{ o.title }} <span class="badge badge--muted">{{ o.fmt }}</span></div>
              <div class="out-aud">대상: {{ o.audience }}</div>
            </div>
          </div>
          <p class="out-desc">{{ o.desc }}</p>
          <button class="btn-soft" disabled><Lock :size="14" /> 계산 엔진 구현 후 활성화</button>
        </div>
      </div>

      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title">학생 카드에 넣지 않는 것</h3><p class="panel-desc">출력 필터 화이트리스트에 없는 항목은 학생용 파일로 나가지 않습니다.</p></div></div>
        <div class="row">
          <span class="badge badge--red">잔차 (residual)</span>
          <span class="badge badge--red">z_resid</span>
          <span class="badge badge--red">회귀 예측값</span>
          <span class="badge badge--red">전이행렬 확률</span>
          <span class="badge badge--red">반 등수</span>
          <span class="badge badge--red">같은 반 친구와 비교</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 14px; }
.out { gap: 10px; }
.out-head { display: flex; align-items: center; gap: 12px; }
.out-icon { color: var(--accent-text); flex-shrink: 0; }
.out-title { font-size: 15px; font-weight: 700; color: var(--tx-1); display: flex; align-items: center; gap: 8px; }
.out-aud { font-size: 12px; color: var(--tx-4); margin-top: 2px; }
.out-desc { font-size: 13px; color: var(--tx-3); margin: 0; line-height: 1.55; flex: 1; }
</style>
