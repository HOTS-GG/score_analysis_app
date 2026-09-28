<script setup>
import { computed, onMounted, ref } from 'vue'
import { Download, Pencil, Plus, Search, Trash2, UserMinus, UserPlus, Users } from 'lucide-vue-next'
import { useStudentStore } from '../stores/student'
import StudentModal from '../components/StudentModal.vue'
import ConfirmModal from '../components/ConfirmModal.vue'

const emit = defineEmits(['navigate'])
const store = useStudentStore()

const modal = ref(null) // { mode, student }
const confirm = ref(null) // { type, student }
const query = ref('')
const showExcluded = ref(true)
const busy = ref(false)
const msg = ref('')

onMounted(() => store.fetchStudents())

const filteredGroups = computed(() => {
  const q = query.value.trim()
  return store.grouped
    .map(g => ({
      ...g,
      students: g.students.filter(s =>
        (showExcluded.value || s.status === 'active') &&
        (!q || s.name.includes(q) || s.anon_id.includes(q) || String(s.number) === q)),
    }))
    .filter(g => g.students.length)
})

async function handleSave(s) {
  if (busy.value) return
  busy.value = true
  msg.value = ''
  try {
    if (modal.value.mode === 'add') await store.createStudent(s)
    else await store.updateStudent(modal.value.student.id, s)
    modal.value = null
  } catch (e) {
    msg.value = String(e)
  } finally {
    busy.value = false
  }
}

async function handleConfirm(inputValue) {
  const { type, student } = confirm.value
  msg.value = ''
  try {
    if (type === 'delete') await store.deleteStudent(student.id)
    else if (type === 'exclude') await store.setStatus(student.id, 'excluded', inputValue)
    else if (type === 'restore') await store.setStatus(student.id, 'active', null)
  } catch (e) {
    msg.value = String(e)
  }
  confirm.value = null
}
</script>

<template>
  <div class="section">
    <div class="section-header">
      <div>
        <h2 class="section-title">학생 명단</h2>
        <div class="section-desc">
          <p>실명은 이 명단에만 저장됩니다. 분석 테이블과 출력물은 익명 ID로 참조하고 최종 출력에서만 이름을 붙입니다.</p>
          <p>전학·자퇴 학생은 삭제하지 말고 <b>제외</b>하세요. 사유가 로그로 남아 생존 편향을 기록합니다.</p>
        </div>
      </div>
      <div class="header-actions">
        <button class="btn-soft" @click="emit('navigate', 'import')"><Download :size="16" /> 엑셀로 가져오기</button>
        <button class="btn-add" @click="modal = { mode: 'add', student: null }"><Plus :size="18" /> 학생 추가</button>
      </div>
    </div>

    <div class="section-body">
      <div class="row">
        <div class="search">
          <Search :size="15" />
          <input v-model="query" class="ui-input ui-input--sm" placeholder="이름 · 익명ID · 번호 검색" />
        </div>
        <label class="checkbox-row"><input v-model="showExcluded" type="checkbox" /> 제외 학생 표시</label>
        <span class="grow" />
        <span class="badge badge--muted">활성 {{ store.activeStudents.length }}명</span>
        <span class="badge badge--muted">제외 {{ store.students.length - store.activeStudents.length }}명</span>
      </div>

      <p v-if="msg" class="msg-error">{{ msg }}</p>

      <div v-if="store.loading" class="state-box"><p class="state-text">불러오는 중...</p></div>
      <div v-else-if="store.error" class="state-box state-box--error"><p>{{ store.error }}</p></div>
      <div v-else-if="store.students.length === 0" class="empty-state">
        <Users :size="40" color="var(--tx-3)" />
        <p class="empty-title">등록된 학생이 없습니다</p>
        <p class="empty-desc">학생을 직접 추가하거나, 데이터 가져오기에서 명단 파일을 불러오세요.</p>
        <div class="row">
          <button class="btn-soft" @click="emit('navigate', 'import')"><Download :size="16" /> 명단 파일 가져오기</button>
          <button class="btn-add" @click="modal = { mode: 'add', student: null }"><Plus :size="18" /> 첫 학생 추가</button>
        </div>
      </div>

      <div v-else class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>익명 ID</th><th>학년</th><th>반</th><th>번호</th><th>이름</th><th>상태</th><th></th>
            </tr>
          </thead>
          <tbody>
            <template v-for="g in filteredGroups" :key="`${g.cohort}-${g.grade}-${g.classNo}`">
              <tr class="group-row">
                <td colspan="7">
                  {{ g.cohort }}학년도 입학 · {{ g.grade }}학년 {{ g.classNo }}반
                  <span class="group-count">{{ g.students.length }}명</span>
                </td>
              </tr>
              <tr v-for="s in g.students" :key="s.id" :class="{ excluded: s.status === 'excluded' }">
                <td class="mono">{{ s.anon_id }}</td>
                <td>{{ s.grade }}</td>
                <td>{{ s.class_no }}</td>
                <td>{{ s.number }}</td>
                <td>{{ s.name || '—' }}</td>
                <td>
                  <span v-if="s.status === 'active'" class="badge badge--green">활성</span>
                  <span v-else class="badge badge--muted" :title="s.exclude_reason">제외 · {{ s.exclude_reason }}</span>
                </td>
                <td class="actions">
                  <button class="btn-icon" title="수정" @click="modal = { mode: 'edit', student: s }"><Pencil :size="14" /></button>
                  <button v-if="s.status === 'active'" class="btn-icon" title="제외(전학·자퇴 등)" @click="confirm = { type: 'exclude', student: s }"><UserMinus :size="14" /></button>
                  <button v-else class="btn-icon" title="복원" @click="confirm = { type: 'restore', student: s }"><UserPlus :size="14" /></button>
                  <button class="btn-icon" title="삭제(성적 데이터도 함께 삭제)" @click="confirm = { type: 'delete', student: s }"><Trash2 :size="14" /></button>
                </td>
              </tr>
            </template>
          </tbody>
        </table>
      </div>
    </div>

    <StudentModal v-if="modal" :mode="modal.mode" :student="modal.student" @save="handleSave" @close="modal = null" />

    <ConfirmModal
      v-if="confirm?.type === 'exclude'" title="학생 제외" danger confirm-label="제외"
      :message="`${confirm.student.name || confirm.student.anon_id} 학생을 분석 대상에서 제외합니다.\n데이터는 삭제되지 않으며 사유가 로그에 남습니다.`"
      input-label="제외 사유" input-placeholder="예: 전학, 자퇴, 장기결석"
      @confirm="handleConfirm" @cancel="confirm = null"
    />
    <ConfirmModal
      v-if="confirm?.type === 'restore'" title="학생 복원" confirm-label="복원"
      :message="`${confirm.student.name || confirm.student.anon_id} 학생을 다시 분석 대상에 포함합니다.`"
      @confirm="handleConfirm" @cancel="confirm = null"
    />
    <ConfirmModal
      v-if="confirm?.type === 'delete'" title="학생 삭제" danger confirm-label="삭제"
      :message="`${confirm.student.name || confirm.student.anon_id} 학생과 연결된 모든 성적 데이터가 함께 삭제됩니다.\n전학·자퇴라면 삭제 대신 '제외'를 사용하세요.`"
      @confirm="handleConfirm" @cancel="confirm = null"
    />
  </div>
</template>

<style scoped>
.search { display: flex; align-items: center; gap: 8px; padding: 0 10px; border: 1px solid var(--bd-1); border-radius: 10px; background: var(--bg-1); color: var(--tx-4); }
.search input { border: none; background: none; width: 240px; }
.search input:focus { background: none; }
.actions { text-align: right; white-space: nowrap; }
tr.excluded td { color: var(--tx-4); }
.state-box p, .state-text { color: var(--tx-3); margin: 0; }
</style>
