<script setup>
/** 가로 막대 목록. rows: [{ label, value, max, text, alt(두 번째 막대 값), cls }] */
defineProps({ rows: { type: Array, required: true } })
</script>

<template>
  <div class="hbars">
    <div v-for="(r, i) in rows" :key="i" class="hbar">
      <span class="lb">{{ r.label }}</span>
      <span class="track">
        <span class="fill" :class="r.cls" :style="{ width: (r.value == null ? 0 : Math.min(100, (r.value / (r.max || 100)) * 100)) + '%' }" />
        <span v-if="r.alt != null" class="fill alt" :style="{ width: Math.min(100, (r.alt / (r.max || 100)) * 100) + '%' }" />
      </span>
      <span class="val" v-html="r.text ?? (r.value == null ? '—' : r.value)" />
    </div>
  </div>
</template>

<style scoped>
.hbars { display: flex; flex-direction: column; gap: 7px; }
.hbar { display: grid; grid-template-columns: 96px 1fr 70px; align-items: center; gap: 10px; font-size: 12.5px; }
.lb { color: var(--tx-2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.track { position: relative; height: 14px; background: var(--bg-2); border: 1px solid var(--bd-1); border-radius: 4px; overflow: hidden; }
.fill { position: absolute; left: 0; top: 0; bottom: 0; background: var(--accent-hex); opacity: 0.85; }
.fill.alt { top: 50%; background: var(--clr-note-text); opacity: 0.7; }
.val { text-align: right; color: var(--tx-3); font-variant-numeric: tabular-nums; line-height: 1.2; }
</style>
