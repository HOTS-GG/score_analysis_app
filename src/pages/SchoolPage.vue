<script setup>
import { ref } from 'vue'
import SchoolSection from '../sections/SchoolSection.vue'
import ExchangeSection from '../sections/ExchangeSection.vue'
import ScreeningSection from '../sections/ScreeningSection.vue'
import ItemSection from '../sections/ItemSection.vue'
import LongitudinalSection from '../sections/LongitudinalSection.vue'
import ValidationSection from '../sections/ValidationSection.vue'
import PreprocessSection from '../sections/PreprocessSection.vue'

const emit = defineEmits(['navigate'])
const tab = ref('report')
const tabs = [
  ['report', '학교 리포트', SchoolSection], ['exchange', '교내↔전국 환율표', ExchangeSection], ['screening', '스크리닝 명단', ScreeningSection],
  ['item', '문항 분석', ItemSection], ['longitudinal', '종단 분석', LongitudinalSection], ['preprocess', '전처리', PreprocessSection], ['validation', '품질 검증', ValidationSection],
]
function nav(id) {
  const map = { student: 'data', import: 'data', settings: 'settings', card: 'student' }
  emit('navigate', map[id] ?? id)
}
</script>

<template>
  <div class="wrap">
    <div class="tabs">
      <button v-for="[id, label] in tabs" :key="id" class="tab" :class="{ on: tab === id }" @click="tab = id">{{ label }}</button>
      <span class="hint">모의고사 전국 백분위 기반 학년·학교 분석 (스펙 L0~L7). 계산 엔진은 2단계에서 채워집니다.</span>
    </div>
    <div class="body">
      <component :is="tabs.find(t => t[0] === tab)[2]" @navigate="nav" />
    </div>
  </div>
</template>

<style scoped>
.wrap { display: flex; flex-direction: column; height: 100%; }
.tabs { display: flex; align-items: center; gap: 4px; padding: 8px 20px; border-bottom: 1px solid var(--bd-1); background: var(--bg-1); flex-shrink: 0; flex-wrap: wrap; }
.tab { padding: 6px 12px; border-radius: 8px; border: 1px solid transparent; background: none; color: var(--tx-3); font-size: 13px; cursor: pointer; }
.tab.on { background: rgba(var(--accent-rgb), 0.14); border-color: rgba(var(--accent-rgb), 0.35); color: var(--accent-text); font-weight: 600; }
.hint { margin-left: auto; font-size: 11.5px; color: var(--tx-4); }
.body { flex: 1; min-height: 0; overflow: hidden; }
</style>
