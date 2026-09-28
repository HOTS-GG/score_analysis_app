<script setup>
/** 성취도 A→E 개수 스택 막대 */
defineProps({ counts: { type: Object, required: true }, mini: { type: Boolean, default: false } })
const keys = ['a', 'b', 'c', 'd', 'e']
</script>

<template>
  <span class="stack" :class="{ mini }" :title="`A ${counts.a} · B ${counts.b} · C ${counts.c} · D ${counts.d} · E ${counts.e}`">
    <template v-for="k in keys" :key="k">
      <i v-if="counts[k]" :class="'s-' + k" :style="{ flex: counts[k] }">{{ mini ? '' : counts[k] }}</i>
    </template>
    <i v-if="!keys.some(k => counts[k])" class="s-x" style="flex:1" />
  </span>
</template>

<style scoped>
.stack { display: inline-flex; width: 120px; height: 16px; border-radius: 4px; overflow: hidden; border: 1px solid var(--bd-1); vertical-align: middle; }
.stack.mini { width: 90px; height: 10px; }
.stack i { display: flex; align-items: center; justify-content: center; font-style: normal; font-size: 10px; color: #fff; min-width: 2px; }
.s-a { background: #0f6d68; } .s-b { background: #57988f; } .s-c { background: #aeb7b9; color: #123 !important; } .s-d { background: #d2953f; } .s-e { background: #b0433a; } .s-x { background: var(--bg-2); }
</style>
