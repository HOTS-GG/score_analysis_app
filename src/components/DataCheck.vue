<script setup>
import { computed, onMounted } from 'vue'
import { AlertTriangle, CheckCircle2, CircleDashed, RotateCcw, UserMinus } from 'lucide-vue-next'
import { useDatasetStore } from '../stores/dataset'
import { useStudentStore } from '../stores/student'

const emit = defineEmits(['navigate', 'tab'])
const dataset = useDatasetStore()
const students = useStudentStore()
onMounted(() => Promise.all([dataset.refreshSummary(), students.fetchExclusionLog()]))
const s = computed(() => dataset.summary)

/** 있어야 할 자료 목록: 무엇이고, 없으면 어느 화면이 비는지, 어디서 넣는지 */
const checks = computed(() => [
  { key: 'students', title: '학생 명단', ok: (s.value?.students_active ?? 0) > 0, count: s.value ? `${s.value.students_active}명 (제외 ${s.value.students_excluded}명)` : '',
    what: '학년·반·번호·이름. 모든 성적을 학생에게 붙이는 기준입니다.', ifMissing: '명단이 없으면 성적을 넣어도 학생을 찾지 못해 건너뜁니다. 나이스·모의고사 파일을 넣을 때 "명단에 없는 학생 자동 생성"을 켜 두면 자동으로 만들어집니다.', go: 'student', goLabel: '학생 명단' },
  { key: 'school', title: '내신 (나이스 교과학습발달상황)', ok: (s.value?.school_rows ?? 0) > 0, count: s.value ? `${s.value.school_semesters.length}개 학기 · ${s.value.school_rows}행` : '',
    what: '학기별 과목 성취도·석차등급·원점수·수강자 수.', ifMissing: '없으면 학급 한눈에·학생 리포트·과목 화면이 비어 있습니다.', go: 'neis', goLabel: '나이스 내신 가져오기' },
  { key: 'mock', title: '모의고사 (학력평가)', ok: (s.value?.mock_rows ?? 0) > 0, count: s.value ? `${s.value.mock_rounds.length}개 회차 · ${s.value.mock_rows}행` : '',
    what: '회차별 전국 백분위·등급. 전국 기준 위치와 성장은 이것으로만 잽니다.', ifMissing: '없으면 "내신 vs 모의고사"와 학년·학교 화면이 비어 있습니다. 회차가 2개 이상이어야 변화 추이가 나옵니다.', go: 'mock', goLabel: '모의고사 성적 가져오기' },
  { key: 'entry', title: '입학 시점 (고입내신·반배치고사)', ok: (s.value?.entry_rows ?? 0) > 0, count: s.value ? `${s.value.entry_rows}명` : '',
    what: '입학 당시 위치. "입학 때보다 올라왔나/내려갔나"를 볼 때 씁니다.', ifMissing: '없어도 다른 화면은 다 나옵니다. 입학 대비 칸만 "—"로 표시됩니다.', go: 'mock', goLabel: '직접 매핑으로 넣기' },
  { key: 'items', title: '문항 자료 (선택)', ok: (s.value?.item_stat_rows ?? 0) > 0, count: s.value ? `${s.value.item_stat_rows}문항 · 응답 ${s.value.item_response_rows}행` : '',
    what: '문항별 정답률과 학생별 정오답. 실수형/실력한계형 구분에 필요합니다.', ifMissing: '없어도 됩니다. 문항 분석 탭만 비어 있습니다.', go: 'mock', goLabel: '직접 매핑으로 넣기' },
])
const missingRounds = computed(() => (s.value?.mock_rounds ?? []).map(r => ({ ...r, missing: Math.max(0, (s.value?.students_active ?? 0) - r.n_students) })))
</script>

<template>
  <div class="section">
    <div class="section-header compact">
      <div>
        <h2 class="section-title">데이터 점검</h2>
        <div class="section-desc"><p>지금 파일에 <b>무엇이 들어와 있고 무엇이 빠졌는지</b> 확인하는 곳입니다. 빠진 것이 있으면 어느 화면이 비는지, 어디서 넣으면 되는지 함께 적었습니다.</p></div>
      </div>
    </div>
    <div class="section-body">
      <div class="checks">
        <div v-for="c in checks" :key="c.key" class="check" :class="{ ok: c.ok }">
          <div class="ck-head">
            <component :is="c.ok ? CheckCircle2 : CircleDashed" :size="18" class="ck-ic" />
            <div class="ck-title"><b>{{ c.title }}</b><span class="ck-count">{{ c.ok ? c.count : '아직 없음' }}</span></div>
            <button class="btn-ghost" @click="emit('tab', c.go)">{{ c.goLabel }}</button>
          </div>
          <p class="ck-what">{{ c.what }}</p>
          <p v-if="!c.ok" class="ck-miss">{{ c.ifMissing }}</p>
        </div>
      </div>

      <div v-if="missingRounds.length" class="panel">
        <div class="panel-head"><div><h3 class="panel-title">모의고사 회차별 응시 인원</h3><p class="panel-desc">명단 인원보다 적으면 그만큼 미응시이거나 파일에서 빠진 학생입니다. 미응시는 0점이 아니라 "없음"으로 처리되어 평균을 끌어내리지 않습니다.</p></div></div>
        <div class="table-wrap"><table class="data-table"><thead><tr><th>학년도</th><th>학년</th><th>회차</th><th>과목</th><th class="n">응시</th><th class="n">명단 대비 빠짐</th></tr></thead><tbody>
          <tr v-for="r in missingRounds" :key="`${r.grade_year}-${r.grade}-${r.round}`">
            <td>{{ r.grade_year }}</td><td>{{ r.grade }}학년</td><td>{{ r.round }}월</td>
            <td><span v-for="sub in r.subjects" :key="sub" class="badge badge--muted" style="margin-right:4px">{{ sub }}</span></td>
            <td class="n">{{ r.n_students }}명</td><td class="n" :class="{ warn: r.missing > 0 }">{{ r.missing ? r.missing + '명' : '없음' }}</td>
          </tr>
        </tbody></table></div>
      </div>

      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title"><UserMinus :size="16" /> 분석에서 뺀 학생</h3><p class="panel-desc">전학·자퇴 학생은 삭제하지 말고 "제외"해 두면 통계에서 빠지고 기록은 남습니다. 학생 명단에서 제외·복원할 수 있습니다.</p></div></div>
        <div v-if="!students.exclusionLog.length" class="placeholder" style="min-height:70px">제외한 학생이 없습니다.</div>
        <div v-else class="table-wrap"><table class="data-table"><thead><tr><th>일시</th><th>익명 ID</th><th>동작</th><th>사유</th></tr></thead><tbody>
          <tr v-for="l in students.exclusionLog" :key="l.id"><td class="muted">{{ l.logged_at }}</td><td class="mono">{{ l.anon_id }}</td>
            <td><span class="badge" :class="l.action === 'exclude' ? 'badge--red' : 'badge--green'"><component :is="l.action === 'exclude' ? UserMinus : RotateCcw" :size="12" /> {{ l.action === 'exclude' ? '제외' : '복원' }}</span></td><td>{{ l.reason || '—' }}</td></tr>
        </tbody></table></div>
      </div>

      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title">프로그램이 자동으로 하는 처리</h3><p class="panel-desc">따로 손댈 것은 없고, 이렇게 다룬다는 것만 알아 두세요.</p></div></div>
        <ul class="rules">
          <li><b>미응시·결시는 0점이 아닙니다.</b> 빈칸으로 두어 평균과 순위에서 뺍니다.</li>
          <li><b>같은 학생·회차·과목을 다시 넣으면 덮어씁니다.</b> 가채점 뒤 정식 성적표를 같은 회차로 넣으면 백분위·등급이 채워집니다.</li>
          <li><b>등급끼리는 계산하지 않습니다.</b> 화면의 등급평균은 보기용이고, 위치·변화는 석차백분율로 계산합니다.</li>
          <li><b>이름은 명단에만 저장됩니다.</b> 분석 표와 내보내는 해석용 JSON에는 익명 ID만 들어갑니다.</li>
          <li><b>백분위 0과 100</b>은 통계 변환 오류를 막기 위해 0.5와 99.5로 다룹니다 (설정에서 조정).</li>
        </ul>
      </div>
      <div class="note note--warn"><AlertTriangle :size="14" /><div>고3 자료는 재수생 유입으로 전국 백분위가 구조적으로 밀립니다. 학년 간 비교는 같은 회차 안에서만 합니다.</div></div>
    </div>
  </div>
</template>

<style scoped>
.compact { padding: 18px 28px 12px; }
.section-body { padding: 16px 28px 40px; }
.checks { display: flex; flex-direction: column; gap: 8px; }
.check { border: 1px solid var(--bd-1); border-radius: 12px; padding: 12px 16px; background: var(--bg-1); }
.check.ok { border-color: var(--clr-green-border); }
.ck-head { display: flex; align-items: center; gap: 10px; }
.ck-ic { color: var(--tx-4); flex-shrink: 0; }
.check.ok .ck-ic { color: var(--clr-green-text); }
.ck-title { flex: 1; display: flex; align-items: baseline; gap: 10px; }
.ck-title b { font-size: 14px; color: var(--tx-1); }
.ck-count { font-size: 12.5px; color: var(--tx-3); }
.ck-what { margin: 6px 0 0 28px; font-size: 12.5px; color: var(--tx-3); }
.ck-miss { margin: 4px 0 0 28px; font-size: 12.5px; color: var(--clr-warn-text); line-height: 1.5; }
.n { text-align: right; }
.warn { color: var(--clr-warn-text); font-weight: 600; }
.muted { color: var(--tx-4); }
.rules { margin: 0; padding-left: 18px; display: flex; flex-direction: column; gap: 6px; font-size: 13px; color: var(--tx-2); line-height: 1.55; }
</style>
