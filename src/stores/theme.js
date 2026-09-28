import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '../lib/ipc'

export function hexToRgb(hex) {
  const h = hex.replace('#', '')
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)]
}

function toHex(r, g, b) {
  const c = (v) => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, '0')
  return `#${c(r)}${c(g)}${c(b)}`
}
const lighten = (r, g, b, k) => toHex(r + (255 - r) * k, g + (255 - g) * k, b + (255 - b) * k)
const darken = (r, g, b, k) => toHex(r * (1 - k), g * (1 - k), b * (1 - k))
const brightness = (r, g, b) => (r * 299 + g * 587 + b * 114) / 1000

export function computeAccentVars(hex, isDark) {
  let [r, g, b] = hexToRgb(hex)
  if (isDark) {
    if (brightness(r, g, b) < 90) [r, g, b] = hexToRgb(lighten(r, g, b, 0.45))
    return {
      '--accent-rgb': `${r}, ${g}, ${b}`,
      '--accent-hex': toHex(r, g, b),
      '--accent-hex-hover': lighten(r, g, b, 0.18),
      '--accent-text': lighten(r, g, b, 0.38),
      '--accent-bright': lighten(r, g, b, 0.58),
    }
  }
  if (brightness(r, g, b) > 170) [r, g, b] = hexToRgb(darken(r, g, b, 0.35))
  const solid = toHex(r, g, b)
  return {
    '--accent-rgb': `${r}, ${g}, ${b}`,
    '--accent-hex': solid,
    '--accent-hex-hover': darken(r, g, b, 0.12),
    '--accent-text': solid,
    '--accent-bright': solid,
  }
}

export const DEFAULT_ACCENT = '#0e6e68'

export const useThemeStore = defineStore('theme', () => {
  const isDark = ref(false)
  const accentHex = ref(DEFAULT_ACCENT)

  function applyAll(dark, hex) {
    const root = document.documentElement
    root.setAttribute('data-mode', dark ? 'dark' : 'light')
    for (const [k, v] of Object.entries(computeAccentVars(hex, dark))) root.style.setProperty(k, v)
  }

  async function loadAndApply() {
    try {
      const [savedMode, savedColor] = await Promise.all([
        invoke('get_global_config', { key: 'theme_mode' }),
        invoke('get_global_config', { key: 'theme_color' }),
      ])
      isDark.value = savedMode === 'dark'
      accentHex.value = savedColor || DEFAULT_ACCENT
    } catch { /* 기본값 */ }
    applyAll(isDark.value, accentHex.value)
  }

  async function setMode(dark) {
    isDark.value = dark
    applyAll(dark, accentHex.value)
    try { await invoke('set_global_config', { key: 'theme_mode', value: dark ? 'dark' : 'light' }) } catch { /* ignore */ }
  }

  async function setAccentColor(hex) {
    accentHex.value = hex
    applyAll(isDark.value, hex)
    try { await invoke('set_global_config', { key: 'theme_color', value: hex }) } catch { /* ignore */ }
  }

  function resetToDefault() {
    isDark.value = false
    accentHex.value = DEFAULT_ACCENT
    applyAll(false, DEFAULT_ACCENT)
  }

  return { isDark, accentHex, loadAndApply, setMode, setAccentColor, resetToDefault }
})
