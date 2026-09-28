<script setup>
import { computed, onMounted, ref } from 'vue'
import { Workbook } from 'exceljs'
import { AlertTriangle, CheckCircle2, FileSpreadsheet, Trash2 } from 'lucide-vue-next'
import { useAnalysisStore } from '../stores/analysis'
import { useDatasetStore } from '../stores/dataset'
import { useContextStore } from '../stores/context'

const emit = defineEmits(['navigate'])
const analysis = useAnalysisStore()
const dataset = useDatasetStore()
const ctx = useContextStore()
const gradeYear = ref(new Date().getFullYear())
const createMissing = ref(true)
const files = ref([]) // {name, grid, parsed, error, result}
const busy = ref(false)
const fileInput = ref(null)
const catalogText = ref('')
const catalogMsg = ref('')

onMounted(async () => {
  await analysis.fetchCatalog()
  catalogText.value = analysis.catalog.map(([s, , k]) => `${s}\t${k}`).join('\n')
})

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
async function readGrid(file) {
  const wb = new Workbook()
  await wb.xlsx.load(await file.arrayBuffer())
  const ws = wb.worksheets[0]
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
    const item = { name: f.name, grid: null, parsed: null, error: '', result: null }
    try {
      item.grid = await readGrid(f)
      item.parsed = await analysis.parseNeis(item.grid)
    } catch (e) {
      item.error = String(e)
    }
    files.value.push(item)
  }
  files.value.sort((a, b) => (a.parsed?.grade ?? 9) - (b.parsed?.grade ?? 9) || (a.parsed?.class_no ?? 99) - (b.parsed?.class_no ?? 99))
  busy.value = false
}
const ready = computed(() => files.value.filter(f => f.parsed && !f.result))
async function importAll() {
  busy.value = true
  for (const f of ready.value) {
    try { f.result = await analysis.importNeis(f.grid, Number(gradeYear.value), f.name, createMissing.value) }
    catch (e) { f.error = String(e) }
  }
  busy.value = false
  await Promise.all([dataset.refreshSummary(), ctx.load()])
}
async function saveCatalog() {
  const items = catalogText.value.split('\n').map(l => l.split(/\t|,|\s{2,}/).map(x => x.trim())).filter(p => p[0] && p[1]).map(p => [p[0], null, p[1]])
  const n = await analysis.saveCatalog(items)
  catalogMsg.value = n == null ? analysis.error : `${n}개 과목의 구분을 저장했습니다.`
}
const totals = computed(() => files.value.reduce((a, f) => { if (f.result) { a.inserted += f.result.inserted; a.updated += f.result.updated; a.skipped += f.result.skipped } return a }, { inserted: 0, updated: 0, skipped: 0 }))
</script>

<template>
  <div class="section">
    <div class="section-header compact">
      <div>
        <h2 class="section-title">나이스 교과학습발달상황 가져오기</h2>
        <div class="section-desc"><p>나이스에서 반별로 내려받은 「학교생활기록부 교과학습발달상황」 엑셀을 <b>여러 개 한꺼번에</b> 넣으세요. 반·학년·학기·과목·학점·원점수/평균·성취도·분포비율·석차등급·수강자수를 자동으로 읽습니다.</p><p>이전 학년 기록(2학년 파일의 1학년 성적)도 함께 들어가며 학년도는 자동 계산됩니다.</p></div>
      </div>
      <div class="header-actions">
        <div class="filter-field"><span class="filter-label">현재 학년도</span><input v-model="gradeYear" type="number" class="ui-input ui-input--sm" style="width:100px" /></div>
        <label class="checkbox-row"><input v-model="createMissing" type="checkbox" /> 명단에 없는 학생 자동 생성</label>
        <input ref="fileInput" type="file" accept=".xlsx" multiple style="display:none" @change="onFiles" />
        <button class="btn-add" :disabled="busy" @click="fileInput.click()"><FileSpreadsheet :size="16" /> 파일 선택 (여러 개)</button>
      </div>
    </div>
    <div class="section-body">
      <div v-if="!files.length" class="placeholder" style="min-height:120px">
        <span class="placeholder-title">download-*.xlsx 파일을 반 수만큼 선택하세요</span>
        <span>파일 상단의 "N학년 M반"으로 반을 인식합니다. 같은 파일을 다시 넣으면 갱신됩니다.</span>
      </div>
      <template v-else>
        <div class="table-wrap"><table class="data-table"><thead><tr><th>파일</th><th>반</th><th class="n">학생</th><th class="n">성적 행</th><th>상태</th></tr></thead><tbody>
          <tr v-for="f in files" :key="f.name">
            <td class="mono">{{ f.name }}</td>
            <td>{{ f.parsed ? `${f.parsed.grade}학년 ${f.parsed.class_no}반` : '—' }}</td>
            <td class="n">{{ f.parsed?.students ?? '' }}</td><td class="n">{{ f.parsed?.rows.length ?? '' }}</td>
            <td>
              <span v-if="f.error" class="badge badge--red" :title="f.error">오류: {{ f.error.slice(0, 60) }}</span>
              <span v-else-if="f.result" class="badge badge--green"><CheckCircle2 :size="12" /> 신규 {{ f.result.inserted }} · 갱신 {{ f.result.updated }}{{ f.result.skipped ? ` · 건너뜀 ${f.result.skipped}` : '' }}</span>
              <span v-else class="badge badge--muted">읽음 · 대기</span>
              <span v-if="f.parsed?.warnings?.length" class="badge badge--warn" :title="f.parsed.warnings.join('\n')">경고 {{ f.parsed.warnings.length }}</span>
              <span v-if="f.result?.unmatched?.length" class="badge badge--warn" :title="f.result.unmatched.join('\n')">명단 없음 {{ f.result.unmatched.length }}명</span>
            </td>
          </tr>
        </tbody></table></div>
        <div class="row">
          <div v-if="totals.inserted || totals.updated" class="note note--accent grow"><CheckCircle2 :size="14" /><div>신규 {{ totals.inserted }} · 갱신 {{ totals.updated }} · 건너뜀 {{ totals.skipped }}. <span class="link" @click="emit('navigate', 'class')">학급 한눈에</span>에서 확인하세요.</div></div>
          <span v-else class="grow" />
          <button class="btn-ghost" @click="files = []"><Trash2 :size="14" /> 목록 비우기</button>
          <button class="btn-add" :disabled="!ready.length || busy" @click="importAll">{{ busy ? '처리 중…' : `${ready.length}개 파일 가져오기` }}</button>
        </div>
      </template>

      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title">과목 편제 (공통 / 일반 / 진로 / 융합)</h3><p class="panel-desc">학교 교육과정 편제표 기준. 경희대(진로·융합) 산식과 과목 표의 구분에 쓰입니다. 한 줄에 "과목명 [탭] 구분".</p></div><button class="btn-soft" @click="saveCatalog">편제 저장</button></div>
        <textarea v-model="catalogText" class="cat" rows="8" placeholder="공통국어1	공통&#10;문학	일반&#10;사회문제 탐구	융합" />
        <p v-if="catalogMsg" class="msg-success">{{ catalogMsg }}</p>
      </div>
      <div class="note"><AlertTriangle :size="14" /><div>나이스 출력물에는 표준편차와 석차가 없습니다. 등급 안 세부 위치가 필요하면 파일 가져오기(매핑)의 담임용 양식으로 보완하세요.</div></div>
    </div>
  </div>
</template>

<style scoped>
.compact { padding: 18px 28px 12px; align-items: center; }
.section-body { padding: 16px 28px 40px; }
.n { text-align: right; }
.cat { width: 100%; padding: 10px 12px; border: 1px solid var(--bd-1); border-radius: 10px; background: var(--bg-2); color: var(--tx-1); font-family: ui-monospace, Consolas, monospace; font-size: 12.5px; }
</style>
