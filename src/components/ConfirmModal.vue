<script setup>
import { ref } from 'vue'
import BaseModal from './BaseModal.vue'

const props = defineProps({
  title: { type: String, required: true },
  message: { type: String, default: '' },
  confirmLabel: { type: String, default: '확인' },
  danger: { type: Boolean, default: false },
  /** 값을 입력받아야 할 때 (예: 제외 사유) */
  inputLabel: { type: String, default: '' },
  inputPlaceholder: { type: String, default: '' },
})
const emit = defineEmits(['confirm', 'cancel'])
const inputValue = ref('')
</script>

<template>
  <BaseModal :title="title" max-width="440px" @close="emit('cancel')">
    <div class="body">
      <p v-if="message" class="msg">{{ message }}</p>
      <div v-if="inputLabel" class="form-field">
        <label class="form-label">{{ inputLabel }}</label>
        <input v-model="inputValue" class="ui-input" :placeholder="inputPlaceholder" @keyup.enter="emit('confirm', inputValue)" />
      </div>
    </div>
    <template #footer>
      <span />
      <div class="row">
        <button class="btn-secondary" @click="emit('cancel')">취소</button>
        <button :class="danger ? 'btn-danger' : 'btn-primary'" :disabled="!!inputLabel && !inputValue.trim()" @click="emit('confirm', inputValue)">
          {{ confirmLabel }}
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<style scoped>
.body { padding: 20px 24px; display: flex; flex-direction: column; gap: 14px; }
.msg { margin: 0; font-size: 14px; color: var(--tx-2); line-height: 1.6; white-space: pre-line; }
</style>
