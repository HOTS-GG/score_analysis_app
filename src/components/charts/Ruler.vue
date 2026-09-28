<script setup>
/** 0~100 자 위에 핀 여러 개. pins: [{ value(0~100, 오른쪽=상위), label, color, below }] */
defineProps({ pins: { type: Array, required: true }, ticks: { type: Array, default: () => [0, 25, 50, 75, 100] } })
</script>

<template>
  <div class="ruler">
    <span class="track" />
    <span v-for="t in ticks" :key="t" class="tick" :style="{ left: t + '%' }">{{ t }}</span>
    <span v-for="(p, i) in pins.filter(p => p.value != null)" :key="i" class="pin" :class="{ below: p.below }" :style="{ left: `calc(${Math.max(0, Math.min(100, p.value))}% - 1.5px)`, background: p.color }">
      <b :style="{ color: p.color }">{{ p.label }}</b>
    </span>
  </div>
</template>

<style scoped>
.ruler { position: relative; height: 62px; margin: 6px 4px 4px; }
.track { position: absolute; left: 0; right: 0; top: 24px; height: 8px; border-radius: 4px; background: linear-gradient(90deg, var(--clr-red-bg), var(--bg-hover), var(--clr-green-bg)); border: 1px solid var(--bd-1); }
.tick { position: absolute; top: 36px; transform: translateX(-50%); font-size: 10px; color: var(--tx-4); }
.pin { position: absolute; top: 4px; width: 3px; height: 30px; border-radius: 2px; }
.pin b { position: absolute; top: -14px; left: 50%; transform: translateX(-50%); font-size: 11px; white-space: nowrap; }
.pin.below { top: 20px; height: 26px; }
.pin.below b { top: 28px; }
</style>
