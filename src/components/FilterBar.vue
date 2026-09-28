<script setup>
import { computed } from 'vue'
import { useDatasetStore } from '../stores/dataset'

/**
 * 학년도 / 학년 / 회차 / 과목 선택 바. 선택지는 가져온 모의고사 회차에서 자동 생성.
 * v-model = { year, grade, round, subject }
 */
const props = defineProps({
  modelValue: { type: Object, required: true },
  show: { type: Array, default: () => ['year', 'grade', 'round', 'subject'] },
})
const emit = defineEmits(['update:modelValue'])
const dataset = useDatasetStore()
const opts = computed(() => dataset.filterOptions)

function set(key, value) {
  emit('update:modelValue', { ...props.modelValue, [key]: value })
}
const has = (k) => props.show.includes(k)
</script>

<template>
  <div class="filter-bar">
    <div v-if="has('year')" class="filter-field">
      <span class="filter-label">학년도</span>
      <select class="ui-select" :value="modelValue.year" :disabled="!opts.years.length" @change="set('year', $event.target.value)">
        <option v-if="!opts.years.length" value="">데이터 없음</option>
        <option v-for="y in opts.years" :key="y" :value="y">{{ y }}학년도</option>
      </select>
    </div>
    <div v-if="has('grade')" class="filter-field">
      <span class="filter-label">학년</span>
      <select class="ui-select" :value="modelValue.grade" :disabled="!opts.grades.length" @change="set('grade', $event.target.value)">
        <option v-if="!opts.grades.length" value="">데이터 없음</option>
        <option v-for="g in opts.grades" :key="g" :value="g">{{ g }}학년</option>
      </select>
    </div>
    <div v-if="has('round')" class="filter-field">
      <span class="filter-label">회차</span>
      <select class="ui-select" :value="modelValue.round" :disabled="!opts.rounds.length" @change="set('round', $event.target.value)">
        <option v-if="!opts.rounds.length" value="">데이터 없음</option>
        <option v-for="r in opts.rounds" :key="r" :value="r">{{ r }}월 학력평가</option>
      </select>
    </div>
    <div v-if="has('subject')" class="filter-field">
      <span class="filter-label">과목</span>
      <select class="ui-select" :value="modelValue.subject" :disabled="!opts.subjects.length" @change="set('subject', $event.target.value)">
        <option v-if="!opts.subjects.length" value="">데이터 없음</option>
        <option v-for="s in opts.subjects" :key="s" :value="s">{{ s }}</option>
      </select>
    </div>
    <slot />
  </div>
</template>
