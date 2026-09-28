<script setup>
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { isTauri } from '../lib/ipc'
import { useProjectStore } from '../stores/project'
import { useConfigStore } from '../stores/configStore'
import { useThemeStore } from '../stores/theme'
import { useDatasetStore } from '../stores/dataset'
import { useContextStore } from '../stores/context'
import TopBar from '../components/TopBar.vue'
import NavRail from '../components/NavRail.vue'
import ClassPage from '../pages/ClassPage.vue'
import StudentPage from '../pages/StudentPage.vue'
import SubjectPage from '../pages/SubjectPage.vue'
import SchoolPage from '../pages/SchoolPage.vue'
import AdmissionPage from '../pages/AdmissionPage.vue'
import DataPage from '../pages/DataPage.vue'
import SettingsSection from '../sections/SettingsSection.vue'

const router = useRouter()
const project = useProjectStore()
const config = useConfigStore()
const themeStore = useThemeStore()
const dataset = useDatasetStore()
const ctx = useContextStore()

const pages = { class: ClassPage, student: StudentPage, subject: SubjectPage, school: SchoolPage, admission: AdmissionPage, data: DataPage, settings: SettingsSection }
const current = computed(() => pages[ctx.page] ?? ClassPage)

function goHome() {
  project.closeProject()
  router.push('/')
}

onMounted(async () => {
  await themeStore.loadAndApply()
  if (isTauri) {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      const { LogicalSize } = await import('@tauri-apps/api/dpi')
      const win = getCurrentWindow()
      await win.setResizable(true)
      await win.setMinSize(new LogicalSize(1024, 640))
      await win.setSize(new LogicalSize(1440, 860))
      await win.center()
    } catch { /* ignore */ }
  }
  try { await config.loadAll() } catch { /* ignore */ }
  try { await Promise.all([dataset.loadSpecs(), dataset.refreshSummary(), ctx.load()]) } catch { /* ignore */ }
  if (!ctx.contexts.length) ctx.page = 'data'
})
</script>

<template>
  <div class="workspace">
    <NavRail :active="ctx.page" :file-path="project.filePath" @select="ctx.page = $event" @go-home="goHome" />
    <div class="main">
      <TopBar />
      <main class="page">
        <component :is="current" @navigate="ctx.page = $event" />
      </main>
    </div>
  </div>
</template>

<style scoped>
.workspace { display: flex; height: 100vh; background: var(--bg-0); overflow: hidden; }
.main { flex: 1; display: flex; flex-direction: column; min-width: 0; }
.page { flex: 1; overflow: hidden; min-height: 0; }
</style>
