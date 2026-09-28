<script setup>
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { Moon, Search, Sun, X } from 'lucide-vue-next'
import { useContextStore } from '../stores/context'
import { useAnalysisStore } from '../stores/analysis'
import { useThemeStore } from '../stores/theme'

const ctx = useContextStore()
const analysis = useAnalysisStore()
const theme = useThemeStore()
const q = ref('')
const results = ref([])
const open = ref(false)
let timer = null

watch(q, (v) => {
  clearTimeout(timer)
  if (!v.trim()) { results.value = []; open.value = false; return }
  timer = setTimeout(async () => {
    try { results.value = await analysis.search(v); open.value = true } catch { results.value = [] }
  }, 180)
})
function pick(s) {
  ctx.setGrade(ctxYearOf(s.grade), s.grade)
  ctx.classNo = s.class_no
  ctx.openStudent(s.id)
  q.value = ''
  open.value = false
}
function ctxYearOf(grade) {
  const c = ctx.contexts.find(x => x.grade === grade)
  return c ? c.grade_year : ctx.gradeYear
}
function onKey(e) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); document.getElementById('global-search')?.focus() }
  if (e.key === 'Escape') open.value = false
}
onMounted(() => window.addEventListener('keydown', onKey))
onUnmounted(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <header class="topbar">
    <div class="ctx">
      <select class="ui-select sel" :value="`${ctx.gradeYear}-${ctx.grade}`" @change="e => { const [y, g] = e.target.value.split('-').map(Number); ctx.setGrade(y, g) }">
        <option v-if="!ctx.contexts.length" value="null-null">학년 데이터 없음</option>
        <option v-for="c in ctx.contexts" :key="c.grade_year + '-' + c.grade" :value="`${c.grade_year}-${c.grade}`">{{ c.grade_year }}학년도 {{ c.grade }}학년 ({{ c.students }}명)</option>
      </select>
      <select class="ui-select sel sel--sm" v-model="ctx.classNo" :disabled="!ctx.classes.length">
        <option v-for="c in ctx.classes" :key="c" :value="c">{{ c }}반</option>
      </select>
      <select class="ui-select sel" v-model="ctx.term">
        <option v-for="t in ctx.termOptions" :key="t.value" :value="t.value">{{ t.label }}</option>
      </select>
      <div class="seg">
        <button :class="{ on: ctx.scope === 'main' }" @click="ctx.scope = 'main'" title="국·수·영·사·과">주요교과</button>
        <button :class="{ on: ctx.scope === 'all' }" @click="ctx.scope = 'all'">전 과목</button>
      </div>
    </div>
    <div class="search" :class="{ open }">
      <Search :size="15" class="ic" />
      <input id="global-search" v-model="q" placeholder="학생 검색 — 이름 · 학번 (Ctrl+K)" @focus="open = results.length > 0" />
      <button v-if="q" class="clr" @click="q = ''"><X :size="13" /></button>
      <ul v-if="open && results.length" class="dropdown">
        <li v-for="s in results" :key="s.id" @mousedown.prevent="pick(s)">
          <b>{{ s.name || s.anon_id }}</b><span>{{ s.grade }}학년 {{ s.class_no }}반 {{ s.number }}번 · {{ s.anon_id }}</span>
        </li>
      </ul>
    </div>
    <button class="btn-icon" :title="theme.isDark ? '라이트 모드' : '다크 모드'" @click="theme.setMode(!theme.isDark)">
      <Sun v-if="theme.isDark" :size="17" /><Moon v-else :size="17" />
    </button>
  </header>
</template>

<style scoped>
.topbar { display: flex; align-items: center; gap: 14px; padding: 10px 18px; border-bottom: 1px solid var(--bd-1); background: var(--bg-1); flex-shrink: 0; }
.ctx { display: flex; align-items: center; gap: 8px; }
.sel { min-width: 150px; padding: 7px 10px; font-size: 13px; }
.sel--sm { min-width: 76px; }
.search { position: relative; flex: 1; max-width: 420px; margin-left: auto; display: flex; align-items: center; gap: 8px; padding: 0 10px; border: 1px solid var(--bd-1); border-radius: 10px; background: var(--bg-2); color: var(--tx-4); }
.search input { flex: 1; border: none; background: none; padding: 8px 0; font-size: 13.5px; color: var(--tx-1); outline: none; }
.clr { border: none; background: none; color: var(--tx-4); cursor: pointer; display: flex; }
.dropdown { position: absolute; top: calc(100% + 4px); left: 0; right: 0; z-index: 30; margin: 0; padding: 4px; list-style: none; background: var(--bg-2); border: 1px solid var(--bd-1); border-radius: 12px; box-shadow: 0 12px 30px var(--shadow); max-height: 320px; overflow-y: auto; }
.dropdown li { display: flex; align-items: baseline; gap: 10px; padding: 8px 10px; border-radius: 8px; cursor: pointer; font-size: 13px; color: var(--tx-2); }
.dropdown li:hover { background: rgba(var(--accent-rgb), 0.12); }
.dropdown li b { color: var(--tx-1); }
.dropdown li span { color: var(--tx-4); font-size: 12px; }
</style>
