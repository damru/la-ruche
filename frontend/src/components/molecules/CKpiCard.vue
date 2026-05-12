<script setup lang="ts">
// Molecule component — CKpiCard
// Used on the dashboard for key metrics

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
  <div class="c-kpi">
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
    <p class="c-kpi__value">
      {{ value }}
    </p>
    <p
      v-if="delta"
      :class="['c-kpi__delta', `c-kpi__delta--${deltaVariant}`]"
    >
      {{ delta }}
    </p>
    <slot name="extra"></slot>
  </div>
</template>

<style scoped>
.c-kpi {
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  padding: var(--space-5);
  box-shadow: var(--shadow-md);
}

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

.c-kpi__value {
  font-family: var(--font-display);
  font-size: 32px;
  font-weight: var(--fw-bold);
  font-variation-settings: 'opsz' 72;
  letter-spacing: -0.03em;
  color: var(--color-on-surface);
  line-height: 1;
  margin-bottom: var(--space-1);
}

.c-kpi__delta {
  font-size: var(--text-xs);
  font-weight: var(--fw-medium);
  display: inline-flex;
  align-items: center;
  gap: 3px;
}
.c-kpi__delta--up      { color: var(--color-success); }
.c-kpi__delta--down    { color: var(--color-error); }
.c-kpi__delta--neutral { color: var(--color-on-surface-variant); }
</style>
