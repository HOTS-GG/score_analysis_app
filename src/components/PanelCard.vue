<script setup>
import { Info } from 'lucide-vue-next'

/**
 * 분석 결과가 들어갈 "칸". 상태에 따라 본문 대신 플레이스홀더를 그린다.
 * status: 'ready' | 'nodata' | 'pending'
 *   nodata  — 필요한 데이터가 아직 없음 (requires 문구 표시)
 *   pending — 데이터는 있으나 계산 엔진이 아직 없음 (2단계 구현 예정)
 */
defineProps({
  title: { type: String, required: true },
  desc: { type: String, default: '' },
  status: { type: String, default: 'pending' },
  requires: { type: String, default: '' },
  chart: { type: Boolean, default: false },
  span: { type: Boolean, default: false },
  n: { type: Number, default: null },
  tag: { type: String, default: '' },
})
</script>

<template>
  <div class="panel" :class="{ 'panel--span': span }">
    <div class="panel-head">
      <div>
        <h3 class="panel-title">
          {{ title }}
          <span v-if="tag" class="badge badge--muted">{{ tag }}</span>
          <span v-if="n !== null" class="badge" :class="n < 30 ? 'badge--warn' : 'badge--muted'">N={{ n }}</span>
        </h3>
        <p v-if="desc" class="panel-desc">{{ desc }}</p>
      </div>
      <slot name="actions" />
    </div>

    <slot v-if="status === 'ready'" />
    <div v-else class="placeholder" :class="{ 'placeholder--chart': chart }">
      <template v-if="status === 'nodata'">
        <span class="placeholder-title">데이터 없음</span>
        <span>{{ requires || '이 칸을 채우려면 데이터 가져오기가 필요합니다.' }}</span>
      </template>
      <template v-else>
        <span class="placeholder-title">
          <Info :size="14" style="display:inline; vertical-align:-2px" /> 계산 엔진 준비 중
        </span>
        <span>데이터는 준비되었습니다. 이 칸은 2단계(계산 파이프라인)에서 채워집니다.</span>
      </template>
    </div>
  </div>
</template>

<style scoped>
.panel--span { grid-column: 1 / -1; }
</style>
