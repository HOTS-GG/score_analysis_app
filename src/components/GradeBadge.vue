<script setup>
import { computed } from 'vue'
/** 등급 배지 (1 진함 → 5 옅음). value: 숫자 등급 또는 성취도 문자 */
const props = defineProps({ value: { type: [Number, String], default: null }, digits: { type: Number, default: 2 } })
const cls = computed(() => {
  if (props.value === null || props.value === undefined || props.value === '') return 'gr-x'
  if (typeof props.value === 'string') return 'ach-' + (/^[A-E]$/.test(props.value) ? props.value : 'x')
  return 'gr-' + Math.max(1, Math.min(5, Math.round(props.value)))
})
const text = computed(() => props.value === null || props.value === undefined || props.value === '' ? '—' : typeof props.value === 'string' ? props.value : Number(props.value).toFixed(props.digits))
</script>

<template><span class="gb" :class="cls">{{ text }}</span></template>

<style scoped>
.gb { display: inline-block; min-width: 34px; padding: 2px 7px; border-radius: 6px; font-size: 12px; font-weight: 700; text-align: center; font-variant-numeric: tabular-nums; color: #fff; }
.gr-1 { background: #0b4f4b; } .gr-2 { background: #14776f; } .gr-3 { background: #4e9c93; } .gr-4 { background: #94bfb8; color: #123; } .gr-5 { background: #cfdedb; color: #123; }
.ach-A { background: #0f6d68; } .ach-B { background: #57988f; } .ach-C { background: #aeb7b9; color: #123; } .ach-D { background: #d2953f; } .ach-E { background: #b0433a; }
.gr-x, .ach-x { background: var(--bg-2); color: var(--tx-4); border: 1px solid var(--bd-1); }
</style>
