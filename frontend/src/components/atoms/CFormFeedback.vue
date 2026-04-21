<script setup lang="ts">
import type { FeedbackStatus } from '@/composables/useFormFeedback'

defineProps<{
  status: FeedbackStatus
  message?: string
}>()
</script>

<template>
  <Transition name="feedback">
    <span
      v-if="status !== 'idle'"
      :class="['form-feedback', `form-feedback--${status}`]"
    >
      <span
        class="form-feedback__icon"
        aria-hidden="true"
      >{{ status === 'success' ? '✓' : '✕' }}</span>
      <span v-if="message">{{ message }}</span>
    </span>
  </Transition>
</template>

<style scoped>
.form-feedback {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  font-weight: 500;
  transition: opacity var(--transition-fast);
}

.form-feedback--success { color: var(--color-success, #2e7d32); }
.form-feedback--error   { color: var(--color-error); }

.form-feedback__icon { font-size: 14px; }

/* fade in/out */
.feedback-enter-active,
.feedback-leave-active { transition: opacity 0.25s ease; }
.feedback-enter-from,
.feedback-leave-to    { opacity: 0; }
</style>
