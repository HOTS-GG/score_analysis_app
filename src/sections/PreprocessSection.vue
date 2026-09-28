<script setup>
import { computed, onMounted } from 'vue'
import { AlertTriangle, RotateCcw, UserMinus } from 'lucide-vue-next'
import { useDatasetStore } from '../stores/dataset'
import { useStudentStore } from '../stores/student'
import { useConfigStore } from '../stores/configStore'
import PanelCard from '../components/PanelCard.vue'

const emit = defineEmits(['navigate'])
const dataset = useDatasetStore()
const students = useStudentStore()
const config = useConfigStore()

onMounted(async () => {
  await Promise.all([dataset.refreshSummary(), students.fetchExclusionLog(), config.loadThresholds()])
})

const s = computed(() => dataset.summary)
const t = computed(() => config.thresholds)
const mockStatus = computed(() => dataset.hasMock ? 'pending' : 'nodata')

const rules = computed(() => [
  { title: '결측 처리', desc: '미응시를 0점으로 치환하지 않습니다. 가져오기에서 빈칸·"-"·"결시"는 결측으로 저장됩니다.', status: 'ok', badge: '가져오기에서 적용됨' },
  { title: '분석 대상', desc: '회차별 통계는 해당 회차 응시자. 추이 분석은 3회 중 2회 이상 응시자.', status: dataset.mockRoundCount >= 2 ? 'ok' : 'wait', badge: dataset.mockRoundCount >= 2 ? `회차 ${dataset.mockRoundCount}개` : `회차 2개 이상 필요 (현재 ${dataset.mockRoundCount})` },
  { title: '백분위 클리핑', desc: `z 변환 시 0/100 오류를 막기 위해 ${t.value?.pct_clip_low ?? 0.5} / ${t.value?.pct_clip_high ?? 99.5}로 클리핑합니다.`, status: 'ok', badge: '기준값에서 조정' },
  { title: '생존 편향 기록', desc: '전학·자퇴 등 제외자의 인원과 사유를 로그로 남깁니다.', status: 'ok', badge: `제외 ${s.value?.students_excluded ?? 0}명` },
])
</script>

<template>
  <div class="section">
    <div class="section-header">
      <div>
        <h2 class="section-title">데이터 현황 · 전처리</h2>
        <div class="section-desc">
          <p>분석 전에 데이터가 어떤 상태인지 확인하는 곳입니다. 이 단계를 건너뛰면 하위권 해석이 전부 틀립니다.</p>
          <p>학교 단위 통계는 항상 <b>[전체]</b>와 <b>[성실 응시자만]</b> 두 버전으로 산출됩니다.</p>
        </div>
      </div>
    </div>

    <div class="section-body">
      <!-- 데이터 보유 현황 -->
      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title">보유 데이터</h3><p class="panel-desc">없는 항목은 그 항목을 쓰는 지표만 "데이터 없음"으로 표시되고 나머지는 계산됩니다.</p></div></div>
        <div class="kpi-row">
          <div class="kpi"><span class="kpi-label">학생 (활성)</span><span class="kpi-value">{{ s?.students_active ?? '—' }}</span><span class="kpi-sub">제외 {{ s?.students_excluded ?? 0 }}명</span></div>
          <div class="kpi" :class="{ 'kpi--warn': s && !dataset.hasMock }"><span class="kpi-label">모의고사</span><span class="kpi-value">{{ s?.mock_rounds.length ?? '—' }}</span><span class="kpi-sub">회차 · {{ s?.mock_rows ?? 0 }}행</span></div>
          <div class="kpi"><span class="kpi-label">내신</span><span class="kpi-value">{{ s?.school_semesters.length ?? '—' }}</span><span class="kpi-sub">학기 · {{ s?.school_rows ?? 0 }}행</span></div>
          <div class="kpi"><span class="kpi-label">입학 시점</span><span class="kpi-value">{{ s?.entry_rows ?? '—' }}</span><span class="kpi-sub">명</span></div>
          <div class="kpi"><span class="kpi-label">문항 통계</span><span class="kpi-value">{{ s?.item_stat_rows ?? '—' }}</span><span class="kpi-sub">문항</span></div>
          <div class="kpi"><span class="kpi-label">문항 응답</span><span class="kpi-value">{{ s?.item_response_rows ?? '—' }}</span><span class="kpi-sub">학생×문항</span></div>
          <div class="kpi"><span class="kpi-label">출결</span><span class="kpi-value">{{ s?.attendance_rows ?? '—' }}</span><span class="kpi-sub">학생×월</span></div>
        </div>

        <div v-if="s?.mock_rounds.length" class="table-wrap">
          <table class="data-table">
            <thead><tr><th>학년도</th><th>학년</th><th>회차</th><th>과목</th><th class="num">응시 학생</th><th class="num">활성 학생 대비</th></tr></thead>
            <tbody>
              <tr v-for="r in s.mock_rounds" :key="`${r.grade_year}-${r.grade}-${r.round}`">
                <td>{{ r.grade_year }}</td><td>{{ r.grade }}학년</td><td>{{ r.round }}월</td>
                <td><span v-for="sub in r.subjects" :key="sub" class="badge badge--muted" style="margin-right:4px">{{ sub }}</span></td>
                <td class="num">{{ r.n_students }}</td>
                <td class="num" :class="{ warn: s.students_active && r.n_students < s.students_active * 0.9 }">{{ s.students_active ? Math.round(r.n_students / s.students_active * 100) : 0 }}%</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div v-if="s?.school_semesters.length" class="table-wrap">
          <table class="data-table">
            <thead><tr><th>학년도</th><th>학년</th><th>학기</th><th>과목</th><th class="num">학생</th></tr></thead>
            <tbody>
              <tr v-for="r in s.school_semesters" :key="`${r.grade_year}-${r.grade}-${r.semester}`">
                <td>{{ r.grade_year }}</td><td>{{ r.grade }}학년</td><td>{{ r.semester }}학기</td>
                <td><span v-for="sub in r.subjects" :key="sub" class="badge badge--muted" style="margin-right:4px">{{ sub }}</span></td>
                <td class="num">{{ r.n_students }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- 전처리 규칙 -->
      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title">전처리 규칙</h3><p class="panel-desc">스펙 2장. 기준값은 설정에서 바꿀 수 있습니다.</p></div></div>
        <div class="rules">
          <div v-for="r in rules" :key="r.title" class="rule">
            <div class="rule-head">
              <span class="rule-title">{{ r.title }}</span>
              <span class="badge" :class="r.status === 'ok' ? 'badge--green' : 'badge--warn'">{{ r.badge }}</span>
            </div>
            <div class="rule-desc">{{ r.desc }}</div>
          </div>
        </div>
      </div>

      <!-- 성실도 필터 -->
      <div class="panel-grid">
        <PanelCard
          title="성실 응시 의심 플래그" tag="전처리 3)"
          :desc="`무응답 ≥ ${t?.sincerity_blank_ratio ?? 20}% · 후반부 ${t?.sincerity_same_choice_run ?? 5}문항 이상 동일 선지 연속 · 전국 정답률 ${t?.easy_item_national_rate ?? 70}%↑ 문항 극단 오답`"
          :status="mockStatus" requires="모의고사(무응답 문항 수) 또는 문항 응답 데이터가 필요합니다."
        />
        <PanelCard
          title="회차별 응시 인원 · 결시" :desc="'[전체] / [성실 응시자만] 두 버전의 N을 회차·과목별로 비교합니다.'"
          :status="mockStatus" requires="모의고사 데이터가 필요합니다."
        />
      </div>

      <!-- 제외자 로그 -->
      <div class="panel">
        <div class="panel-head">
          <div><h3 class="panel-title"><UserMinus :size="16" /> 제외자 로그 (생존 편향 기록)</h3><p class="panel-desc">전학·자퇴 등으로 분석에서 빠진 학생과 사유. 학생 명단에서 제외/복원할 수 있습니다.</p></div>
          <button class="btn-ghost" @click="emit('navigate', 'student')">학생 명단으로</button>
        </div>
        <div v-if="!students.exclusionLog.length" class="placeholder" style="min-height:80px">제외된 학생이 없습니다.</div>
        <div v-else class="table-wrap">
          <table class="data-table">
            <thead><tr><th>일시</th><th>익명 ID</th><th>동작</th><th>사유</th></tr></thead>
            <tbody>
              <tr v-for="l in students.exclusionLog" :key="l.id">
                <td class="muted">{{ l.logged_at }}</td>
                <td class="mono">{{ l.anon_id }}</td>
                <td><span class="badge" :class="l.action === 'exclude' ? 'badge--red' : 'badge--green'"><component :is="l.action === 'exclude' ? UserMinus : RotateCcw" :size="12" /> {{ l.action === 'exclude' ? '제외' : '복원' }}</span></td>
                <td>{{ l.reason || '—' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="note note--warn">
        <AlertTriangle :size="15" />
        <div>고3 데이터는 재수생 유입으로 백분위가 구조적으로 밀립니다. 종단 분석은 회차 내 상대 이동으로 계산합니다.</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.rules { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
.rule { padding: 12px 14px; border-radius: 12px; background: var(--bg-2); border: 1px solid var(--bd-1); display: flex; flex-direction: column; gap: 6px; }
.rule-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.rule-title { font-size: 13px; font-weight: 700; color: var(--tx-1); }
.rule-desc { font-size: 13px; color: var(--tx-3); line-height: 1.55; }
.warn { color: var(--clr-warn-text); font-weight: 600; }
</style>
