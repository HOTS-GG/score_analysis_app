<script setup>
import { computed } from 'vue'
/** 등급 추이 (1등급이 위). points: [{term, gpa}] */
const props = defineProps({ points: { type: Array, required: true }, width: { type: Number, default: 220 }, height: { type: Number, default: 64 } })
const pad = { l: 8, r: 8, t: 12, b: 16 }
const xs = computed(() => props.points.map((_, i) => props.points.length === 1 ? props.width / 2 : pad.l + (i * (props.width - pad.l - pad.r)) / (props.points.length - 1)))
const y = (g) => pad.t + ((g - 1) / 4) * (props.height - pad.t - pad.b)
const path = computed(() => props.points.map((p, i) => p.gpa == null ? null : `${xs.value[i]},${y(p.gpa)}`).filter(Boolean).join(' '))
</script>

<template>
  <svg :viewBox="`0 0 ${width} ${height}`" class="spark">
    <line v-for="g in [1, 3, 5]" :key="g" :x1="pad.l" :x2="width - pad.r" :y1="y(g)" :y2="y(g)" class="grid" />
    <polyline v-if="path" :points="path" class="line" />
    <g v-for="(p, i) in points" :key="i">
      <circle v-if="p.gpa != null" :cx="xs[i]" :cy="y(p.gpa)" r="3.2" class="dot" />
      <text :x="xs[i]" :y="height - 3" text-anchor="middle" class="lbl">{{ p.term }}</text>
    </g>
  </svg>
</template>

<style scoped>
.spark { width: 100%; height: auto; }
.grid { stroke: var(--bd-1); stroke-width: 1; }
.line { fill: none; stroke: var(--accent-hex); stroke-width: 2; stroke-linejoin: round; }
.dot { fill: var(--accent-hex); }
.lbl { font-size: 9px; fill: var(--tx-4); }
</style>
