<script setup>
import { ref } from 'vue'
import NeisImport from '../components/NeisImport.vue'
import MockImport from '../components/MockImport.vue'
import StudentSection from '../sections/StudentSection.vue'
import DataCheck from '../components/DataCheck.vue'
import ExportSection from '../sections/ExportSection.vue'
import ManualSection from '../sections/ManualSection.vue'

const emit = defineEmits(['navigate'])
const tab = ref('neis')
const tabs = [
  ['neis', '나이스 내신 가져오기', '반별 교과학습발달상황 파일'],
  ['mock', '모의고사 성적 가져오기', '학력평가 종합분석 · 가채점'],
  ['student', '학생 명단', '추가 · 제외 · 복원'],
  ['check', '데이터 점검', '무엇이 있고 무엇이 빠졌나'],
  ['export', '내보내기', '리포트 · 명단 · 파일'],
  ['manual', '설명서', '처음 쓰는 분께'],
]
function nav(id) {
  if (['neis', 'mock', 'student', 'check', 'export', 'manual'].includes(id)) { tab.value = id; return }
  if (id === 'import') { tab.value = 'mock'; return }
  emit('navigate', id === 'card' ? 'student' : id)
}
</script>

<template>
  <div class="wrap">
    <div class="tabs">
      <button v-for="[id, label, desc] in tabs" :key="id" class="tab" :class="{ on: tab === id }" @click="tab = id"><b>{{ label }}</b><small>{{ desc }}</small></button>
    </div>
    <div class="body">
      <NeisImport v-if="tab === 'neis'" @navigate="nav" />
      <MockImport v-else-if="tab === 'mock'" @navigate="nav" />
      <StudentSection v-else-if="tab === 'student'" @navigate="nav" />
      <DataCheck v-else-if="tab === 'check'" @navigate="nav" @tab="tab = $event" />
      <ExportSection v-else-if="tab === 'export'" @navigate="nav" />
      <ManualSection v-else @navigate="nav" />
    </div>
  </div>
</template>

<style scoped>
.wrap { display: flex; flex-direction: column; height: 100%; }
.tabs { display: flex; align-items: stretch; gap: 6px; padding: 8px 20px; border-bottom: 1px solid var(--bd-1); background: var(--bg-1); flex-shrink: 0; flex-wrap: wrap; }
.tab { display: flex; flex-direction: column; align-items: flex-start; gap: 1px; padding: 7px 14px; border-radius: 10px; border: 1px solid transparent; background: none; color: var(--tx-3); cursor: pointer; text-align: left; }
.tab b { font-size: 13px; }
.tab small { font-size: 10.5px; color: var(--tx-4); }
.tab.on { background: rgba(var(--accent-rgb), 0.14); border-color: rgba(var(--accent-rgb), 0.35); color: var(--accent-text); }
.tab.on small { color: var(--accent-text); opacity: 0.8; }
.body { flex: 1; min-height: 0; overflow: hidden; }
</style>
