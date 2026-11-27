<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(defineProps<{
  variant?: 'mock' | 'demo'
}>(), {
  variant: 'mock',
})

const { t } = useI18n()

const title = computed(() => t('common.demoModeTitle'))

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
    oklch(0.541 0.2466 293.01 / 0.09) 0px,
    oklch(0.541 0.2466 293.01 / 0.09) 10px,
    transparent 10px,
    transparent 20px
  ),
  linear-gradient(135deg, oklch(0.541 0.2466 293.01 / 0.08), oklch(0.627 0.2325 303.90 / 0.08));
  border: 1.5px solid oklch(0.541 0.2466 293.01 / 0.25);
  border-radius: var(--radius-xl);
  color: oklch(0.432 0.2106 292.76);
}

.mock-banner--demo {
  background:
    repeating-linear-gradient(
      -45deg,
      oklch(0.618 0.1171 60.40 / 0.1) 0px,
      oklch(0.618 0.1171 60.40 / 0.1) 10px,
      transparent 10px,
      transparent 20px
    ),
    linear-gradient(135deg, oklch(0.861 0.1731 91.94 / 0.18), oklch(0.705 0.1867 47.60 / 0.18));
  border: 1px solid oklch(0.555 0.1455 49.00 / 0.28);
  border-radius: var(--radius-xl);
  color: oklch(0.473 0.1247 46.20);
}

[data-theme="dark"] .mock-banner--mock {
  background: repeating-linear-gradient(
    -45deg,
    oklch(0.709 0.1592 293.54 / 0.09) 0px,
    oklch(0.709 0.1592 293.54 / 0.09) 10px,
    transparent 10px,
    transparent 20px
  ),
  linear-gradient(135deg, oklch(0.541 0.2466 293.01 / 0.12), oklch(0.627 0.2325 303.90 / 0.12));
  border-color: oklch(0.541 0.2466 293.01 / 0.31);
  color: var(--color-primary);
}

[data-theme="dark"] .mock-banner--demo {
  background:
    repeating-linear-gradient(
      -45deg,
      oklch(0.837 0.1644 84.43 / 0.1) 0px,
      oklch(0.837 0.1644 84.43 / 0.1) 10px,
      transparent 10px,
      transparent 20px
    ),
    linear-gradient(135deg, oklch(0.414 0.1054 45.90 / 0.34), oklch(0.470 0.1430 37.30 / 0.34));
  border-color: oklch(0.769 0.1647 70.08 / 0.35);
  color: oklch(0.879 0.1534 91.61);
}

.mock-banner svg {
  width: var(--icon-sm);
  height: var(--icon-sm);
  flex-shrink: 0;
  opacity: 0.8;
}
</style>
