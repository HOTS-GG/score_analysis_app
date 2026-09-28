import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '../lib/ipc'

/** 플래그 기준값(thresholds) + 프로젝트 설정 */
export const useConfigStore = defineStore('config', () => {
  const thresholds = ref(null)
  const loading = ref(false)
  const error = ref('')

  async function loadAll() {
    await loadThresholds()
  }

  async function loadThresholds() {
    loading.value = true
    error.value = ''
    try {
      thresholds.value = await invoke('get_thresholds')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveThresholds(next) {
    thresholds.value = await invoke('set_thresholds', { thresholds: next })
    return thresholds.value
  }

  async function resetThresholds() {
    thresholds.value = await invoke('reset_thresholds')
    return thresholds.value
  }

  async function getConfig(key) {
    return await invoke('get_config', { key })
  }

  async function setConfig(key, value) {
    await invoke('set_config', { key, value: String(value) })
  }

  return { thresholds, loading, error, loadAll, loadThresholds, saveThresholds, resetThresholds, getConfig, setConfig }
})
