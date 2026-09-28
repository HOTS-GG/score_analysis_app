<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { Check, ChevronDown, ChevronUp, Moon, RotateCcw, Save, Sun } from 'lucide-vue-next'
import { useConfigStore } from '../stores/configStore'
import { useThemeStore } from '../stores/theme'
import { useProjectStore } from '../stores/project'
import ConfirmModal from '../components/ConfirmModal.vue'
import SliderField from '../components/SliderField.vue'

const config = useConfigStore()
const theme = useThemeStore()
const project = useProjectStore()

const form = ref(null)
const saving = ref(false)
const saved = ref(false)
const error = ref('')
const confirmReset = ref(false)
const showAdvanced = ref(false)

/** 기본값 (Rust Thresholds::default 와 같음) */
const DEF = {
  pos_delta_warn: 10, e_count_warn: 3, entry_move_warn: 20, area_gap_warn: 1.2, consistency_abs_min: 20,
  sharp_drop_delta: -15, consecutive_drop_delta: -7, subject_gap_min: 30, volatility_min: 15, cutline_margin_max: 3,
  adaptation_entry_top_pct: 30, adaptation_sept_bottom_pct: 50,
  easy_item_national_rate: 70, hard_item_national_rate: 30, careless_easy_item_share: 40, item_gap_report: -15, misconception_choice_share: 40,
  sincerity_blank_ratio: 20, sincerity_same_choice_run: 5,
  min_n_warn: 30, min_n_regression: 100, min_r_regression: 0.4, min_n_equipercentile: 200, transition_cell_min: 10, skew_warn: 0.5, pct_clip_low: 0.5, pct_clip_high: 99.5, top_tier_pct: 96,
  z_resid_growth: 1, z_resid_growth_count: 2, z_resid_quiet_drop: -1,
}

onMounted(() => config.loadThresholds())
watch(() => config.thresholds, (t) => { if (t) form.value = JSON.parse(JSON.stringify(t)) }, { immediate: true })
const dirty = computed(() => form.value && config.thresholds && JSON.stringify(form.value) !== JSON.stringify(config.thresholds))

/** 그룹 → 칸 정의. [key, label, desc, example, min, max, step, unit] */
const groups = [
  {
    title: '학급 화면에서 "눈여겨볼 점" 태그를 붙이는 기준',
    desc: '학급 한눈에 표와 "먼저 만나 볼 학생"에 나타나는 표시입니다. 기준을 낮추면 더 많은 학생이 걸리고, 높이면 정말 두드러진 학생만 남습니다.',
    fields: [
      ['pos_delta_warn', '직전 학기보다 교내 위치가 이만큼 내려가면 표시', '교내 위치는 학년 안에서 상위 몇 %인지입니다. 지난 학기보다 이 숫자(p)만큼 아래로 내려간 학생에게 "하락" 태그가 붙고, 같은 크기로 올라가면 "상승" 태그가 붙습니다.', '10이면 상위 30% → 상위 42%로 내려간 학생이 표시됩니다.', 3, 30, 1, 'p'],
      ['e_count_warn', '성취도 E 과목이 이 개수 이상이면 표시', '이번 학기(또는 누적) 성취도 E를 받은 과목 수입니다.', '3이면 E가 세 과목 이상인 학생만.', 1, 8, 1, '과목'],
      ['area_gap_warn', '한 교과만 유독 낮을 때 (등급 차이)', '가장 약한 교과의 등급평균이 주요교과 전체 평균보다 이만큼 높으면(=나쁘면) "○○만 유독 낮음"이 붙습니다.', '1.2이면 전체 2.5등급인데 수학만 3.7등급 이상일 때.', 0.5, 3, 0.1, '등급'],
      ['consistency_abs_min', '내신과 모의고사 위치가 이만큼 어긋나면 표시', '교내 내신 위치(상위 %)와 최근 모의고사 전국 위치(상위 %)의 차이입니다. 내신이 훨씬 좋으면 "모의 약함", 모의고사가 훨씬 좋으면 "내신 약함"이 붙습니다.', '20이면 내신 상위 20%인데 모의고사 상위 45%면 표시.', 5, 40, 1, 'p'],
      ['entry_move_warn', '입학 때보다 이만큼 움직이면 표시', '고입내신·반배치고사로 잡은 입학 당시 위치와 지금 교내 위치의 차이입니다. 내려가면 주의, 올라가면 칭찬 태그.', '20이면 입학 상위 20% → 지금 상위 45%인 학생이 표시.', 5, 40, 1, 'p'],
    ],
  },
  {
    title: '모의고사 추이로 걸러내는 기준 (학년·학교 → 스크리닝)',
    desc: '전국 백분위(0~100)의 회차 간 변화로 판단합니다. 등급끼리는 비교하지 않습니다.',
    fields: [
      ['sharp_drop_delta', '급락: 한 회차 사이에 백분위가 이만큼 떨어지면', '3월→6월 또는 6월→9월 어느 한 구간이라도 전국 백분위가 이 값 이하로 떨어진 학생.', '−15면 백분위 70 → 55.', -40, -3, 1, '백분위'],
      ['consecutive_drop_delta', '연속 하락: 두 구간 연달아 이만큼씩 떨어지면', '3→6월과 6→9월 두 구간 모두 이 값 이하로 떨어진 학생. 급락보다 작아도 계속 내려가는 학생을 잡습니다.', '−7이면 70 → 62 → 54.', -30, -2, 1, '백분위'],
      ['subject_gap_min', '과목 불균형: 잘하는 과목과 못하는 과목 차이', '한 회차 안에서 과목 백분위 최고와 최저의 차이.', '30이면 국어 85, 수학 50인 학생.', 10, 60, 1, '백분위'],
      ['volatility_min', '기복: 회차마다 오르내리는 폭', '3·6·9월 백분위의 표준편차. 클수록 시험마다 들쭉날쭉합니다.', '15면 60 → 85 → 55처럼 흔들리는 학생.', 5, 30, 1, '백분위'],
      ['cutline_margin_max', '경계선: 다음 등급컷까지 이만큼 남았으면', '조금만 오르면 등급이 바뀌는 학생. 상담에서 "한 문제 차이"를 말할 때 씁니다.', '3이면 2등급 컷까지 백분위 3 이내.', 1, 10, 0.5, '백분위'],
      ['adaptation_entry_top_pct', '적응 실패 ①: 반배치고사 상위 몇 % 안에 들었던 학생 중', '입학 때는 이 범위 안에 있었는데', '30이면 반배치 상위 30% 이내였던 학생.', 5, 50, 5, '%'],
      ['adaptation_sept_bottom_pct', '적응 실패 ②: 9월 교내 위치가 이보다 아래로 떨어진 학생', '…9월에 이 위치보다 아래라면 "적응 실패"로 표시합니다.', '50이면 9월 교내 하위 50%.', 30, 90, 5, '%'],
    ],
  },
  {
    title: '문항 분석 기준 (문항 자료가 있을 때만)',
    desc: '전국 정답률로 문항을 쉬움·중간·어려움으로 나누고, 어디서 점수를 잃는지 봅니다.',
    fields: [
      ['easy_item_national_rate', '쉬운 문항: 전국 정답률이 이 이상', '대부분이 맞히는 문항. 여기서 틀리면 "실수"일 가능성이 큽니다.', '70이면 전국 70% 이상이 맞힌 문항.', 50, 95, 5, '%'],
      ['hard_item_national_rate', '어려운 문항: 전국 정답률이 이 이하', '여기서 맞히면 실력이 있다는 신호.', '30이면 전국 30% 이하만 맞힌 문항.', 5, 50, 5, '%'],
      ['careless_easy_item_share', '실수형: 잃은 점수 중 쉬운 문항 비중이 이 이상이면', '총 실점 가운데 쉬운 문항에서 잃은 비율. 높으면 "실수·시간관리형"으로 분류.', '40이면 잃은 점수의 40% 이상이 쉬운 문항.', 20, 80, 5, '%'],
      ['item_gap_report', '우리 학교가 전국보다 이만큼 못 푼 문항을 보고', '교내 정답률 − 전국 정답률이 이 값 이하인 문항·단원을 골라 보여 줍니다.', '−15면 전국 60%인데 우리 학교 45% 이하.', -40, -5, 1, '%p'],
      ['misconception_choice_share', '공통 오개념: 오답이 한 선지에 이만큼 몰리면', '틀린 학생들이 같은 오답을 고른 비율.', '40이면 오답의 40%가 ③번에 몰린 문항.', 20, 80, 5, '%'],
    ],
  },
  {
    title: '성실하게 응시했는지 판정',
    desc: '찍었거나 포기한 답안지를 학교 통계에서 따로 떼어 보기 위한 기준입니다. 학교 통계는 [전체]와 [성실 응시자만] 두 버전으로 나옵니다.',
    fields: [
      ['sincerity_blank_ratio', '무응답 문항이 이 비율 이상이면 의심', '', '20이면 45문항 중 9문항 이상 비움.', 5, 50, 5, '%'],
      ['sincerity_same_choice_run', '뒷부분에서 같은 번호를 이만큼 연속 찍으면 의심', '', '5면 ③③③③③.', 3, 15, 1, '문항'],
    ],
  },
]
const advanced = [
  ['min_n_warn', '인원이 이보다 적으면 경고 표시', '통계는 사람이 적을수록 우연에 흔들립니다. 이 수 미만이면 화면에 "N 부족" 경고가 붙습니다.', '', 10, 100, 5, '명'],
  ['min_n_regression', '입학 대비 성장 분석(회귀)을 하려면 최소 이만큼', '학년 인원이 이보다 적으면 부가가치(잔차) 계산을 하지 않습니다.', '', 30, 300, 10, '명'],
  ['min_r_regression', '입학 성적과 모의고사의 상관이 이보다 약하면 보류', '상관계수 r. 약하면 회귀 예측이 의미 없어 계산을 멈춥니다.', '', 0.1, 0.9, 0.05, 'r'],
  ['min_n_equipercentile', '환율표를 구간 방식으로 만들 최소 인원', '이 인원 이상이면 실제 분포(등백분위)로, 미만이면 정규분포 가정으로 환율표를 만듭니다.', '', 50, 400, 10, '명'],
  ['transition_cell_min', '종단 전이표에서 인원이 이보다 적은 칸은 숨김', '몇 명 안 되는 칸의 비율은 우연이라 "표본 부족"으로 표시합니다.', '', 3, 30, 1, '명'],
  ['skew_warn', '분포가 이만큼 치우치면 정규 가정 경고', '왜도(skew)의 절댓값.', '', 0.2, 1.5, 0.1, ''],
  ['pct_clip_low', '백분위 0은 이 값으로', '통계 변환(z) 오류를 막기 위한 하한.', '', 0.1, 2, 0.1, ''],
  ['pct_clip_high', '백분위 100은 이 값으로', '상한.', '', 98, 99.9, 0.1, ''],
  ['top_tier_pct', '최상위권 표준점수 추적 기준', '백분위가 이 이상인 학생은 천장 효과를 피하려고 표준점수로 따로 봅니다.', '', 90, 99, 1, '백분위'],
  ['z_resid_growth', '저평가 성장: 잔차가 이 이상이면', '입학 성적으로 예상한 것보다 이만큼(표준편차 단위) 잘하는 학생.', '', 0.5, 2, 0.1, 'z'],
  ['z_resid_growth_count', '…그런 회차가 이 개수 이상일 때', '한 번은 우연일 수 있어 여러 회차 반복을 요구합니다.', '', 1, 3, 1, '회'],
  ['z_resid_quiet_drop', '조용한 하락: 잔차가 이 이하인데 등급은 그대로면', '등급은 안 바뀌어 눈에 띄지 않지만 예상보다 밀리는 학생. 가장 놓치기 쉬운 경우입니다.', '', -2, -0.5, 0.1, 'z'],
]

function setList(key, text) { form.value[key] = text.split(',').map(x => x.trim()).filter(Boolean) }
function setIntList(key, text) { form.value[key] = text.split(',').map(x => Number(x.trim())).filter(x => !isNaN(x)) }
function setCuts(key, text) { form.value[key] = text.split(',').map(x => Number(x.trim())).filter(x => !isNaN(x)) }

async function save() {
  saving.value = true; error.value = ''; saved.value = false
  try {
    const t = JSON.parse(JSON.stringify(form.value))
    for (const k of Object.keys(DEF)) t[k] = Number(t[k])
    await config.saveThresholds(t)
    saved.value = true
    setTimeout(() => (saved.value = false), 2000)
  } catch (e) { error.value = String(e) } finally { saving.value = false }
}
async function reset() { confirmReset.value = false; error.value = ''; try { await config.resetThresholds() } catch (e) { error.value = String(e) } }
const palette = ['#0e6e68', '#007aff', '#3b5bdb', '#9333ea', '#d97706', '#16a34a', '#dc2626', '#0891b2']
</script>

<template>
  <div class="section">
    <div class="section-header compact">
      <div>
        <h2 class="section-title">설정 · 판단 기준</h2>
        <div class="section-desc"><p>프로그램이 "이 학생을 눈여겨보라"고 표시하는 기준입니다. 슬라이더를 움직이거나 숫자를 적으면 되고, 각 칸 아래에 무슨 뜻인지와 예시를 적어 두었습니다. 바꾼 값은 이 파일에 저장되어 화면에 바로 반영됩니다.</p></div>
      </div>
      <div class="header-actions">
        <button class="btn-ghost" @click="confirmReset = true"><RotateCcw :size="14" /> 전부 기본값으로</button>
        <button class="btn-add" :disabled="!dirty || saving" @click="save"><component :is="saved ? Check : Save" :size="16" /> {{ saved ? '저장됨' : saving ? '저장 중…' : '저장' }}</button>
      </div>
    </div>

    <div class="section-body">
      <p v-if="error" class="msg-error">{{ error }}</p>
      <p v-if="config.error" class="msg-error">{{ config.error }}</p>
      <div v-if="dirty" class="note note--accent">바뀐 값이 있습니다. 오른쪽 위 <b>저장</b>을 눌러야 적용됩니다.</div>

      <template v-if="form">
        <div v-for="g in groups" :key="g.title" class="panel">
          <div class="panel-head"><div><h3 class="panel-title">{{ g.title }}</h3><p class="panel-desc">{{ g.desc }}</p></div></div>
          <div class="fields">
            <SliderField v-for="[key, label, desc, ex, min, max, step, unit] in g.fields" :key="key" v-model="form[key]" :label="label" :desc="desc" :example="ex" :min="min" :max="max" :step="step" :unit="unit" :def="DEF[key]" />
          </div>
        </div>

        <div class="panel">
          <div class="panel-head"><div><h3 class="panel-title">집단 착시 (성취도와 등급이 어긋나는 과목)</h3><p class="panel-desc">성취도는 절대 기준(몇 점 이상), 등급은 상대 기준(반에서 몇 %)입니다. 성취도는 낮은데 등급이 높으면 그 과목의 수강 집단이 약한 것이라 전국 기준으로는 위험 신호입니다.</p></div></div>
          <div class="form-grid">
            <div class="form-field"><label class="form-label">성취도가 이 중 하나이면서</label><input :value="form.group_illusion_achievements.join(', ')" class="ui-input ui-input--sm" @change="setList('group_illusion_achievements', $event.target.value)" /><span class="form-hint">쉼표로 구분. 기본 C, D, E</span></div>
            <div class="form-field"><label class="form-label">내신 등급이 이 중 하나이면 표시</label><input :value="form.group_illusion_grades.join(', ')" class="ui-input ui-input--sm" @change="setIntList('group_illusion_grades', $event.target.value)" /><span class="form-hint">기본 1, 2 → 성취도 C인데 1등급이면 "집단 착시"</span></div>
          </div>
        </div>

        <div class="panel">
          <button class="toggle" @click="showAdvanced = !showAdvanced"><component :is="showAdvanced ? ChevronUp : ChevronDown" :size="16" /> 고급 · 통계 안전장치와 등급컷 <span class="muted">(보통은 손댈 필요 없습니다)</span></button>
          <template v-if="showAdvanced">
            <div class="fields" style="margin-top:12px">
              <SliderField v-for="[key, label, desc, ex, min, max, step, unit] in advanced" :key="key" v-model="form[key]" :label="label" :desc="desc" :example="ex" :min="min" :max="max" :step="step" :unit="unit" :def="DEF[key]" />
            </div>
            <div class="form-grid" style="margin-top:12px">
              <div class="form-field"><label class="form-label">5등급제 누적 % (내신, 2025학년도 입학생부터)</label><input :value="form.grade5_cuts.join(', ')" class="ui-input ui-input--sm" @change="setCuts('grade5_cuts', $event.target.value)" /><span class="form-hint">기본 10, 34, 66, 90, 100 — 상위 10%까지 1등급, 34%까지 2등급…</span></div>
              <div class="form-field"><label class="form-label">9등급제 누적 % (학력평가·수능)</label><input :value="form.grade9_cuts.join(', ')" class="ui-input ui-input--sm" @change="setCuts('grade9_cuts', $event.target.value)" /><span class="form-hint">기본 4, 11, 23, 40, 60, 77, 89, 96, 100</span></div>
            </div>
          </template>
        </div>
      </template>

      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title">화면</h3><p class="panel-desc">이 컴퓨터의 앱 설정입니다. 파일과 무관하게 유지됩니다.</p></div></div>
        <div class="row">
          <div class="seg">
            <button :class="{ on: !theme.isDark }" @click="theme.setMode(false)"><Sun :size="14" style="display:inline; vertical-align:-2px" /> 라이트</button>
            <button :class="{ on: theme.isDark }" @click="theme.setMode(true)"><Moon :size="14" style="display:inline; vertical-align:-2px" /> 다크</button>
          </div>
          <div class="swatches"><button v-for="c in palette" :key="c" class="swatch" :class="{ on: theme.accentHex === c }" :style="{ background: c }" @click="theme.setAccentColor(c)" /></div>
        </div>
      </div>

      <div class="panel">
        <div class="panel-head"><div><h3 class="panel-title">파일</h3></div></div>
        <div class="file-info"><span class="mono">{{ project.filePath }}</span><span class="form-hint">열 때마다 같은 폴더에 .db.backup(텍스트)이 갱신됩니다. 이 폴더는 로컬이므로 드라이브 등에 따로 복사해 두세요.</span></div>
      </div>
    </div>

    <ConfirmModal v-if="confirmReset" title="기준값 초기화" danger confirm-label="초기화" message="모든 판단 기준을 기본값으로 되돌립니다." @confirm="reset" @cancel="confirmReset = false" />
  </div>
</template>

<style scoped>
.compact { padding: 18px 28px 12px; }
.section-body { padding: 16px 28px 40px; }
.fields { display: grid; grid-template-columns: repeat(auto-fill, minmax(360px, 1fr)); gap: 12px; }
.toggle { display: flex; align-items: center; gap: 8px; border: none; background: none; color: var(--tx-1); font-size: 14px; font-weight: 700; cursor: pointer; padding: 0; }
.muted { color: var(--tx-4); font-weight: 400; font-size: 12.5px; }
.swatches { display: flex; gap: 8px; }
.swatch { width: 26px; height: 26px; border-radius: 50%; border: 2px solid transparent; cursor: pointer; }
.swatch.on { border-color: var(--tx-1); box-shadow: 0 0 0 2px var(--bg-1); }
.file-info { display: flex; flex-direction: column; gap: 6px; }
</style>
