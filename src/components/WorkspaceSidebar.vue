<script setup>
import { computed } from 'vue'
import { isTauri } from '../lib/ipc'
import schoolLogo from '../assets/school_logo.svg'
import {
  ArrowLeftRight, BarChart3, ChevronLeft, ChevronRight, Download, FolderOpen, HelpCircle, IdCard,
  LayoutDashboard, ListChecks, ListOrdered, Settings, ShieldCheck, Siren, TrendingUp, Upload, Users,
} from 'lucide-vue-next'

const props = defineProps({
  collapsed: Boolean,
  activeSection: String,
  filePath: String,
})
const emit = defineEmits(['update:collapsed', 'select', 'go-home'])

const fileName = computed(() => props.filePath ? props.filePath.replace(/\\/g, '/').split('/').pop() : '')

async function openFolder() {
  if (!props.filePath || !isTauri) return
  const { revealItemInDir } = await import('@tauri-apps/plugin-opener')
  await revealItemInDir(props.filePath)
}

const navGroups = [
  { title: '', items: [{ id: 'overview', label: '개요', icon: LayoutDashboard }] },
  {
    title: '데이터',
    items: [
      { id: 'student', label: '학생 명단', icon: Users },
      { id: 'import', label: '데이터 가져오기', icon: Download },
      { id: 'preprocess', label: '데이터 현황·전처리', icon: ListChecks },
    ],
  },
  {
    title: '분석',
    items: [
      { id: 'school', label: '학교 리포트', icon: BarChart3 },
      { id: 'exchange', label: '교내↔전국 환율표', icon: ArrowLeftRight },
      { id: 'screening', label: '스크리닝 명단', icon: Siren },
      { id: 'card', label: '학생 카드', icon: IdCard },
      { id: 'item', label: '문항 분석', icon: ListOrdered },
      { id: 'longitudinal', label: '종단 분석', icon: TrendingUp },
    ],
  },
  {
    title: '검증·출력',
    items: [
      { id: 'validation', label: '품질 검증', icon: ShieldCheck },
      { id: 'export', label: '내보내기', icon: Upload },
    ],
  },
  { title: '', items: [{ id: 'manual', label: '사용 설명서', icon: HelpCircle }] },
]
</script>

<template>
  <aside :class="['sidebar', collapsed ? 'sidebar--collapsed' : '']">
    <div class="sidebar-header">
      <div v-if="!collapsed" class="sidebar-title" @click="emit('go-home')" title="초기 화면으로">
        <img :src="schoolLogo" alt="로고" class="sidebar-logo" />
        <span class="title-text">성적분석ON</span>
      </div>
      <img v-else :src="schoolLogo" alt="초기 화면으로" class="sidebar-logo sidebar-logo--collapsed" @click="emit('go-home')" />
      <button class="toggle-btn" @click="emit('update:collapsed', !collapsed)" :title="collapsed ? '사이드바 열기' : '사이드바 접기'">
        <ChevronLeft v-if="!collapsed" :size="18" />
        <ChevronRight v-else :size="18" />
      </button>
    </div>

    <nav class="sidebar-nav">
      <template v-for="(group, gi) in navGroups" :key="gi">
        <div v-if="gi > 0" class="nav-divider" />
        <div v-if="group.title && !collapsed" class="nav-group-title">{{ group.title }}</div>
        <button
          v-for="item in group.items"
          :key="item.id"
          :class="['nav-item', activeSection === item.id ? 'nav-item--active' : '']"
          @click="emit('select', item.id)"
          :title="item.label"
          :aria-label="item.label"
        >
          <component :is="item.icon" :size="19" class="nav-icon" />
          <span v-if="!collapsed" class="nav-label">{{ item.label }}</span>
        </button>
      </template>
    </nav>

    <div class="sidebar-footer">
      <div class="footer-divider" />
      <button v-if="fileName" class="footer-btn" @click="openFolder" :title="filePath">
        <FolderOpen :size="19" class="footer-icon" />
        <span v-if="!collapsed" class="file-name">{{ fileName }}</span>
      </button>
      <button
        class="footer-btn"
        :class="{ 'footer-btn--active': activeSection === 'settings' }"
        @click="emit('select', 'settings')"
        title="설정"
      >
        <Settings :size="19" class="footer-icon" />
        <span v-if="!collapsed">설정 · 기준값</span>
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar { display: flex; flex-direction: column; width: 232px; min-height: 100vh; background-color: var(--bg-1); border-right: 1px solid var(--bd-1); transition: width 0.25s ease; overflow: hidden; flex-shrink: 0; }
.sidebar--collapsed { width: 60px; }
.sidebar-header { display: flex; align-items: center; justify-content: space-between; padding: 16px 12px 12px; border-bottom: 1px solid var(--bd-1); min-height: 60px; gap: 8px; }
.sidebar-title { display: flex; align-items: center; gap: 6px; overflow: hidden; white-space: nowrap; cursor: pointer; border-radius: 6px; padding: 2px 4px; margin: -2px -4px; }
.sidebar-title:hover { background-color: rgba(var(--accent-rgb), 0.1); }
.sidebar-logo { width: 22px; height: 22px; object-fit: contain; flex-shrink: 0; }
.sidebar-logo--collapsed { cursor: pointer; width: 26px; height: 26px; }
.title-text { font-size: 15px; font-weight: 600; color: var(--tx-2); white-space: nowrap; }
.toggle-btn { display: flex; align-items: center; justify-content: center; width: 32px; height: 32px; flex-shrink: 0; border-radius: 8px; background: none; border: none; color: var(--clr-text-hint); cursor: pointer; margin-left: auto; }
.toggle-btn:hover { background-color: var(--bd-1); color: var(--accent-text); }
.sidebar-nav { flex: 1; padding: 10px 8px; display: flex; flex-direction: column; gap: 2px; overflow-y: auto; }
.nav-divider { height: 1px; background-color: var(--bd-2); margin: 6px 4px; }
.nav-group-title { font-size: 11px; font-weight: 600; color: var(--tx-4); letter-spacing: 0.08em; padding: 6px 10px 2px; text-transform: uppercase; }
.nav-item { display: flex; align-items: center; gap: 10px; width: 100%; padding: 8px 10px; border-radius: 10px; background: none; border: none; color: var(--tx-3); cursor: pointer; font-size: 14px; font-weight: 500; text-align: left; white-space: nowrap; transition: background-color 0.15s, color 0.15s; }
.nav-item:hover { background-color: rgba(var(--accent-rgb), 0.18); color: var(--accent-text); }
.nav-item--active { background-color: rgba(var(--accent-rgb), 0.2); color: var(--accent-bright); }
.nav-icon { flex-shrink: 0; }
.sidebar--collapsed .nav-item { justify-content: center; padding: 9px; }
.sidebar-footer { padding: 8px; }
.footer-divider { height: 1px; background-color: var(--bd-1); margin-bottom: 8px; }
.footer-btn { display: flex; align-items: center; gap: 8px; width: 100%; padding: 8px 10px; border-radius: 10px; background: none; border: none; color: var(--tx-3); cursor: pointer; font-size: 14px; text-align: left; white-space: nowrap; overflow: hidden; transition: background-color 0.15s; }
.footer-btn:hover { background-color: var(--bg-hover-bright); }
.footer-btn--active { background-color: rgba(var(--accent-rgb), 0.2); color: var(--accent-bright); }
.sidebar--collapsed .footer-btn { justify-content: center; padding: 8px; }
.footer-icon { flex-shrink: 0; }
.file-name { overflow: hidden; text-overflow: ellipsis; }
</style>
