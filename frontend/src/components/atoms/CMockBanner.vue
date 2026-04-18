<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(defineProps<{
  variant?: 'mock' | 'demo'
}>(), {
  variant: 'mock',
})

const { t } = useI18n()

const title = computed(() => (
  props.variant === 'demo'
    ? t('common.demoModeTitle')
    : t('common.mockModeTitle')
))

const message = computed(() => (
  props.variant === 'demo'
    ? t('common.demoModeMessage')
    : t('common.mockModeMessage')
))
</script>

<template>
  <div
    :class="['mock-banner', `mock-banner--${variant}`]"
    role="status"
    aria-live="polite"
  >
    <CIconLayers aria-hidden="true" />
    <span>
      <strong>{{ title }}</strong> — {{ message }}
    </span>
  </div>
</template>

<style scoped>
.mock-banner {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 10px var(--space-4);
  font-size: var(--text-sm);
  margin: var(--space-2);
}

.mock-banner--mock {
  background: repeating-linear-gradient(
    -45deg,
    #7c3aed18 0px,
    #7c3aed18 10px,
    transparent 10px,
    transparent 20px
  ),
  linear-gradient(135deg, #7c3aed14, #a855f714);
  border: 1.5px solid #7c3aed40;
  border-radius: var(--radius-xl);
  color: #5b21b6;
}

.mock-banner--demo {
  background:
    repeating-linear-gradient(
      -45deg,
      rgba(184, 115, 51, 0.10) 0px,
      rgba(184, 115, 51, 0.10) 10px,
      transparent 10px,
      transparent 20px
    ),
    linear-gradient(135deg, rgba(250, 204, 21, 0.18), rgba(249, 115, 22, 0.18));
  border: 1px solid rgba(180, 83, 9, 0.28);
  border-radius: var(--radius-xl);
  color: #92400e;
}

[data-theme="dark"] .mock-banner--mock {
  background: repeating-linear-gradient(
    -45deg,
    #a78bfa18 0px,
    #a78bfa18 10px,
    transparent 10px,
    transparent 20px
  ),
  linear-gradient(135deg, #7c3aed20, #a855f720);
  border-color: #7c3aed50;
  color: #c4b5fd;
}

[data-theme="dark"] .mock-banner--demo {
  background:
    repeating-linear-gradient(
      -45deg,
      rgba(251, 191, 36, 0.10) 0px,
      rgba(251, 191, 36, 0.10) 10px,
      transparent 10px,
      transparent 20px
    ),
    linear-gradient(135deg, rgba(120, 53, 15, 0.34), rgba(154, 52, 18, 0.34));
  border-color: rgba(245, 158, 11, 0.35);
  color: #fcd34d;
}

.mock-banner svg {
  width: var(--icon-sm);
  height: var(--icon-sm);
  flex-shrink: 0;
  opacity: 0.8;
}
</style>
