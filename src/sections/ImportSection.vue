<script setup>
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { Workbook } from 'exceljs'
import { AlertTriangle, CheckCircle2, FileSpreadsheet, Plus, Trash2, Wand2, X } from 'lucide-vue-next'
import { useDatasetStore } from '../stores/dataset'
import { useStudentStore } from '../stores/student'
import { useReportStore } from '../stores/report'
import { buildSchoolTemplate } from '../lib/schoolTemplate'
import ConfirmModal from '../components/ConfirmModal.vue'

const emit = defineEmits(['navigate'])
const dataset = useDatasetStore()
const students = useStudentStore()
const reportStore = useReportStore()
const templateMsg = ref('')

/** 담임용 내신 입력 양식: 현재 명단의 첫 반(또는 선택된 공통값의 학년)을 기준으로 학생 행을 미리 채운다 */
async function downloadSchoolTemplate() {
  templateMsg.value = ''
  try {
    if (!students.students.length) await students.fetchStudents()
    const grade = Number(mapping.context.grade) || students.activeStudents[0]?.grade || 1
    const classNo = students.activeStudents.find(s => s.grade === grade)?.class_no || 1
    const roster = students.activeStudents.filter(s => s.grade === grade && s.class_no === classNo).map(s => ({ number: s.number, name: s.name }))
    const buf = await buildSchoolTemplate({ grade, classNo, students: roster })
    const saved = await reportStore.saveBinary(buf, `내신입력양식_${grade}학년${classNo}반.xlsx`)
    if (saved) templateMsg.value = `양식을 저장했습니다: ${saved} (반이 다르면 양식의 반·번호를 바꿔 쓰세요)`
  } catch (e) {
    templateMsg.value = String(e)
  }
}

const activeId = ref('students')
const spec = computed(() => dataset.specById[activeId.value])

// 파일 상태
const fileName = ref('')
const sheetNames = ref([])
const sheetIndex = ref(0)
const rawRows = ref([]) // 시트 전체 (배열의 배열)
const headerRow = ref(1)
const fileError = ref('')
const fileInputRef = ref(null)
let workbookRef = null

const headers = computed(() => {
  const r = rawRows.value[headerRow.value - 1] ?? []
  return r.map((h, i) => (h === null || h === undefined || h === '') ? `열${i + 1}` : String(h).trim())
})
const dataRows = computed(() => rawRows.value.slice(headerRow.value).filter(r => r.some(v => v !== null && v !== undefined && v !== '')))
const previewRows = computed(() => dataRows.value.slice(0, 5))

function objectRows() {
  const hs = headers.value
  return dataRows.value.map(r => {
    const o = {}
    hs.forEach((h, i) => { o[h] = r[i] ?? null })
    return o
  })
}

// 매핑 상태
const mapping = reactive({ keys: {}, context: {}, subjects: [], values: {}, mode: 'wide', create_missing: false })
const result = ref(null)
const importing = ref(false)
const importError = ref('')
const confirmClear = ref(false)

function resetMapping() {
  mapping.keys = {}
  mapping.context = {}
  mapping.subjects = []
  mapping.values = {}
  mapping.mode = 'wide'
  const year = new Date().getFullYear()
  for (const cf of spec.value?.context_fields ?? []) {
    if (cf.key === 'grade_year' || cf.key === 'cohort') mapping.context[cf.key] = year
    else if (cf.key === 'grade') mapping.context[cf.key] = 1
    else if (cf.key === 'round') mapping.context[cf.key] = 3
    else if (cf.key === 'semester') mapping.context[cf.key] = 1
    else mapping.context[cf.key] = ''
  }
}

async function autoMap() {
  if (!headers.value.length) return
  const m = await dataset.suggestMapping(activeId.value, headers.value)
  mapping.keys = m.keys ?? {}
  mapping.values = m.values ?? {}
  mapping.subjects = (m.subjects ?? []).map(s => ({ subject: s.subject, columns: { ...s.columns } }))
  mapping.mode = m.mode ?? 'wide'
}

function addSubject() {
  const cols = {}
  for (const sf of spec.value.subject_fields) cols[sf.key] = null
  mapping.subjects.push({ subject: '', columns: cols })
}

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

async function loadSheet(idx) {
  const ws = workbookRef.worksheets[idx]
  const rows = []
  ws.eachRow({ includeEmpty: false }, (row, rowNumber) => {
    const arr = []
    row.eachCell({ includeEmpty: true }, (cell, colNumber) => { arr[colNumber - 1] = cellValue(cell.value) })
    rows[rowNumber - 1] = arr
  })
  rawRows.value = rows.map(r => r ?? [])
  headerRow.value = 1
  await autoMap()
}

async function onFileChange(ev) {
  const file = ev.target.files?.[0]
  ev.target.value = ''
  if (!file) return
  fileError.value = ''
  result.value = null
  importError.value = ''
  try {
    const buf = await file.arrayBuffer()
    const wb = new Workbook()
    if (/\.csv$/i.test(file.name)) {
      const text = new TextDecoder('utf-8').decode(buf)
      const lines = text.split(/\r?\n/).filter(l => l.trim())
      rawRows.value = lines.map(parseCsvLine)
      sheetNames.value = ['CSV']
      workbookRef = null
      fileName.value = file.name
      headerRow.value = 1
      await autoMap()
      return
    }
    await wb.xlsx.load(buf)
    workbookRef = wb
    sheetNames.value = wb.worksheets.map(w => w.name)
    sheetIndex.value = 0
    fileName.value = file.name
    await loadSheet(0)
  } catch (e) {
    fileError.value = `파일을 읽을 수 없습니다: ${e}`
  }
}

function parseCsvLine(line) {
  const out = []
  let cur = ''
  let inQ = false
  for (let i = 0; i < line.length; i++) {
    const c = line[i]
    if (inQ) {
      if (c === '"' && line[i + 1] === '"') { cur += '"'; i++ }
      else if (c === '"') inQ = false
      else cur += c
    } else if (c === '"') inQ = true
    else if (c === ',') { out.push(cur); cur = '' }
    else cur += c
  }
  out.push(cur)
  return out.map(v => v.trim())
}

watch(sheetIndex, async (i) => { if (workbookRef) await loadSheet(i) })
watch(headerRow, autoMap)
watch(activeId, () => { resetMapping(); result.value = null; importError.value = ''; if (headers.value.length) autoMap() })

const missingRequired = computed(() => {
  if (!spec.value) return []
  const miss = []
  for (const f of spec.value.key_fields) if (f.required && !mapping.keys[f.key]) miss.push(f.label)
  for (const f of spec.value.context_fields) if (f.required && (mapping.context[f.key] === '' || mapping.context[f.key] === null || mapping.context[f.key] === undefined)) miss.push(f.label)
  if (spec.value.supports_subjects) {
    if (mapping.mode === 'wide' && !mapping.subjects.some(s => s.subject.trim())) miss.push('과목 블록')
    if (mapping.mode === 'long' && !mapping.values.subject) miss.push('과목 컬럼')
  }
  for (const f of spec.value.value_fields) if (f.required && f.key !== 'subject' && !mapping.values[f.key]) miss.push(f.label)
  return miss
})

const canImport = computed(() => dataRows.value.length > 0 && missingRequired.value.length === 0 && !importing.value)

async function doImport() {
  if (!canImport.value) return
  importing.value = true
  importError.value = ''
  result.value = null
  try {
    const payload = {
      keys: { ...mapping.keys },
      context: Object.fromEntries(Object.entries(mapping.context).map(([k, v]) => [k, typeof v === 'string' && v !== '' && !isNaN(v) ? Number(v) : v])),
      subjects: mapping.subjects.filter(s => s.subject.trim()).map(s => ({ subject: s.subject.trim(), columns: { ...s.columns } })),
      values: { ...mapping.values },
      mode: mapping.mode,
      create_missing: mapping.create_missing,
    }
    result.value = await dataset.importDataset(activeId.value, objectRows(), payload, fileName.value)
  } catch (e) {
    importError.value = String(e)
  } finally {
    importing.value = false
  }
}

async function doClear() {
  confirmClear.value = false
  try {
    await dataset.clearDataset(activeId.value)
    result.value = null
  } catch (e) {
    importError.value = String(e)
  }
}

const datasetCount = computed(() => {
  const s = dataset.summary
  if (!s) return null
  return { students: s.students_total, mock: s.mock_rows, school: s.school_rows, entry: s.entry_rows, item_stat: s.item_stat_rows, item_response: s.item_response_rows, attendance: s.attendance_rows }[activeId.value]
})

onMounted(async () => {
  await dataset.loadSpecs()
  await Promise.all([dataset.refreshSummary(), dataset.refreshLogs()])
  resetMapping()
})
</script>

<template>
  <div class="section">
    <div class="section-header">
      <div>
        <h2 class="section-title">데이터 가져오기</h2>
        <div class="section-desc">
          <p>실제 파일은 컬럼명이 제각각입니다. 파일을 올리면 표준 항목에 자동으로 짝을 맞춰 보여 주니, 확인·수정한 뒤 가져오기를 누르세요.</p>
          <p>미응시·결시는 반드시 빈칸(또는 '-', '결시')으로 두세요. 0점으로 넣으면 하위권 해석이 전부 틀립니다.</p>
        </div>
      </div>
    </div>

    <div class="section-body">
      <!-- 데이터셋 탭 -->
      <div class="tabs">
        <button v-for="s in dataset.specs" :key="s.id" class="tab" :class="{ on: activeId === s.id }" @click="activeId = s.id">
          {{ s.label }}
          <span v-if="datasetCount !== null && activeId === s.id" class="tab-count">{{ datasetCount }}</span>
        </button>
      </div>

      <div v-if="spec" class="panel">
        <div class="panel-head">
          <div>
            <h3 class="panel-title">{{ spec.label }}</h3>
            <p class="panel-desc">{{ spec.desc }}</p>
          </div>
          <div class="row">
            <button v-if="activeId === 'school'" class="btn-soft" @click="downloadSchoolTemplate"><FileSpreadsheet :size="15" /> 담임용 입력 양식 내려받기</button>
            <button class="btn-ghost" :disabled="!datasetCount" @click="confirmClear = true"><Trash2 :size="14" /> 이 데이터셋 비우기</button>
            <input ref="fileInputRef" type="file" accept=".xlsx,.xlsm,.csv" style="display:none" @change="onFileChange" />
            <button class="btn-add" @click="fileInputRef.click()"><FileSpreadsheet :size="17" /> 파일 선택 (.xlsx / .csv)</button>
          </div>
        </div>

        <p v-if="fileError" class="msg-error">{{ fileError }}</p>
        <p v-if="templateMsg" class="msg-success">{{ templateMsg }}</p>
        <div v-if="activeId === 'school' && !fileName" class="note note--accent">
          <CheckCircle2 :size="15" />
          <div>담임 입력 흐름: <b>입력 양식 내려받기</b> → 과목별로 원점수·성취도·5등급·과목평균·표준편차·수강자수·석차를 채움 → 여기서 파일 선택 → 가져오기 → <span class="link" @click="emit('navigate', 'card')">학생 카드</span>에서 학생별 개인 분석이 나옵니다. 성취도만 있는 과목은 5등급을 비우고, 석차가 없으면 비워도 됩니다.</div>
        </div>

        <!-- 1) 파일 / 시트 / 헤더 행 -->
        <div v-if="fileName" class="file-bar">
          <span class="badge badge--accent"><FileSpreadsheet :size="13" /> {{ fileName }}</span>
          <div v-if="sheetNames.length > 1" class="filter-field">
            <span class="filter-label">시트</span>
            <select v-model="sheetIndex" class="ui-select">
              <option v-for="(n, i) in sheetNames" :key="n" :value="i">{{ n }}</option>
            </select>
          </div>
          <div class="filter-field">
            <span class="filter-label">헤더(제목) 행</span>
            <input v-model.number="headerRow" type="number" min="1" :max="Math.max(1, rawRows.length)" class="ui-input ui-input--sm" style="width:90px" />
          </div>
          <span class="badge badge--muted">데이터 {{ dataRows.length }}행 · {{ headers.length }}열</span>
          <span class="grow" />
          <button class="btn-ghost" @click="autoMap"><Wand2 :size="14" /> 자동 매핑 다시</button>
        </div>

        <!-- 미리보기 -->
        <div v-if="fileName" class="table-wrap preview">
          <table class="data-table">
            <thead><tr><th v-for="h in headers" :key="h">{{ h }}</th></tr></thead>
            <tbody>
              <tr v-for="(r, i) in previewRows" :key="i">
                <td v-for="(h, j) in headers" :key="j" :class="{ muted: r[j] === null || r[j] === undefined || r[j] === '' }">{{ r[j] ?? '' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div v-else class="placeholder">
          <span class="placeholder-title">파일을 선택하면 여기에 미리보기와 컬럼 매핑이 나타납니다</span>
          <span>첫 행이 제목이 아니면 '헤더 행' 번호를 바꾸세요. 병합된 제목 줄이 있는 나이스 출력물도 가능합니다.</span>
        </div>
      </div>

      <!-- 2) 매핑 -->
      <div v-if="spec && fileName" class="panel-grid panel-grid--2">
        <div class="panel">
          <div class="panel-head"><div><h3 class="panel-title">공통값</h3><p class="panel-desc">파일 전체에 똑같이 적용되는 값입니다.</p></div></div>
          <div class="form-grid">
            <div v-for="cf in spec.context_fields" :key="cf.key" class="form-field">
              <label class="form-label">{{ cf.label }} <span v-if="cf.required" class="req">*</span></label>
              <input v-if="cf.kind === 'text'" v-model="mapping.context[cf.key]" class="ui-input ui-input--sm" :placeholder="cf.hint" />
              <input v-else v-model="mapping.context[cf.key]" type="number" class="ui-input ui-input--sm" :placeholder="cf.hint" />
              <span v-if="cf.hint" class="form-hint">{{ cf.hint }}</span>
            </div>
          </div>
        </div>

        <div v-if="spec.key_fields.length" class="panel">
          <div class="panel-head"><div><h3 class="panel-title">학생 식별 컬럼</h3><p class="panel-desc">명단의 (입학연도·반·번호)와 맞춰 학생을 찾습니다.</p></div></div>
          <div class="form-grid">
            <div v-for="kf in spec.key_fields" :key="kf.key" class="form-field">
              <label class="form-label">{{ kf.label }} <span v-if="kf.required" class="req">*</span></label>
              <select v-model="mapping.keys[kf.key]" class="ui-select">
                <option :value="null">— 없음 —</option>
                <option v-for="h in headers" :key="h" :value="h">{{ h }}</option>
              </select>
              <span v-if="kf.hint" class="form-hint">{{ kf.hint }}</span>
            </div>
          </div>
        </div>

        <div v-if="spec.supports_subjects" class="panel" style="grid-column: 1 / -1">
          <div class="panel-head">
            <div>
              <h3 class="panel-title">과목별 컬럼</h3>
              <p class="panel-desc">가로형: 과목마다 컬럼 묶음이 있는 파일. 세로형: 한 행에 과목 컬럼이 따로 있는 파일.</p>
            </div>
            <div class="row">
              <div class="seg">
                <button :class="{ on: mapping.mode === 'wide' }" @click="mapping.mode = 'wide'">가로형(과목 블록)</button>
                <button :class="{ on: mapping.mode === 'long' }" @click="mapping.mode = 'long'">세로형(과목 컬럼)</button>
              </div>
              <button v-if="mapping.mode === 'wide'" class="btn-soft" @click="addSubject"><Plus :size="14" /> 과목 추가</button>
            </div>
          </div>

          <template v-if="mapping.mode === 'wide'">
            <div v-if="!mapping.subjects.length" class="placeholder" style="min-height:90px">과목 블록이 없습니다. '과목 추가'를 누르거나 자동 매핑을 실행하세요.</div>
            <div v-for="(sb, i) in mapping.subjects" :key="i" class="subject-block">
              <div class="subject-head">
                <input v-model="sb.subject" class="ui-input ui-input--sm subject-name" placeholder="과목명 (예: 국어)" />
                <button class="btn-icon" title="블록 삭제" @click="mapping.subjects.splice(i, 1)"><X :size="15" /></button>
              </div>
              <div class="form-grid">
                <div v-for="sf in spec.subject_fields" :key="sf.key" class="form-field">
                  <label class="form-label">{{ sf.label }}</label>
                  <select v-model="sb.columns[sf.key]" class="ui-select">
                    <option :value="null">— 없음 —</option>
                    <option v-for="h in headers" :key="h" :value="h">{{ h }}</option>
                  </select>
                  <span v-if="sf.hint" class="form-hint">{{ sf.hint }}</span>
                </div>
              </div>
            </div>
          </template>
          <div v-else class="form-grid">
            <div v-for="vf in [...spec.value_fields, ...spec.subject_fields]" :key="vf.key" class="form-field">
              <label class="form-label">{{ vf.label }} <span v-if="vf.required" class="req">*</span></label>
              <select v-model="mapping.values[vf.key]" class="ui-select">
                <option :value="null">— 없음 —</option>
                <option v-for="h in headers" :key="h" :value="h">{{ h }}</option>
              </select>
            </div>
          </div>
        </div>

        <div v-if="spec.value_fields.some(v => v.key !== 'subject') && (mapping.mode === 'wide' || !spec.supports_subjects)" class="panel" style="grid-column: 1 / -1">
          <div class="panel-head"><div><h3 class="panel-title">값 컬럼</h3><p class="panel-desc">행 단위로 들어오는 값입니다.</p></div></div>
          <div class="form-grid">
            <div v-for="vf in spec.value_fields.filter(v => v.key !== 'subject')" :key="vf.key" class="form-field">
              <label class="form-label">{{ vf.label }} <span v-if="vf.required" class="req">*</span></label>
              <select v-model="mapping.values[vf.key]" class="ui-select">
                <option :value="null">— 없음 —</option>
                <option v-for="h in headers" :key="h" :value="h">{{ h }}</option>
              </select>
              <span v-if="vf.hint" class="form-hint">{{ vf.hint }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 3) 실행 -->
      <div v-if="spec && fileName" class="panel">
        <div class="row">
          <div v-if="missingRequired.length" class="note note--warn grow">
            <AlertTriangle :size="15" /> 필수 항목이 비어 있습니다: {{ missingRequired.join(', ') }}
          </div>
          <div v-else class="note note--accent grow">
            <CheckCircle2 :size="15" /> {{ dataRows.length }}행을 '{{ spec.label }}'(으)로 가져옵니다. 같은 키의 기존 행은 덮어씁니다.
          </div>
          <button class="btn-add" :disabled="!canImport" @click="doImport">{{ importing ? '가져오는 중…' : '가져오기 실행' }}</button>
        </div>
        <p v-if="importError" class="msg-error">{{ importError }}</p>
        <div v-if="result" class="result">
          <div class="kpi-row">
            <div class="kpi"><span class="kpi-label">읽은 행</span><span class="kpi-value">{{ result.row_count }}</span></div>
            <div class="kpi"><span class="kpi-label">신규</span><span class="kpi-value">{{ result.inserted }}</span></div>
            <div class="kpi"><span class="kpi-label">갱신</span><span class="kpi-value">{{ result.updated }}</span></div>
            <div class="kpi" :class="{ 'kpi--warn': result.skipped }"><span class="kpi-label">건너뜀</span><span class="kpi-value">{{ result.skipped }}</span></div>
          </div>
          <div v-if="result.unmatched.length" class="note note--warn">
            <AlertTriangle :size="15" />
            <div>명단에서 찾지 못한 학생 {{ result.unmatched.length }}명: {{ result.unmatched.slice(0, 15).join(', ') }}<span v-if="result.unmatched.length > 15"> 외 {{ result.unmatched.length - 15 }}명</span><br>
              <span class="link" @click="emit('navigate', 'student')">학생 명단</span>을 먼저 맞춘 뒤 다시 가져오세요.</div>
          </div>
          <div v-if="result.errors.length" class="note note--red">
            <AlertTriangle :size="15" />
            <div>오류 {{ result.errors.length }}건<br><span v-for="e in result.errors.slice(0, 10)" :key="e">{{ e }}<br></span></div>
          </div>
        </div>
      </div>

      <!-- 가져오기 이력 -->
      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title">가져오기 이력</h3><p class="panel-desc">파일별 결과가 기록됩니다. 같은 파일을 다시 가져오면 갱신으로 처리됩니다.</p></div></div>
        <div v-if="!dataset.logs.length" class="placeholder" style="min-height:80px">아직 가져온 파일이 없습니다.</div>
        <div v-else class="table-wrap">
          <table class="data-table">
            <thead><tr><th>일시</th><th>데이터셋</th><th>파일</th><th class="num">행</th><th class="num">신규</th><th class="num">갱신</th><th class="num">건너뜀</th></tr></thead>
            <tbody>
              <tr v-for="l in dataset.logs" :key="l.id">
                <td class="muted">{{ l.imported_at }}</td>
                <td>{{ dataset.specById[l.dataset]?.label ?? l.dataset }}</td>
                <td>{{ l.file_name }}</td>
                <td class="num">{{ l.row_count }}</td>
                <td class="num">{{ l.inserted }}</td>
                <td class="num">{{ l.updated }}</td>
                <td class="num" :class="{ warn: l.skipped }">{{ l.skipped }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <ConfirmModal
      v-if="confirmClear" title="데이터셋 비우기" danger confirm-label="비우기"
      :message="`'${spec?.label}' 데이터를 모두 삭제합니다.${activeId === 'students' ? '\n학생 명단을 비우면 연결된 모든 성적 데이터도 함께 삭제됩니다.' : ''}`"
      @confirm="doClear" @cancel="confirmClear = false"
    />
  </div>
</template>

<style scoped>
.tabs { display: flex; flex-wrap: wrap; gap: 6px; }
.tab { padding: 8px 14px; border-radius: 10px; border: 1px solid var(--bd-1); background: var(--bg-1); color: var(--tx-3); font-size: 13px; font-weight: 500; cursor: pointer; display: inline-flex; align-items: center; gap: 6px; }
.tab.on { background: rgba(var(--accent-rgb), 0.16); border-color: rgba(var(--accent-rgb), 0.4); color: var(--accent-text); font-weight: 600; }
.tab-count { font-size: 11px; padding: 1px 6px; border-radius: 999px; background: rgba(var(--accent-rgb), 0.2); }
.file-bar { display: flex; align-items: flex-end; gap: 14px; flex-wrap: wrap; }
.preview { max-height: 220px; }
.subject-block { border: 1px solid var(--bd-1); border-radius: 12px; padding: 12px 14px; display: flex; flex-direction: column; gap: 10px; background: var(--bg-2); }
.subject-head { display: flex; align-items: center; gap: 8px; }
.subject-name { width: 200px; font-weight: 600; }
.result { display: flex; flex-direction: column; gap: 10px; }
.warn { color: var(--clr-warn-text); font-weight: 600; }
</style>
