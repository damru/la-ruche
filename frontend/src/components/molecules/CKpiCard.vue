<script setup lang="ts">
// Molecule component — CKpiCard
// Used on the dashboard for key metrics
import CCard from '@/components/molecules/CCard.vue'
import CKpiMetric from '@/components/atoms/CKpiMetric.vue'

type DeltaVariant = 'up' | 'down' | 'neutral'

withDefaults(defineProps<{
  label: string
  value: string
  delta?: string
  deltaVariant?: DeltaVariant
  iconAccent?: boolean    // gradient on the icon
}>(), {
  delta: undefined,
  deltaVariant: 'neutral',
  iconAccent: false,
})
</script>

<template>
  <CCard
    class="c-kpi"
    padding="md"
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
          'kpi-icon',
          iconAccent ? 'tone--accent' : 'tone--default',
        ]"
      >
        <slot name="icon"></slot>
      </div>
    </div>
    <CKpiMetric
      :data="value"
      :delta="delta"
      :delta-variant="deltaVariant"
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
}

.c-kpi__label {
  font-size: var(--text-sm);
  color: var(--color-on-surface-variant);
  font-weight: var(--fw-regular);
}
</style>
