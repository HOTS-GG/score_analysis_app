<script setup>
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import schoolLogo from '../assets/school_logo.svg'
import { isTauri } from '../lib/ipc'
import { useProjectStore } from '../stores/project'
import { useThemeStore } from '../stores/theme'

const router = useRouter()
const project = useProjectStore()
const themeStore = useThemeStore()
const error = ref('')
const currentVersion = ref('0.1.0')
const showAbout = ref(false)

onMounted(async () => {
  await themeStore.loadAndApply()
  if (!isTauri) return
  try {
    const { getVersion } = await import('@tauri-apps/api/app')
    currentVersion.value = await getVersion()
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const { LogicalSize } = await import('@tauri-apps/api/dpi')
    const win = getCurrentWindow()
    await win.setSize(new LogicalSize(1280, 720))
    await win.center()
  } catch { /* 브라우저 미리보기 */ }
})

async function pickSavePath() {
  if (!isTauri) return 'C:/preview/성적분석.db'
  const { save } = await import('@tauri-apps/plugin-dialog')
  return await save({
    title: '새 성적분석 파일 위치 선택',
    defaultPath: 'score_analysis.db',
    filters: [{ name: '성적분석 파일', extensions: ['db'] }],
  })
}

async function pickOpenPath() {
  if (!isTauri) return 'C:/preview/성적분석.db'
  const { open } = await import('@tauri-apps/plugin-dialog')
  return await open({
    title: '기존 성적분석 파일 선택',
    filters: [{ name: '성적분석 파일', extensions: ['db'] }],
    multiple: false,
  })
}

async function handleNew() {
  error.value = ''
  const path = await pickSavePath()
  if (!path) return
  try {
    await project.newProject(path)
    router.push('/workspace')
  } catch (e) {
    error.value = String(e)
  }
}

async function handleOpen() {
  error.value = ''
  const path = await pickOpenPath()
  if (!path) return
  try {
    await project.openProject(path)
    await project.backupProject()
    await project.migrateSchema()
    await project.checkAndUpdateVersion()
    router.push('/workspace')
  } catch (e) {
    error.value = String(e)
    project.closeProject()
  }
}
</script>

<template>
  <div class="page">
    <div class="bg-image" />
    <div class="bg-dim" />

    <div class="card">
      <div class="logo-wrap">
        <div class="logo-icon">
          <img :src="schoolLogo" alt="학교 로고" class="logo-img" />
        </div>
        <div class="logo-text">
          <h1>춘천고등학교 성적분석ON</h1>
          <p>모의고사·내신·입학 데이터를 한 파일에 모아 학생별 성장을 읽는 교사용 도구</p>
        </div>
      </div>

      <div class="divider">
        <div class="divider-line" />
        <span class="divider-dot" />
        <div class="divider-line" />
      </div>

      <div class="actions">
        <button class="btn-primary" @click="handleNew">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 4v16m8-8H4" /></svg>
          새 성적분석 파일 만들기
          <svg class="arrow" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 5l7 7-7 7" /></svg>
        </button>
        <button class="btn-secondary" @click="handleOpen">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" /></svg>
          기존 파일 열기
          <svg class="arrow" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 5l7 7-7 7" /></svg>
        </button>
      </div>

      <transition name="err">
        <div v-if="error" class="error-box">{{ error }}</div>
      </transition>

      <p class="version">
        v{{ currentVersion }} ·
        <button class="license-btn" @click="showAbout = true">이 프로그램에 대하여</button>
        <span v-if="!isTauri" class="preview-tag">브라우저 미리보기 (mock 데이터)</span>
      </p>
    </div>

    <transition name="modal">
      <div v-if="showAbout" class="overlay" @click.self="showAbout = false">
        <div class="modal">
          <div class="modal-header">
            <div>
              <h2>성적분석ON</h2>
              <p>Educational Use Only</p>
            </div>
            <button class="close-btn" @click="showAbout = false">✕</button>
          </div>
          <div class="about-body">
            <h3>무엇을 하는 프로그램인가</h3>
            <p>학력평가(전국 준거)와 내신·반배치(교내 준거)를 한 파일에 모아, 학생 개인의 전국 위치 변화와 교내 수행을 별개 축으로 읽습니다. 담임이 누구를 먼저 볼지 정하는 스크리닝 명단과 학생 상담용 카드를 만듭니다.</p>
            <h3>지키는 원칙</h3>
            <ul>
              <li>등급끼리 빼지 않습니다. 모든 비교는 백분위·석차백분율로 합니다.</li>
              <li>성장은 모의고사 전국 백분위로만 잽니다. 내신 등급 변화는 성장이 아닙니다.</li>
              <li>잔차·회귀 예측값은 교사용입니다. 학생 카드에는 나오지 않습니다.</li>
              <li>교사·학급 평가 목적의 집계 기능은 만들지 않습니다.</li>
            </ul>
            <h3>데이터</h3>
            <p>모든 데이터는 선택한 .db 파일 하나에만 저장되며 외부로 전송되지 않습니다. 실명은 학생 명단에만 있고 분석 테이블은 익명 ID로 참조합니다.</p>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped>
.page { position: relative; display: flex; align-items: center; justify-content: center; min-height: 100vh; overflow: hidden; background-color: var(--bg-0); }
.bg-image { position: absolute; inset: -20px; background-image: url('/background.webp'); background-size: cover; background-position: center; filter: blur(10px); pointer-events: none; }
.bg-dim { position: absolute; inset: 0; background: rgba(0, 0, 0, 0.45); pointer-events: none; }
.card { position: relative; z-index: 1; width: 100%; max-width: 460px; background-color: var(--bg-2); border: 1px solid var(--bd-1); border-radius: 20px; padding: 40px 36px 32px; box-shadow: 0 24px 80px rgba(0, 0, 0, 0.6), 0 0 0 1px rgba(255, 255, 255, 0.03); }
.logo-wrap { display: flex; flex-direction: column; align-items: center; gap: 18px; margin-bottom: 30px; }
.logo-icon { width: 96px; height: 96px; display: flex; align-items: center; justify-content: center; filter: drop-shadow(0 4px 16px rgba(13, 148, 136, 0.35)); }
.logo-img { width: 96px; height: 96px; object-fit: contain; }
.logo-text { text-align: center; }
.logo-text h1 { font-size: 21px; font-weight: 700; color: var(--tx-1); letter-spacing: -0.02em; margin: 0; }
.logo-text p { font-size: 14px; color: var(--clr-text-hint); margin: 6px 0 0; line-height: 1.5; }
.divider { display: flex; align-items: center; margin-bottom: 26px; }
.divider-line { flex: 1; height: 1px; background-color: var(--bd-1); }
.divider-dot { width: 6px; height: 6px; border-radius: 50%; background-color: var(--clr-warn-text); opacity: 0.5; margin: 0 12px; }
.actions { display: flex; flex-direction: column; gap: 12px; }
.btn-primary, .btn-secondary { display: flex; align-items: center; gap: 12px; width: 100%; padding: 15px 20px; border-radius: 14px; font-size: 17px; font-weight: 500; cursor: pointer; border: 1px solid transparent; transition: background-color 0.15s, transform 0.1s; text-align: left; }
.btn-primary:active, .btn-secondary:active { transform: scale(0.98); }
.btn-primary { background-color: var(--accent-hex); color: #fff; }
.btn-primary:hover { background-color: var(--accent-hex-hover); }
.btn-secondary { background-color: var(--bg-hover); border-color: var(--bd-2); color: var(--tx-3); }
.arrow { margin-left: auto; opacity: 0; transition: opacity 0.15s, transform 0.15s; }
.btn-primary:hover .arrow, .btn-secondary:hover .arrow { opacity: 1; transform: translateX(2px); }
.error-box { margin-top: 18px; padding: 14px 18px; border-radius: 12px; background-color: var(--clr-red-deep-bg); border: 1px solid var(--clr-red-deep-border); font-size: 14px; color: var(--clr-red-text-light); line-height: 1.5; }
.err-enter-from, .err-leave-to { opacity: 0; transform: translateY(4px); }
.err-enter-active, .err-leave-active { transition: all 0.2s; }
.version { margin-top: 24px; text-align: center; font-size: 13px; color: var(--clr-text-hint); }
.license-btn { background: none; border: none; padding: 0; font-size: inherit; color: inherit; cursor: pointer; text-decoration: underline; font-family: inherit; }
.preview-tag { display: block; margin-top: 6px; color: var(--clr-warn-text); font-size: 12px; }
.overlay { position: fixed; inset: 0; z-index: 50; display: flex; align-items: center; justify-content: center; background-color: rgba(4, 6, 12, 0.75); backdrop-filter: blur(6px); }
.modal { width: 100%; max-width: 560px; background-color: var(--bg-2); border: 1px solid var(--bd-1); border-radius: 24px; padding: 34px; box-shadow: 0 24px 80px rgba(0, 0, 0, 0.7); }
.modal-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 20px; }
.modal-header h2 { font-size: 19px; font-weight: 600; color: var(--tx-1); margin: 0; }
.modal-header p { font-size: 14px; color: var(--clr-text-hint); margin: 5px 0 0; }
.close-btn { background: none; border: none; cursor: pointer; color: var(--clr-text-hint); padding: 8px; border-radius: 10px; }
.about-body { font-size: 14px; color: var(--tx-3); line-height: 1.7; max-height: 420px; overflow-y: auto; }
.about-body h3 { font-size: 13px; font-weight: 600; color: var(--tx-1); margin: 16px 0 6px; }
.about-body h3:first-child { margin-top: 0; }
.about-body ul { margin: 0; padding-left: 18px; }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-active, .modal-leave-active { transition: opacity 0.2s; }
</style>
