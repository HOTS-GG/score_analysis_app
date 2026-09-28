import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '../lib/ipc'

/** 데이터셋 정의·현황·가져오기 */
export const useDatasetStore = defineStore('dataset', () => {
  const specs = ref([])
  const summary = ref(null)
  const logs = ref([])
  const loading = ref(false)
  const error = ref('')

  const specById = computed(() => Object.fromEntries(specs.value.map(s => [s.id, s])))

  /** 필터 UI용 선택지 (모의고사 회차 기준) */
  const filterOptions = computed(() => {
    const rounds = summary.value?.mock_rounds ?? []
    const years = [...new Set(rounds.map(r => r.grade_year))].sort()
    const grades = [...new Set(rounds.map(r => r.grade))].sort()
    const roundNos = [...new Set(rounds.map(r => r.round))].sort((a, b) => a - b)
    const subjects = [...new Set(rounds.flatMap(r => r.subjects))]
    return { years, grades, rounds: roundNos, subjects }
  })

  const hasMock = computed(() => (summary.value?.mock_rows ?? 0) > 0)
  const hasSchool = computed(() => (summary.value?.school_rows ?? 0) > 0)
  const hasEntry = computed(() => (summary.value?.entry_rows ?? 0) > 0)
  const hasItems = computed(() => (summary.value?.item_stat_rows ?? 0) > 0)
  const hasItemResponses = computed(() => (summary.value?.item_response_rows ?? 0) > 0)
  const hasAttendance = computed(() => (summary.value?.attendance_rows ?? 0) > 0)
  /** 추이 분석 가능 여부: 같은 학년도·학년에 회차 2개 이상 */
  const mockRoundCount = computed(() => {
    const rounds = summary.value?.mock_rounds ?? []
    const byYear = {}
    for (const r of rounds) {
      const k = `${r.grade_year}-${r.grade}`
      byYear[k] = (byYear[k] ?? 0) + 1
    }
    return Math.max(0, ...Object.values(byYear))
  })

  async function loadSpecs() {
    if (specs.value.length) return
    specs.value = await invoke('get_dataset_specs')
  }

  async function refreshSummary() {
    loading.value = true
    error.value = ''
    try {
      summary.value = await invoke('get_dataset_summary')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function refreshLogs() {
    logs.value = await invoke('get_import_logs')
  }

  async function suggestMapping(dataset, headers) {
    return await invoke('suggest_mapping', { dataset, headers })
  }

  async function importDataset(dataset, rows, mapping, fileName) {
    const result = await invoke('import_dataset', { dataset, rows, mapping, fileName })
    await Promise.all([refreshSummary(), refreshLogs()])
    return result
  }

  async function clearDataset(dataset) {
    const n = await invoke('clear_dataset', { dataset })
    await refreshSummary()
    return n
  }

  return {
    specs, summary, logs, loading, error, specById, filterOptions,
    hasMock, hasSchool, hasEntry, hasItems, hasItemResponses, hasAttendance, mockRoundCount,
    loadSpecs, refreshSummary, refreshLogs, suggestMapping, importDataset, clearDataset,
  }
})
