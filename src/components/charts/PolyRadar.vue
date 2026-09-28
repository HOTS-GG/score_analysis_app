<script setup>
import { computed } from 'vue'

/**
 * 다중 시리즈 레이더. 등급(1 = 바깥) 또는 0~100(100 = 바깥) 값을 그린다.
 * axes: ['국어',...], series: [{ name, values: [n|null], fill, color, label }]
 * mode: 'grade' (1~5, 1이 바깥) | 'pct' (0~100, 100이 바깥)
 */
const props = defineProps({
  axes: { type: Array, required: true },
  series: { type: Array, required: true },
  mode: { type: String, default: 'grade' },
  size: { type: Number, default: 300 },
})
const cx = computed(() => props.size / 2)
const cy = computed(() => props.size / 2)
const r = computed(() => props.size / 2 - 52)
const n = computed(() => props.axes.length)
const rings = computed(() => props.mode === 'grade' ? [1, 2, 3, 4, 5] : [20, 40, 60, 80, 100])

function ratio(v) {
  if (v === null || v === undefined) return 0
  if (props.mode === 'grade') return Math.max(0, Math.min(1, (5.5 - v) / 4.5))
  return Math.max(0, Math.min(1, v / 100))
}
function pt(i, ra) {
  const a = -Math.PI / 2 + (2 * Math.PI * i) / n.value
  return [cx.value + r.value * ra * Math.cos(a), cy.value + r.value * ra * Math.sin(a)]
}
const ringPaths = computed(() => rings.value.map(v => Array.from({ length: n.value }, (_, i) => pt(i, ratio(v)).join(',')).join(' ')))
const paths = computed(() => props.series.map(s => Array.from({ length: n.value }, (_, i) => pt(i, ratio(s.values?.[i])).join(',')).join(' ')))
const labels = computed(() => props.axes.map((a, i) => { const [x, y] = pt(i, 1.17); return { x, y, text: a, anchor: x < cx.value - 4 ? 'end' : x > cx.value + 4 ? 'start' : 'middle' } }))
const colors = ['var(--accent-hex)', 'var(--clr-note-text)', 'var(--clr-purple-text)']
</script>

<template>
  <div class="wrap">
    <svg :viewBox="`0 0 ${size} ${size}`" class="radar" role="img">
      <g v-if="n >= 3">
        <polygon v-for="(p, k) in ringPaths" :key="k" :points="p" class="ring" />
        <line v-for="i in n" :key="'ax' + i" :x1="cx" :y1="cy" :x2="pt(i - 1, 1)[0]" :y2="pt(i - 1, 1)[1]" class="axis" />
        <text v-if="mode === 'grade'" :x="cx + 4" :y="cy - r * ratio(1) + 10" class="tick">1등급</text>
        <text v-if="mode === 'grade'" :x="cx + 4" :y="cy - r * ratio(3) + 10" class="tick">3</text>
        <g v-for="(s, si) in series" :key="si">
          <polygon :points="paths[si]" :style="{ fill: s.fill ? (s.color ?? colors[si]) : 'none', fillOpacity: 0.22, stroke: s.color ?? colors[si], strokeWidth: 2.2, strokeDasharray: s.dashed ? '5 4' : 'none' }" />
          <g v-if="s.label">
            <circle v-for="(v, i) in s.values" :key="i" :cx="pt(i, ratio(v))[0]" :cy="pt(i, ratio(v))[1]" r="3.5" :style="{ fill: s.color ?? colors[si] }" />
            <text v-for="(v, i) in s.values" :key="'t' + i" :x="pt(i, ratio(v))[0]" :y="pt(i, ratio(v))[1] - 8" text-anchor="middle" class="val">{{ v == null ? '' : (mode === 'grade' ? Number(v).toFixed(1) : Math.round(v)) }}</text>
          </g>
        </g>
        <text v-for="(l, i) in labels" :key="'lb' + i" :x="l.x" :y="l.y" :text-anchor="l.anchor" dominant-baseline="middle" class="label">{{ l.text }}</text>
      </g>
      <text v-else :x="cx" :y="cy" text-anchor="middle" class="label">축 3개 이상일 때 표시</text>
    </svg>
    <div class="legend">
      <span v-for="(s, si) in series" :key="si"><i :style="{ background: s.color ?? colors[si] }" />{{ s.name }}</span>
    </div>
  </div>
</template>

<style scoped>
.wrap { display: flex; flex-direction: column; align-items: center; gap: 4px; }
.radar { width: 100%; max-width: 320px; height: auto; }
.ring { fill: none; stroke: var(--bd-2); stroke-width: 1; }
.axis { stroke: var(--bd-1); }
.label { font-size: 12px; font-weight: 600; fill: var(--tx-2); }
.tick { font-size: 9px; fill: var(--tx-4); }
.val { font-size: 10px; fill: var(--tx-1); font-weight: 700; }
.legend { display: flex; gap: 12px; font-size: 12px; color: var(--tx-3); flex-wrap: wrap; justify-content: center; }
.legend i { display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 5px; vertical-align: -1px; }
</style>
