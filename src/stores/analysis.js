import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '../lib/ipc'

/** 분석 결과 캐시. 계산은 전부 Rust. */
export const useAnalysisStore = defineStore('analysis', () => {
  const classReport = ref(null)
  const bundle = ref(null)
  const subjectOptions = ref([])
  const subjectReport = ref(null)
  const admissionAgg = ref([])
  const regional = ref([])
  const reference = ref([])
  const catalog = ref([])
  const loading = ref(false)
  const error = ref('')

  async function run(fn) {
    loading.value = true
    error.value = ''
    try {
      return await fn()
    } catch (e) {
      error.value = String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  const fetchClassReport = (gradeYear, grade, classNo, term, scope) =>
    run(async () => { classReport.value = await invoke('get_class_report', { gradeYear, grade, classNo, term, scope }); return classReport.value })
  const fetchBundle = (studentId, term, scope) =>
    run(async () => { bundle.value = await invoke('get_student_bundle', { studentId, term, scope }); return bundle.value })
  const fetchSubjectOptions = (gradeYear, grade) =>
    run(async () => { subjectOptions.value = await invoke('get_subject_options', { gradeYear, grade }); return subjectOptions.value })
  const fetchSubjectReport = (gradeYear, grade, term, subject, classNo) =>
    run(async () => { subjectReport.value = await invoke('get_subject_report', { gradeYear, grade, term, subject, classNo: classNo ?? null }); return subjectReport.value })
  const fetchAdmission = (minApply = 3) =>
    run(async () => { admissionAgg.value = await invoke('get_admission_aggregate', { minApply }); return admissionAgg.value })
  const fetchRegional = () => run(async () => { regional.value = await invoke('get_regional_admission'); return regional.value })
  const fetchReference = () => run(async () => { reference.value = await invoke('get_univ_reference'); return reference.value })
  const saveReference = (id, cut, note) => run(async () => { await invoke('set_univ_reference', { id, cut, note: note ?? null }); return fetchReference() })
  const fetchCatalog = () => run(async () => { catalog.value = await invoke('get_subject_catalog'); return catalog.value })
  const saveCatalog = (items) => run(async () => { const n = await invoke('set_subject_catalog', { items }); await fetchCatalog(); return n })
  const search = (q) => invoke('search_student', { q })
  const parseNeis = (grid) => invoke('parse_neis', { grid })
  const importNeis = (grid, gradeYear, fileName, createMissing) => invoke('import_neis', { grid, gradeYear, fileName, createMissing })

  return {
    classReport, bundle, subjectOptions, subjectReport, admissionAgg, regional, reference, catalog, loading, error,
    fetchClassReport, fetchBundle, fetchSubjectOptions, fetchSubjectReport, fetchAdmission, fetchRegional, fetchReference, saveReference,
    fetchCatalog, saveCatalog, search, parseNeis, importNeis,
  }
})
