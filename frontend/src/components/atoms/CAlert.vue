<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

type Variant = 'info' | 'warning' | 'error' | 'success'

withDefaults(defineProps<{
  variant?: Variant
  title?: string
  dismissible?: boolean
}>(), {
  variant: 'info',
  dismissible: false,
})

const emit = defineEmits<{ dismiss: [] }>()
const visible = ref(true)

const icons: Record<Variant, string> = {
  info:    'M12 2a10 10 0 100 20A10 10 0 0012 2zm0 9v5m0-9v.01',
  warning: 'M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0zM12 9v4m0 4h.01',
  error:   'M12 2a10 10 0 100 20A10 10 0 0012 2zm4-4L8 16m0-8l8 8',
  success: 'M22 11.08V12a10 10 0 11-5.93-9.14M22 4L12 14.01l-3-3',
}

function dismiss() {
  visible.value = false
  emit('dismiss')
}
</script>

<template>
  <Transition name="c-alert-fade">
    <div v-if="visible" :class="['c-alert', `c-alert--${variant}`]" role="alert">
      <svg class="c-alert__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <path :d="icons[variant]" />
      </svg>
      <div class="c-alert__body">
        <p v-if="title" class="c-alert__title">{{ title }}</p>
        <p class="c-alert__text"><slot /></p>
      </div>
      <button v-if="dismissible" class="c-alert__close" @click="dismiss" :aria-label="t('common.close')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.c-alert {
  display: flex;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
}

.c-alert__icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  margin-top: 1px;
}

.c-alert__body { flex: 1; min-width: 0; }

.c-alert__title {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 2px;
}

.c-alert__text { font-size: 13px; opacity: 0.9; line-height: 1.4; }

.c-alert__close {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  min-width: unset;
  min-height: unset;
  border-radius: var(--radius-sm);
  opacity: 0.6;
  transition: opacity var(--transition-fast);
}
.c-alert__close:hover { opacity: 1; }

/* ── Variantes ────────────────────────────────────────────────── */
.c-alert--info    { background: var(--color-secondary-container); color: var(--color-on-secondary-container); }
.c-alert--warning { background: var(--color-warning-container);   color: var(--color-warning); }
.c-alert--error   { background: var(--color-error-container);     color: var(--color-error); }
.c-alert--success { background: var(--color-success-container);   color: var(--color-success); }

/* ── Transition ────────────────────────────────────────────────── */
.c-alert-fade-enter-active { transition: all var(--transition-base); }
.c-alert-fade-leave-active { transition: all var(--transition-fast); }
.c-alert-fade-enter-from, .c-alert-fade-leave-to { opacity: 0; transform: translateY(-4px); }
</style>
