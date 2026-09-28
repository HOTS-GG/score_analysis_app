<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useContextStore } from '../stores/context'
import { useAnalysisStore } from '../stores/analysis'
import GradeBadge from '../components/GradeBadge.vue'
import HBars from '../components/charts/HBars.vue'

const emit = defineEmits(['navigate'])
const ctx = useContextStore()
const analysis = useAnalysisStore()
const term = ref('')
const subject = ref('')
const onlyClass = ref(false)

const terms = computed(() => [...new Set(analysis.subjectOptions.map(o => o.term))].sort())
const subjects = computed(() => analysis.subjectOptions.filter(o => o.term === term.value))

async function loadOptions() {
  if (!ctx.grade) return
  await analysis.fetchSubjectOptions(ctx.gradeYear, ctx.grade)
  if (!terms.value.includes(term.value)) term.value = terms.value.at(-1) ?? ''
  if (!subjects.value.some(s => s.subject === subject.value)) subject.value = subjects.value[0]?.subject ?? ''
}
async function loadReport() {
  if (!term.value || !subject.value) { analysis.subjectReport = null; return }
  await analysis.fetchSubjectReport(ctx.gradeYear, ctx.grade, term.value, subject.value, onlyClass.value ? ctx.classNo : null)
}
onMounted(async () => { await loadOptions(); await loadReport() })
watch(() => [ctx.gradeYear, ctx.grade], async () => { await loadOptions(); await loadReport() })
watch(term, () => { if (!subjects.value.some(s => s.subject === subject.value)) subject.value = subjects.value[0]?.subject ?? '' })
watch([subject, onlyClass, () => ctx.classNo], loadReport)

const r = computed(() => analysis.subjectReport)
const achBars = computed(() => !r.value ? [] : 'ABCDE'.split('').map((L, i) => ({ label: `${L} (학년 ${r.value.ach_counts[i]}명)`, value: r.value.dist_official[i], max: 100, text: r.value.dist_official[i] == null ? '—' : r.value.dist_official[i] + '%' })))
const gradeBars = computed(() => !r.value ? [] : r.value.grade_counts.map((n, i) => ({ label: `${i + 1}등급`, value: n, max: Math.max(1, ...r.value.grade_counts), text: n + '명' })))
</script>

<template>
  <div class="section">
    <div class="section-header compact">
      <div>
        <h2 class="section-title">과목별 성취 <span class="ctxl">{{ ctx.contextLabel.replace(/ \d+반$/, '') }}</span></h2>
        <div class="section-desc"><p>교과 담당 관점. 나이스 성취도 분포(수강자 전체)와 이 학년 학생들의 등급 분포, 반별 비교, 학생 목록.</p></div>
      </div>
      <div class="header-actions">
        <select v-model="term" class="ui-select"><option v-for="t in terms" :key="t" :value="t">{{ t.replace('-', '학년 ') }}학기</option></select>
        <select v-model="subject" class="ui-select" style="min-width:200px"><option v-for="s in subjects" :key="s.subject" :value="s.subject">{{ s.subject }} ({{ s.n }}명{{ s.ach_scale === 3 ? ' · 3단계' : '' }})</option></select>
        <label class="checkbox-row"><input v-model="onlyClass" type="checkbox" /> {{ ctx.classNo }}반만</label>
      </div>
    </div>
    <div class="section-body">
      <p v-if="analysis.error" class="msg-error">{{ analysis.error }}</p>
      <div v-if="!analysis.subjectOptions.length" class="empty-state">
        <p class="empty-title">내신 데이터가 없습니다</p>
        <p class="empty-desc">데이터 → 가져오기에서 나이스 교과학습발달상황 파일을 넣으세요.</p>
        <button class="btn-add" @click="emit('navigate', 'data')">데이터 가져오기</button>
      </div>
      <template v-else-if="r">
        <div class="kpi-row">
          <div class="kpi"><span class="kpi-label">과목</span><span class="kpi-value" style="font-size:18px">{{ r.subject }}</span><span class="kpi-sub">{{ r.area }} · {{ r.kind }} · {{ r.credits }}학점</span></div>
          <div class="kpi"><span class="kpi-label">이 학년 학생</span><span class="kpi-value">{{ r.n }}<small>명</small></span><span class="kpi-sub">수강자 {{ r.enrolled_n ?? '—' }}명 (나이스)</span></div>
          <div class="kpi"><span class="kpi-label">과목 평균 (나이스)</span><span class="kpi-value">{{ r.subject_mean ?? '—' }}</span><span class="kpi-sub">학생 원점수 평균 {{ r.raw_mean ?? '—' }} · SD {{ r.raw_sd ?? '—' }}</span></div>
          <div class="kpi"><span class="kpi-label">A 비율 (수강자)</span><span class="kpi-value">{{ r.dist_official[0] == null ? '—' : r.dist_official[0] + '%' }}</span><span class="kpi-sub">대학은 학교 A 비율이 높으면 A 가치를 낮게 볼 수 있음</span></div>
        </div>
        <div class="panel-grid three">
          <div class="panel"><div class="panel-head"><div><h3 class="panel-title">성취도 분포</h3><p class="panel-desc">나이스 분포비율(수강자 전체) · 괄호는 이 학년 학생 수</p></div></div><HBars :rows="achBars" /></div>
          <div class="panel"><div class="panel-head"><div><h3 class="panel-title">석차등급 분포</h3><p class="panel-desc">이 학년 학생 기준</p></div></div><HBars v-if="r.ach_scale === 5" :rows="gradeBars" /><div v-else class="placeholder" style="min-height:80px">3단계 성취도 과목은 등급이 없습니다.</div></div>
          <div class="panel"><div class="panel-head"><div><h3 class="panel-title">반별 비교</h3><p class="panel-desc">원점수 평균 · A 비율 · 등급평균(표시용)</p></div></div>
            <div class="table-wrap"><table class="data-table mini"><thead><tr><th>반</th><th class="n">인원</th><th class="n">원점수 평균</th><th class="n">A 비율</th><th class="n">등급평균</th></tr></thead><tbody>
              <tr v-for="c in r.by_class" :key="c.class_no" :class="{ me: c.class_no === ctx.classNo }"><td>{{ c.class_no }}반</td><td class="n">{{ c.n }}</td><td class="n">{{ c.raw_mean ?? '—' }}</td><td class="n">{{ c.a_rate ?? '—' }}%</td><td class="n">{{ c.gpa ?? '—' }}</td></tr>
            </tbody></table></div>
          </div>
        </div>
        <div class="panel">
          <div class="panel-head"><div><h3 class="panel-title">학생 목록 <span class="badge badge--muted">{{ r.students.length }}명</span></h3><p class="panel-desc">원점수 내림차순. 행을 누르면 상담 리포트로.</p></div></div>
          <div class="table-wrap" style="max-height:480px"><table class="data-table"><thead><tr><th>반</th><th>번호</th><th>이름</th><th class="n">원점수</th><th class="n">편차</th><th>성취도</th><th>등급</th></tr></thead><tbody>
            <tr v-for="s in r.students" :key="s.student_id" class="pick" @click="ctx.classNo = s.class_no; ctx.openStudent(s.student_id)">
              <td>{{ s.class_no }}</td><td>{{ s.number }}</td><td class="nm">{{ s.name }}</td><td class="n">{{ s.raw ?? '—' }}</td>
              <td class="n" :class="{ good: s.deviation >= 8, bad: s.deviation <= -8 }">{{ s.deviation == null ? '—' : (s.deviation > 0 ? '+' : '') + s.deviation }}</td>
              <td><GradeBadge :value="s.achievement" /></td><td><GradeBadge :value="s.grade5" :digits="0" /></td>
            </tr>
          </tbody></table></div>
        </div>
      </template>
      <div v-else-if="analysis.loading" class="placeholder">계산 중…</div>
    </div>
  </div>
</template>

<style scoped>
.compact { padding: 18px 28px 12px; align-items: center; }
.ctxl { font-size: 13px; font-weight: 500; color: var(--tx-4); margin-left: 8px; }
.section-body { padding: 16px 28px 40px; }
.three { grid-template-columns: repeat(3, minmax(0, 1fr)); }
@media (max-width: 1300px) { .three { grid-template-columns: 1fr; } }
.n { text-align: right; white-space: nowrap; }
.nm { font-weight: 600; color: var(--tx-1); }
.mini td { padding: 5px 10px; font-size: 12.5px; }
.me td { background: rgba(var(--accent-rgb), 0.1); font-weight: 700; }
.pick { cursor: pointer; }
.good { color: var(--clr-green-text); font-weight: 600; } .bad { color: var(--clr-red-text); font-weight: 600; }
</style>
