<script setup>
import { computed } from 'vue'
import { isTauri } from '../lib/ipc'
import schoolLogo from '../assets/school_logo.svg'
import { BookOpen, Database, FolderOpen, GraduationCap, LayoutGrid, Settings, UserRound, Users } from 'lucide-vue-next'

const props = defineProps({ active: String, filePath: String })
const emit = defineEmits(['select', 'go-home'])
const fileName = computed(() => props.filePath ? props.filePath.replace(/\\/g, '/').split('/').pop() : '')
async function openFolder() {
  if (!props.filePath || !isTauri) return
  const { revealItemInDir } = await import('@tauri-apps/plugin-opener')
  await revealItemInDir(props.filePath)
}
const items = [
  { id: 'class', label: '학급', desc: '학급 한눈에', icon: Users },
  { id: 'student', label: '학생', desc: '상담 리포트', icon: UserRound },
  { id: 'subject', label: '과목', desc: '성취 분포', icon: BookOpen },
  { id: 'school', label: '학년·학교', desc: '리포트·스크리닝', icon: LayoutGrid },
  { id: 'admission', label: '진학', desc: '참고선·입결', icon: GraduationCap },
  { id: 'data', label: '데이터', desc: '가져오기·점검', icon: Database },
]
</script>

<template>
  <aside class="rail">
    <button class="brand" @click="emit('go-home')" title="초기 화면으로">
      <img :src="schoolLogo" alt="" class="logo" />
      <span class="brand-text">성적분석ON</span>
    </button>
    <nav class="nav">
      <button v-for="it in items" :key="it.id" class="nav-btn" :class="{ on: active === it.id }" :aria-label="it.label" @click="emit('select', it.id)">
        <component :is="it.icon" :size="20" />
        <span class="nl">{{ it.label }}</span>
        <span class="nd">{{ it.desc }}</span>
      </button>
    </nav>
    <div class="foot">
      <button v-if="fileName" class="foot-btn" :title="filePath" @click="openFolder"><FolderOpen :size="16" /><span>{{ fileName }}</span></button>
      <button class="foot-btn" :class="{ on: active === 'settings' }" @click="emit('select', 'settings')" aria-label="설정"><Settings :size="16" /><span>설정</span></button>
    </div>
  </aside>
</template>

<style scoped>
.rail { width: 96px; flex-shrink: 0; display: flex; flex-direction: column; background: var(--bg-1); border-right: 1px solid var(--bd-1); }
.brand { display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 14px 6px 10px; border: none; background: none; cursor: pointer; border-bottom: 1px solid var(--bd-1); }
.logo { width: 30px; height: 30px; }
.brand-text { font-size: 11px; font-weight: 700; color: var(--tx-2); letter-spacing: -0.02em; }
.nav { flex: 1; display: flex; flex-direction: column; gap: 4px; padding: 10px 8px; }
.nav-btn { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 10px 4px 8px; border-radius: 12px; border: 1px solid transparent; background: none; color: var(--tx-3); cursor: pointer; transition: background-color 0.15s, color 0.15s; }
.nav-btn:hover { background: rgba(var(--accent-rgb), 0.1); color: var(--accent-text); }
.nav-btn.on { background: rgba(var(--accent-rgb), 0.16); border-color: rgba(var(--accent-rgb), 0.35); color: var(--accent-text); }
.nl { font-size: 13px; font-weight: 700; margin-top: 3px; }
.nd { font-size: 10px; color: var(--tx-4); }
.nav-btn.on .nd { color: var(--accent-text); opacity: 0.8; }
.foot { padding: 8px; border-top: 1px solid var(--bd-1); display: flex; flex-direction: column; gap: 4px; }
.foot-btn { display: flex; flex-direction: column; align-items: center; gap: 3px; padding: 8px 4px; border-radius: 10px; border: none; background: none; color: var(--tx-4); font-size: 10px; cursor: pointer; overflow: hidden; }
.foot-btn span { max-width: 80px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.foot-btn:hover, .foot-btn.on { background: rgba(var(--accent-rgb), 0.12); color: var(--accent-text); }
</style>
