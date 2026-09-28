import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke, isTauri } from '../lib/ipc'

/** 내신 개인 분석 카드 (계산은 전부 Rust) */
export const useReportStore = defineStore('report', () => {
  const report = ref(null)
  const classReports = ref([])
  const loading = ref(false)
  const error = ref('')

  async function fetchStudentReport(studentId, gradeYear, semester) {
    loading.value = true
    error.value = ''
    try {
      report.value = await invoke('get_school_report', { studentId, gradeYear, semester })
    } catch (e) {
      error.value = String(e)
      report.value = null
    } finally {
      loading.value = false
    }
    return report.value
  }

  async function fetchClassReports(gradeYear, semester, grade, classNo) {
    loading.value = true
    error.value = ''
    try {
      classReports.value = await invoke('get_class_school_reports', { gradeYear, semester, grade, classNo })
    } catch (e) {
      error.value = String(e)
      classReports.value = []
    } finally {
      loading.value = false
    }
    return classReports.value
  }

  /** ArrayBuffer/Uint8Array 를 사용자가 고른 경로에 저장. 브라우저 미리보기에서는 다운로드. */
  async function saveBinary(bytes, defaultName, filterName = 'Excel', ext = 'xlsx') {
    const u8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes)
    if (!isTauri) {
      const blob = new Blob([u8])
      const a = document.createElement('a')
      a.href = URL.createObjectURL(blob)
      a.download = defaultName
      a.click()
      URL.revokeObjectURL(a.href)
      return defaultName
    }
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({ title: '저장 위치', defaultPath: defaultName, filters: [{ name: filterName, extensions: [ext] }] })
    if (!path) return null
    let bin = ''
    for (let i = 0; i < u8.length; i += 0x8000) bin += String.fromCharCode.apply(null, u8.subarray(i, i + 0x8000))
    await invoke('write_bytes_file', { path, dataBase64: btoa(bin) })
    return path
  }

  return { report, classReports, loading, error, fetchStudentReport, fetchClassReports, saveBinary }
})
