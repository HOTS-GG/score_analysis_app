<script setup>
import { computed } from 'vue'
import { RotateCcw } from 'lucide-vue-next'

/** 기준값 한 칸: 이름·쉬운 설명·슬라이더·숫자·기본값 되돌리기 */
const props = defineProps({
  modelValue: { type: [Number, String], default: 0 },
  label: { type: String, required: true },
  desc: { type: String, default: '' },
  example: { type: String, default: '' },
  min: { type: Number, default: 0 },
  max: { type: Number, default: 100 },
  step: { type: Number, default: 1 },
  unit: { type: String, default: '' },
  def: { type: Number, default: null },
})
const emit = defineEmits(['update:modelValue'])
const val = computed({ get: () => Number(props.modelValue), set: v => emit('update:modelValue', Number(v)) })
const changed = computed(() => props.def !== null && Math.abs(val.value - props.def) > 1e-9)
const pct = computed(() => Math.max(0, Math.min(100, ((val.value - props.min) / (props.max - props.min)) * 100)))
</script>

<template>
  <div class="field" :class="{ changed }">
    <div class="top">
      <div class="lbl"><b>{{ label }}</b><span v-if="changed" class="badge badge--accent">변경됨</span></div>
      <div class="num">
        <input type="number" :step="step" :min="min" :max="max" v-model="val" class="ui-input ui-input--sm nbox" />
        <span class="unit">{{ unit }}</span>
        <button v-if="def !== null" class="btn-icon" :disabled="!changed" title="기본값으로" @click="val = def"><RotateCcw :size="13" /></button>
      </div>
    </div>
    <input type="range" :min="min" :max="max" :step="step" v-model="val" class="range" :style="{ '--p': pct + '%' }" />
    <div class="scale"><span>{{ min }}{{ unit }}</span><span v-if="def !== null" class="def">기본 {{ def }}{{ unit }}</span><span>{{ max }}{{ unit }}</span></div>
    <p v-if="desc" class="desc">{{ desc }}</p>
    <p v-if="example" class="ex">예) {{ example }}</p>
  </div>
</template>

<style scoped>
.field { border: 1px solid var(--bd-1); border-radius: 12px; padding: 12px 14px 10px; background: var(--bg-2); display: flex; flex-direction: column; gap: 6px; }
.field.changed { border-color: rgba(var(--accent-rgb), 0.5); }
.top { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
.lbl { display: flex; align-items: center; gap: 8px; }
.lbl b { font-size: 13.5px; color: var(--tx-1); }
.num { display: flex; align-items: center; gap: 6px; }
.nbox { width: 84px; text-align: right; }
.unit { font-size: 12px; color: var(--tx-4); min-width: 24px; }
.range { width: 100%; accent-color: var(--accent-hex); height: 22px; cursor: pointer; }
.scale { display: flex; justify-content: space-between; font-size: 10.5px; color: var(--tx-4); }
.def { color: var(--accent-text); }
.desc { margin: 2px 0 0; font-size: 12.5px; color: var(--tx-2); line-height: 1.5; }
.ex { margin: 0; font-size: 12px; color: var(--tx-4); line-height: 1.5; }
</style>
