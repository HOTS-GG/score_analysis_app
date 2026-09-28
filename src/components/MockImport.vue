<script setup>
import { computed, ref } from 'vue'
import { Workbook } from 'exceljs'
import { AlertTriangle, CheckCircle2, ChevronDown, ChevronUp, FileSpreadsheet, Trash2 } from 'lucide-vue-next'
import { useAnalysisStore } from '../stores/analysis'
import { useDatasetStore } from '../stores/dataset'
import { useContextStore } from '../stores/context'
import { invoke } from '../lib/ipc'
import ImportSection from '../sections/ImportSection.vue'

const emit = defineEmits(['navigate'])
const analysis = useAnalysisStore()
const dataset = useDatasetStore()
const ctx = useContextStore()
const gradeYear = ref(new Date().getFullYear())
const createMissing = ref(true)
const items = ref([]) // {file, sheet, grid, parsed, round, error, result}
const busy = ref(false)
const fileInput = ref(null)
const showMapping = ref(false)
const ROUNDS = [3, 5, 6, 7, 9, 10, 11]

function cellValue(v) {
  if (v === null || v === undefined) return null
  if (typeof v === 'object') {
    if (v.result !== undefined) return cellValue(v.result)
    if (v.richText) return v.richText.map(t => t.text).join('')
    if (v.text) return v.text
    if (v instanceof Date) return v.toISOString().slice(0, 10)
    return String(v)
  }
  return v
}
function sheetGrid(ws) {
  const grid = []
  ws.eachRow({ includeEmpty: true }, (row, rn) => {
    const arr = []
    row.eachCell({ includeEmpty: true }, (cell, cn) => { arr[cn - 1] = cellValue(cell.value) })
    grid[rn - 1] = Array.from(arr, v => v ?? null)
  })
  return grid.map(r => r ?? [])
}
async function onFiles(ev) {
  const list = [...(ev.target.files ?? [])]
  ev.target.value = ''
  busy.value = true
  for (const f of list) {
    if (/\.xlsb$/i.test(f.name)) { items.value.push({ file: f.name, sheet: '', error: 'xlsb 형식은 읽을 수 없습니다. 엑셀에서 "다른 이름으로 저장 → Excel 통합 문서(*.xlsx)"로 저장한 뒤 넣어 주세요.' }); continue }
    try {
      const wb = new Workbook()
      await wb.xlsx.load(await f.arrayBuffer())
      let any = false
      for (const ws of wb.worksheets) {
        const grid = sheetGrid(ws)
        if (!grid.length) continue
        try {
          const parsed = await invoke('parse_mock_file', { grid })
          const roundFromSheet = (ws.name.match(/(\d+)\s*월/) || [])[1]
          const round = parsed.round ?? (roundFromSheet ? Number(roundFromSheet) : (parsed.layout === 'raw' ? 9 : null))
          items.value.push({ file: f.name, sheet: ws.name, grid, parsed, round, error: '', result: null })
          any = true
        } catch { /* 이 시트는 성적 시트가 아님 */ }
      }
      if (!any) items.value.push({ file: f.name, sheet: '', error: '성적 시트를 찾지 못했습니다. 아래 "직접 매핑"으로 넣어 보세요.' })
    } catch (e) {
      items.value.push({ file: f.name, sheet: '', error: String(e) })
    }
  }
  busy.value = false
}
const ready = computed(() => items.value.filter(i => i.parsed && !i.result && i.round))
async function importAll() {
  busy.value = true
  for (const it of ready.value) {
    try { it.result = await invoke('import_mock_file', { grid: it.grid, gradeYear: Number(gradeYear.value), round: Number(it.round), fileName: `${it.file} [${it.sheet}]`, createMissing: createMissing.value }) }
    catch (e) { it.error = String(e) }
  }
  busy.value = false
  await Promise.all([dataset.refreshSummary(), ctx.load()])
}
const totals = computed(() => items.value.reduce((a, i) => { if (i.result) { a.inserted += i.result.inserted; a.updated += i.result.updated; a.skipped += i.result.skipped } return a }, { inserted: 0, updated: 0, skipped: 0 }))
const layoutLabel = (l) => l === 'analysis' ? '학력평가 종합분석 (표준점수·백분위·등급)' : '학급 가채점 (원점수만)'
</script>

<template>
  <div class="section">
    <div class="section-header compact">
      <div>
        <h2 class="section-title">모의고사 성적 가져오기</h2>
        <div class="section-desc">
          <p><b>학력평가 성적 종합 분석</b> 파일이나 <b>학급 가채점</b> 파일을 그대로 넣으세요. 학번·과목 열을 자동으로 찾아 읽습니다. 매핑 화면은 필요 없습니다.</p>
          <p>종합분석 파일이 <b>.xlsb</b>라면 엑셀에서 "다른 이름으로 저장 → Excel 통합 문서(.xlsx)"로 한 번 저장해 주세요. 미응시는 자동으로 결측 처리됩니다(0점 아님).</p>
        </div>
      </div>
      <div class="header-actions">
        <div class="filter-field"><span class="filter-label">학년도</span><input v-model="gradeYear" type="number" class="ui-input ui-input--sm" style="width:100px" /></div>
        <label class="checkbox-row"><input v-model="createMissing" type="checkbox" /> 명단에 없는 학생 자동 생성</label>
        <input ref="fileInput" type="file" accept=".xlsx,.xlsm,.xlsb" multiple style="display:none" @change="onFiles" />
        <button class="btn-add" :disabled="busy" @click="fileInput.click()"><FileSpreadsheet :size="16" /> 파일 선택</button>
      </div>
    </div>
    <div class="section-body">
      <div class="steps">
        <div class="step"><span class="num">1</span><div><b>파일 선택</b><p>여러 파일을 한꺼번에 골라도 됩니다. 시트마다 회차(3월·6월…)를 알아서 읽습니다.</p></div></div>
        <div class="step"><span class="num">2</span><div><b>회차 확인</b><p>표에서 학년·회차·학생 수가 맞는지 보고, 틀리면 회차 칸을 바꿉니다.</p></div></div>
        <div class="step"><span class="num">3</span><div><b>가져오기</b><p>같은 학생·회차·과목이 이미 있으면 새 값으로 갱신됩니다.</p></div></div>
      </div>

      <div v-if="!items.length" class="placeholder" style="min-height:110px">
        <span class="placeholder-title">아직 선택한 파일이 없습니다</span>
        <span>예: 「2026학년도 고1 6월 학력평가 성적 종합 분석.xlsx」, 「9월 학력평가 학급 가채점.xlsx」</span>
      </div>
      <template v-else>
        <div class="table-wrap"><table class="data-table"><thead><tr><th>파일</th><th>시트</th><th>인식한 양식</th><th class="n">학년</th><th>회차</th><th class="n">학생</th><th>과목</th><th>상태</th></tr></thead><tbody>
          <tr v-for="(it, i) in items" :key="i">
            <td class="mono">{{ it.file }}</td><td class="muted">{{ it.sheet }}</td>
            <td>{{ it.parsed ? layoutLabel(it.parsed.layout) : '—' }}</td>
            <td class="n">{{ it.parsed?.grade ?? '' }}</td>
            <td><select v-if="it.parsed && !it.result" v-model="it.round" class="ui-select" style="min-width:80px"><option :value="null">선택</option><option v-for="r in ROUNDS" :key="r" :value="r">{{ r }}월</option></select><span v-else>{{ it.round ? it.round + '월' : '' }}</span></td>
            <td class="n">{{ it.parsed?.students ?? '' }}</td>
            <td class="muted small">{{ it.parsed?.subjects.join(' · ') ?? '' }}</td>
            <td>
              <span v-if="it.error" class="badge badge--red" :title="it.error">{{ it.error.slice(0, 70) }}</span>
              <span v-else-if="it.result" class="badge badge--green"><CheckCircle2 :size="12" /> 신규 {{ it.result.inserted }} · 갱신 {{ it.result.updated }}{{ it.result.skipped ? ` · 건너뜀 ${it.result.skipped}` : '' }}</span>
              <span v-else-if="!it.round" class="badge badge--warn">회차를 골라 주세요</span>
              <span v-else class="badge badge--muted">읽음 · 대기</span>
              <span v-if="it.parsed?.warnings?.length" class="badge badge--warn" :title="it.parsed.warnings.join('\n')">주의 {{ it.parsed.warnings.length }}</span>
              <span v-if="it.result?.unmatched?.length" class="badge badge--warn" :title="it.result.unmatched.join('\n')">명단 없음 {{ it.result.unmatched.length }}명</span>
            </td>
          </tr>
        </tbody></table></div>
        <div class="row">
          <div v-if="totals.inserted || totals.updated" class="note note--accent grow"><CheckCircle2 :size="14" /><div>신규 {{ totals.inserted }} · 갱신 {{ totals.updated }} · 건너뜀 {{ totals.skipped }}. <span class="link" @click="emit('navigate', 'check')">데이터 점검</span>에서 회차별 응시 인원을 확인하세요.</div></div>
          <span v-else class="grow" />
          <button class="btn-ghost" @click="items = []"><Trash2 :size="14" /> 목록 비우기</button>
          <button class="btn-add" :disabled="!ready.length || busy" @click="importAll">{{ busy ? '처리 중…' : `${ready.length}개 가져오기` }}</button>
        </div>
      </template>

      <div class="note"><AlertTriangle :size="14" /><div>가채점 파일은 원점수만 있어 회차 간 비교에는 쓰지 않습니다. 정식 성적표가 나오면 같은 회차로 다시 가져오면 백분위·등급이 채워집니다.</div></div>

      <div class="panel">
        <button class="toggle" @click="showMapping = !showMapping">
          <component :is="showMapping ? ChevronUp : ChevronDown" :size="16" />
          양식이 다르거나 입학(고입내신·반배치)·문항·출결 파일을 넣으려면 — 직접 매핑
        </button>
        <div v-if="showMapping" class="mapping"><ImportSection @navigate="emit('navigate', $event)" /></div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.compact { padding: 18px 28px 12px; align-items: center; }
.section-body { padding: 16px 28px 40px; }
.steps { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
.step { display: flex; gap: 10px; padding: 12px 14px; border: 1px solid var(--bd-1); border-radius: 12px; background: var(--bg-1); }
.step .num { width: 24px; height: 24px; border-radius: 50%; background: var(--accent-hex); color: #fff; font-weight: 700; font-size: 12px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
.step b { font-size: 13px; color: var(--tx-1); }
.step p { margin: 3px 0 0; font-size: 12px; color: var(--tx-3); line-height: 1.5; }
.n { text-align: right; }
.muted { color: var(--tx-4); }
.small { font-size: 11.5px; }
.toggle { display: flex; align-items: center; gap: 8px; border: none; background: none; color: var(--tx-2); font-size: 13.5px; font-weight: 600; cursor: pointer; padding: 0; }
.mapping { border-top: 1px solid var(--bd-1); margin-top: 8px; height: 720px; overflow: hidden; border-radius: 12px; }
@media (max-width: 1100px) { .steps { grid-template-columns: 1fr; } }
</style>
