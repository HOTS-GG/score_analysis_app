<script setup>
import { reactive, ref } from 'vue'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  mode: { type: String, default: 'add' },
  student: { type: Object, default: null },
  defaultCohort: { type: Number, default: new Date().getFullYear() },
})
const emit = defineEmits(['save', 'close'])

const form = reactive({
  cohort: props.student?.cohort ?? props.defaultCohort,
  grade: props.student?.grade ?? 1,
  class_no: props.student?.class_no ?? 1,
  number: props.student?.number ?? 1,
  name: props.student?.name ?? '',
})
const error = ref('')

function submit() {
  error.value = ''
  const s = { cohort: +form.cohort, grade: +form.grade, class_no: +form.class_no, number: +form.number, name: form.name.trim() }
  if (!s.cohort || !s.grade || !s.class_no || !s.number) {
    error.value = '입학연도·학년·반·번호를 모두 입력하세요.'
    return
  }
  emit('save', s)
}
</script>

<template>
  <BaseModal :title="mode === 'add' ? '학생 추가' : '학생 정보 수정'" max-width="460px" @close="emit('close')">
    <div class="body">
      <div class="form-grid">
        <div class="form-field">
          <label class="form-label">입학연도 <span class="req">*</span></label>
          <input v-model="form.cohort" type="number" class="ui-input" />
        </div>
        <div class="form-field">
          <label class="form-label">학년 <span class="req">*</span></label>
          <select v-model="form.grade" class="ui-select">
            <option :value="1">1학년</option>
            <option :value="2">2학년</option>
            <option :value="3">3학년</option>
          </select>
        </div>
        <div class="form-field">
          <label class="form-label">반 <span class="req">*</span></label>
          <input v-model="form.class_no" type="number" min="1" class="ui-input" />
        </div>
        <div class="form-field">
          <label class="form-label">번호 <span class="req">*</span></label>
          <input v-model="form.number" type="number" min="1" class="ui-input" />
        </div>
      </div>
      <div class="form-field">
        <label class="form-label">이름</label>
        <input v-model="form.name" class="ui-input" placeholder="실명은 명단에만 저장됩니다" @keyup.enter="submit" />
      </div>
      <p v-if="error" class="msg-error">{{ error }}</p>
    </div>
    <template #footer>
      <span class="hint">익명 ID는 저장 시 자동 부여됩니다.</span>
      <div class="row">
        <button class="btn-secondary" @click="emit('close')">취소</button>
        <button class="btn-primary" @click="submit">저장</button>
      </div>
    </template>
  </BaseModal>
</template>

<style scoped>
.body { padding: 20px 24px; display: flex; flex-direction: column; gap: 14px; }
.hint { font-size: 12px; color: var(--tx-4); }
</style>
