import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '../lib/ipc'

export const useStudentStore = defineStore('student', () => {
  const students = ref([])
  const exclusionLog = ref([])
  const loading = ref(false)
  const error = ref('')

  const activeStudents = computed(() => students.value.filter(s => s.status === 'active'))

  /** (입학연도, 학년, 반) 단위 그룹 */
  const grouped = computed(() => {
    const map = new Map()
    for (const s of students.value) {
      const key = `${s.cohort}-${s.grade}-${s.class_no}`
      if (!map.has(key)) map.set(key, { cohort: s.cohort, grade: s.grade, classNo: s.class_no, students: [] })
      map.get(key).students.push(s)
    }
    return [...map.values()]
  })

  async function fetchStudents() {
    loading.value = true
    error.value = ''
    try {
      students.value = await invoke('get_students')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function fetchExclusionLog() {
    exclusionLog.value = await invoke('get_exclusion_log')
  }

  async function createStudent(student) {
    await invoke('create_student', { student })
    await fetchStudents()
  }

  async function updateStudent(id, student) {
    await invoke('update_student', { id, student })
    await fetchStudents()
  }

  async function deleteStudent(id) {
    await invoke('delete_student', { id })
    await fetchStudents()
  }

  async function bulkUpsertStudents(list) {
    const r = await invoke('bulk_upsert_students', { students: list })
    await fetchStudents()
    return r
  }

  async function setStatus(id, status, reason) {
    await invoke('set_student_status', { id, status, reason: reason ?? null })
    await Promise.all([fetchStudents(), fetchExclusionLog()])
  }

  return { students, exclusionLog, loading, error, activeStudents, grouped, fetchStudents, fetchExclusionLog, createStudent, updateStudent, deleteStudent, bulkUpsertStudents, setStatus }
})
