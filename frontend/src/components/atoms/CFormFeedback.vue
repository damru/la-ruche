<script setup lang="ts">
import type { FeedbackStatus } from '@/composables/useFormFeedback'

defineProps<{
  status: FeedbackStatus
  message?: string
}>()
</script>

<template>
  <Transition name="feedback">
    <div
      v-if="status !== 'idle'"
      class="form-feedback-wrap"
    >
      <span
        :class="['form-feedback', { 'tone--valid': status === 'success', 'tone--err': status === 'error' }]"
      >
        <span
          class="form-feedback__icon"
          aria-hidden="true"
        >{{ status === 'success' ? '✓' : '✗' }}</span>
        <span v-if="message">{{ message }}</span>
      </span>
    </div>
  </Transition>
</template>

<style scoped>
/* Grid wrapper — always 1fr; transitions to 0fr on leave for push effect */
.form-feedback-wrap {
  display: grid;
  grid-template-rows: 1fr;
  width: 100%;
  justify-content: end;
}

/* overflow:hidden on the inner span lets 0fr fully collapse the row */
.form-feedback-wrap > .form-feedback {
  overflow: hidden;
}

.form-feedback {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding-top: var(--space-2); /* spacing from buttons; clipped during 0fr */
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
  color: var(--tone-fg, inherit);
}

.form-feedback__icon { font-size: var(--text-md); }

/* Push in/out: height + fade */
.feedback-enter-active,
.feedback-leave-active {
  transition: grid-template-rows 0.28s ease, opacity 0.25s ease;
}
.feedback-enter-from,
.feedback-leave-to {
  grid-template-rows: 0fr;
  opacity: 0;
}
</style>
