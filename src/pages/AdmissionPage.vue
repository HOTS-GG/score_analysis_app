<script setup>
import { computed, onMounted, ref } from 'vue'
import { AlertTriangle, Pencil } from 'lucide-vue-next'
import { useContextStore } from '../stores/context'
import { useAnalysisStore } from '../stores/analysis'

const emit = defineEmits(['navigate'])
const ctx = useContextStore()
const analysis = useAnalysisStore()
const tab = ref('school')
const typeFilter = ref('')
const trackFilter = ref('')
const regTrack = ref('자연')
const regCat = ref('교과')
const q = ref('')
const editing = ref(null)
const editCut = ref('')
const editNote = ref('')

onMounted(() => Promise.all([analysis.fetchAdmission(3), analysis.fetchRegional(), analysis.fetchReference()]))

const g9 = computed(() => analysis.bundle?.univ?.grade9_cont ?? null)
const types = computed(() => [...new Set(analysis.admissionAgg.map(a => a.adm_type))].filter(Boolean))
const tracks = computed(() => [...new Set(analysis.admissionAgg.map(a => a.track))].filter(Boolean))
const agg = computed(() => analysis.admissionAgg.filter(a => (!typeFilter.value || a.adm_type === typeFilter.value) && (!trackFilter.value || a.track === trackFilter.value) && (!q.value || a.univ.includes(q.value))))
const regional = computed(() => analysis.regional.filter(r => r.track === regTrack.value && r.category === regCat.value && (!q.value || r.univ.includes(q.value))))
const refRows = computed(() => analysis.reference.filter(r => !q.value || r.univ.includes(q.value)))
const fit = (min, max) => g9.value == null || min == null ? '' : g9.value <= min ? 'in' : g9.value <= max ? 'edge' : 'out'
function startEdit(r) { editing.value = r.id; editCut.value = r.cut ?? ''; editNote.value = r.note ?? '' }
async function saveEdit() { await analysis.saveReference(editing.value, editCut.value === '' ? null : Number(editCut.value), editNote.value); editing.value = null }
</script>

<template>
  <div class="section">
    <div class="section-header compact">
      <div>
        <h2 class="section-title">진학 참고선</h2>
        <div class="section-desc"><p>춘고 3개년 수시 결과, 강원 지역 대학별 지원·합격 범위, 참고선. 모두 <b>9등급제 시절 값</b>이라 현재 5등급제 학생과는 9등급 환산(연속값)으로만 대략 맞댑니다. 확률·합격 예측은 하지 않습니다.</p></div>
      </div>
      <div class="header-actions">
        <span v-if="analysis.bundle" class="badge badge--accent">선택 학생 {{ analysis.bundle.student.name }} · 9등급 환산 {{ g9 ?? '—' }}</span>
        <button v-else class="btn-ghost" @click="emit('navigate', 'student')">학생을 고르면 위치가 표시됩니다</button>
        <input v-model="q" class="ui-input ui-input--sm" placeholder="대학명 검색" style="width:160px" />
      </div>
    </div>
    <div class="section-body">
      <div class="seg">
        <button :class="{ on: tab === 'school' }" @click="tab = 'school'">춘고 3개년 수시 결과</button>
        <button :class="{ on: tab === 'regional' }" @click="tab = 'regional'">강원 지역 대학별 범위</button>
        <button :class="{ on: tab === 'ref' }" @click="tab = 'ref'">참고선 (편집)</button>
      </div>
      <p v-if="analysis.error" class="msg-error">{{ analysis.error }}</p>

      <div v-if="tab === 'school'" class="panel">
        <div class="panel-head">
          <div><h3 class="panel-title">대학·전형유형·계열별 지원과 합격 <span class="badge badge--muted">지원 3건 이상</span></h3><p class="panel-desc">합격 = 최초합격 + 충원합격. 등급은 전교과 평균(9등급제). 선택 학생의 9등급 환산이 합격자 범위 안이면 초록, 최저 근처면 노랑.</p></div>
          <div class="row">
            <select v-model="typeFilter" class="ui-select"><option value="">전형유형 전체</option><option v-for="t in types" :key="t" :value="t">{{ t }}</option></select>
            <select v-model="trackFilter" class="ui-select"><option value="">계열 전체</option><option v-for="t in tracks" :key="t" :value="t">{{ t }}</option></select>
          </div>
        </div>
        <div v-if="!analysis.admissionAgg.length" class="placeholder">수시 진학결과 데이터가 없습니다. 데이터 → 가져오기(ETL)로 2023~2025 진학결과를 넣으세요.</div>
        <div v-else class="table-wrap" style="max-height:560px"><table class="data-table"><thead><tr><th>대학</th><th>전형유형</th><th>계열</th><th>연도</th><th class="n">지원</th><th class="n">합격</th><th class="n">합격률</th><th class="n">지원 등급 범위</th><th class="n">합격자 등급 (최고~중앙~최저)</th><th>위치</th></tr></thead><tbody>
          <tr v-for="a in agg" :key="a.univ + a.adm_type + a.track" :class="fit(a.pass_min, a.pass_max)">
            <td class="nm">{{ a.univ }}</td><td>{{ a.adm_type }}</td><td>{{ a.track }}</td><td class="muted">{{ a.years.join('·') }}</td>
            <td class="n">{{ a.apply_n }}</td><td class="n">{{ a.pass_n }}</td><td class="n">{{ a.pass_rate }}%</td>
            <td class="n">{{ a.apply_min ?? '—' }} ~ {{ a.apply_max ?? '—' }}</td>
            <td class="n">{{ a.pass_n ? `${a.pass_min} ~ ${a.pass_median} ~ ${a.pass_max}` : '—' }}</td>
            <td><span v-if="fit(a.pass_min, a.pass_max) === 'in'" class="badge badge--green">합격자 범위 안</span><span v-else-if="fit(a.pass_min, a.pass_max) === 'edge'" class="badge badge--warn">최저 근처</span><span v-else-if="fit(a.pass_min, a.pass_max) === 'out'" class="badge badge--muted">범위 밖</span></td>
          </tr>
        </tbody></table></div>
      </div>

      <div v-else-if="tab === 'regional'" class="panel">
        <div class="panel-head">
          <div><h3 class="panel-title">강원 지역 수험생의 대학별·전형별 지원·합격 범위</h3><p class="panel-desc">교육청 공유 자료(2023~2025 대입). 등급은 전교과. '대학발표 70%컷'은 있는 경우만.</p></div>
          <div class="row">
            <div class="seg"><button :class="{ on: regTrack === '인문' }" @click="regTrack = '인문'">인문</button><button :class="{ on: regTrack === '자연' }" @click="regTrack = '자연'">자연</button></div>
            <div class="seg"><button :class="{ on: regCat === '교과' }" @click="regCat = '교과'">교과</button><button :class="{ on: regCat === '종합' }" @click="regCat = '종합'">종합</button></div>
          </div>
        </div>
        <div v-if="!analysis.regional.length" class="placeholder">지역 자료가 없습니다. ETL 로 '4) 분석자료' 파일을 넣으세요.</div>
        <div v-else class="table-wrap" style="max-height:560px"><table class="data-table"><thead><tr><th>대학</th><th>전형명</th><th class="n">지원</th><th class="n">지원 최고~최저</th><th class="n">합격</th><th class="n">합격 최고~최저</th><th class="n">합격률</th><th class="n">70%컷</th><th>위치</th></tr></thead><tbody>
          <tr v-for="r in regional" :key="r.univ + r.adm_name" :class="fit(r.pass_best, r.pass_worst)">
            <td class="nm">{{ r.univ }}</td><td>{{ r.adm_name }}</td><td class="n">{{ r.apply_n }}</td><td class="n">{{ r.apply_best ?? '—' }} ~ {{ r.apply_worst ?? '—' }}</td>
            <td class="n">{{ r.pass_n }}</td><td class="n">{{ r.pass_n ? `${r.pass_best} ~ ${r.pass_worst}` : '—' }}</td><td class="n">{{ r.pass_rate == null ? '—' : Number(r.pass_rate).toFixed(1) + '%' }}</td><td class="n">{{ r.cut70 ?? '' }}</td>
            <td><span v-if="fit(r.pass_best, r.pass_worst) === 'in'" class="badge badge--green">범위 안</span><span v-else-if="fit(r.pass_best, r.pass_worst) === 'edge'" class="badge badge--warn">최저 근처</span></td>
          </tr>
        </tbody></table></div>
      </div>

      <div v-else class="panel">
        <div class="panel-head"><div><h3 class="panel-title">참고선</h3><p class="panel-desc">기준값은 예시입니다. 학교 입결 자료로 바꿔 쓰세요 (연필로 편집).</p></div></div>
        <div class="table-wrap" style="max-height:560px"><table class="data-table"><thead><tr><th>권역</th><th>대학</th><th>모집단위</th><th>전형</th><th class="n">기준 등급</th><th>메모</th><th></th></tr></thead><tbody>
          <tr v-for="r in refRows" :key="r.id" :class="g9 != null && r.cut != null ? (g9 <= r.cut ? 'in' : g9 <= r.cut + 0.3 ? 'edge' : '') : ''">
            <td class="muted">{{ r.region }}</td><td class="nm">{{ r.univ }}</td><td>{{ r.unit }}</td><td>{{ r.adm_type }}</td>
            <td class="n"><input v-if="editing === r.id" v-model="editCut" type="number" step="0.1" class="ui-input ui-input--sm" style="width:80px" /><template v-else>{{ r.cut ?? '—' }}</template></td>
            <td><input v-if="editing === r.id" v-model="editNote" class="ui-input ui-input--sm" /><span v-else class="muted">{{ r.note }}</span></td>
            <td class="n"><template v-if="editing === r.id"><button class="btn-soft" @click="saveEdit">저장</button> <button class="btn-ghost" @click="editing = null">취소</button></template><button v-else class="btn-icon" @click="startEdit(r)"><Pencil :size="14" /></button></td>
          </tr>
        </tbody></table></div>
      </div>

      <div class="note note--warn"><AlertTriangle :size="14" /><div>2026학년도 1·2학년은 5등급제 첫 적용 학년입니다. 9등급제 시절 합격선과 등급 숫자를 맞대어 "몇 등급이면 어디 합격"이라고 단정하지 마세요. 이 화면은 범위를 보여줄 뿐 합격 가능성을 계산하지 않습니다.</div></div>
    </div>
  </div>
</template>

<style scoped>
.compact { padding: 18px 28px 12px; align-items: center; }
.section-body { padding: 16px 28px 40px; }
.n { text-align: right; white-space: nowrap; }
.nm { font-weight: 600; color: var(--tx-1); white-space: nowrap; }
.muted { color: var(--tx-4); }
tr.in td { background: var(--clr-green-bg); }
tr.edge td { background: var(--clr-warn-bg); }
</style>
