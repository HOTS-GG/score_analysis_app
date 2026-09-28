import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '../lib/ipc'

export const useProjectStore = defineStore('project', () => {
  const isOpen = ref(false)
  const filePath = ref('')

  function setProject(path) {
    filePath.value = path
    isOpen.value = true
  }

  function closeProject() {
    filePath.value = ''
    isOpen.value = false
  }

  async function newProject(path) {
    await invoke('new_project', { path })
    setProject(path)
  }

  async function openProject(path) {
    await invoke('open_project', { path })
    setProject(path)
  }

  async function backupProject() {
    await invoke('backup_project')
  }

  async function migrateSchema() {
    await invoke('migrate_schema')
  }

  async function checkAndUpdateVersion() {
    return await invoke('check_and_update_app_version')
  }

  return { isOpen, filePath, setProject, closeProject, newProject, openProject, backupProject, migrateSchema, checkAndUpdateVersion }
})
