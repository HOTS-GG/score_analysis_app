import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import { invoke } from '../lib/ipc'

/**
 * 앱 전체가 공유하는 맥락: 학년도·학년·반·학기·산출 범위 + 선택 학생.
 * 상단 바가 바꾸고 모든 화면이 읽는다.
 */
export const useContextStore = defineStore('context', () => {
  const contexts = ref([]) // [{grade_year, grade, classes, terms, students}]
  const gradeYear = ref(null)
  const grade = ref(null)
  const classNo = ref(null)
  const term = ref('ALL')
  const scope = ref('main') // main | all
  const studentId = ref(null)
  const page = ref('class')
  const loaded = ref(false)

  const current = computed(() => contexts.value.find(c => c.grade_year === gradeYear.value && c.grade === grade.value) ?? null)
  const classes = computed(() => current.value?.classes ?? [])
  const terms = computed(() => current.value?.terms ?? [])
  const termOptions = computed(() => [{ value: 'ALL', label: '전체 누적' }, ...terms.value.map(t => ({ value: t, label: t.replace('-', '학년 ') + '학기' }))])
  const contextLabel = computed(() => grade.value ? `${gradeYear.value}학년도 ${grade.value}학년${classNo.value ? ' ' + classNo.value + '반' : ''}` : '맥락 없음')

  async function load() {
    try {
      contexts.value = await invoke('get_contexts')
    } catch {
      contexts.value = []
    }
    loaded.value = true
    if (!current.value && contexts.value.length) {
      // 내신이 있는 학년을 우선, 없으면 첫 학년
      const pick = contexts.value.find(c => c.terms.length) ?? contexts.value[0]
      gradeYear.value = pick.grade_year
      grade.value = pick.grade
    }
    if (current.value && !classes.value.includes(classNo.value)) classNo.value = classes.value[0] ?? null
    if (term.value !== 'ALL' && !terms.value.includes(term.value)) term.value = 'ALL'
  }

  function setGrade(gy, g) {
    gradeYear.value = gy
    grade.value = g
    if (!classes.value.includes(classNo.value)) classNo.value = classes.value[0] ?? null
    if (term.value !== 'ALL' && !terms.value.includes(term.value)) term.value = 'ALL'
  }

  function openStudent(id) {
    studentId.value = id
    page.value = 'student'
  }

  watch(current, (c) => { if (c && !classes.value.includes(classNo.value)) classNo.value = classes.value[0] ?? null })

  return { contexts, gradeYear, grade, classNo, term, scope, studentId, page, loaded, current, classes, terms, termOptions, contextLabel, load, setGrade, openStudent }
})
