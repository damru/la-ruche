<script setup lang="ts">
// Molecule component — CKpiCard
// Used on the dashboard for key metrics
import CCard from '@/components/molecules/CCard.vue'
import CKpiMetric from '@/components/atoms/CKpiMetric.vue'

withDefaults(defineProps<{
  label: string
  value: string
  subtitle?: string
  delta?: string
  deltaVariant?: 'up' | 'down' | 'neutral'
  tone?: 'default' | 'accent' | 'warn' | 'err' | 'valid' | 'info'
  emphasize?: boolean
  emphasizeValue?: boolean
}>(), {
  delta: undefined,
  deltaVariant: 'neutral',
  subtitle: undefined,
  tone: 'default',
  emphasize: false,
  emphasizeValue: false,
})
</script>

<template>
  <CCard
    padding="md"
    class="min-w-40!"
    :class="emphasize ? `tone--${tone}` : ''"
  >
    <div class="c-kpi__header">
      <div>
        <p class="c-kpi__label">
          {{ label }}
        </p>
      </div>
      <div
        v-if="$slots.icon"
        :class="[
          'c-kpi__icon',
          `tone--${tone}`,
        ]"
      >
        <slot name="icon"></slot>
      </div>
    </div>
    <CKpiMetric
      :data="value"
      :delta="delta"
      :delta-variant="deltaVariant"
      :subtitle="subtitle"
      :class="emphasizeValue ? `tone--${tone}` : ''"
    />
    <slot name="extra"></slot>
  </CCard>
</template>

<style scoped>
.c-kpi__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: var(--space-3);
  text-transform: uppercase;
}

.c-kpi__label {
  font-size: var(--text-sm);
  color: var(--color-on-surface-variant);
  font-weight: var(--fw-regular);
}

.c-kpi__icon {
  width: calc(var(--icon-md) * 2);
  height: calc(var(--icon-md) * 2);
  border-radius: var(--radius-xl);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: var(--tone-bg, var(--color-secondary-container));
  color: var(--tone-fg, var(--color-primary));
}

.c-kpi__icon :deep(svg) {
  width: var(--icon-md);
  height: var(--icon-md);
}
</style>
