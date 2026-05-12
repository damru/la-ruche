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
      :class="['form-feedback', { 'tone--valid': status === 'success', 'tone--err': status === 'error' }]"
    >
      <span
        class="form-feedback__icon"
        aria-hidden="true"
      >{{ status === 'success' ? '✓' : '✗' }}</span>
      <span v-if="message">{{ message }}</span>
    </span>
  </Transition>
</template>

<style scoped>
.form-feedback {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
  transition: opacity var(--transition-fast);
  color: var(--tone-fg, inherit);
}

.form-feedback__icon { font-size: var(--text-md); }

/* fade in/out */
.feedback-enter-active,
.feedback-leave-active { transition: opacity 0.25s ease; }
.feedback-enter-from,
.feedback-leave-to    { opacity: 0; }
</style>
