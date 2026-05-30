<script setup lang="ts">
type DeltaVariant = 'up' | 'down' | 'neutral'

withDefaults(defineProps<{
  data: string | number
  subtitle?: string
  delta?: string
  deltaVariant?: DeltaVariant
}>(), {
  subtitle: undefined,
  delta: undefined,
  deltaVariant: 'neutral',
})
</script>

<template>
  <div class="c-kpi-metric">
    <p class="c-kpi-metric__data">
      <slot name="data">
        {{ data }}
      </slot>
    </p>
    <p
      v-if="subtitle || $slots.subtitle"
      class="c-kpi-metric__subtitle"
    >
      <slot name="subtitle">
        {{ subtitle }}
      </slot>
    </p>
    <p
      v-if="delta || $slots.delta"
      :class="['c-kpi-metric__delta', `c-kpi-metric__delta--${deltaVariant}`]"
    >
      <slot name="delta">
        {{ delta }}
      </slot>
    </p>
  </div>
</template>

<style scoped>
.c-kpi-metric {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  min-width: 0;
}

.c-kpi-metric__data {
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: var(--fw-bold);
  font-variation-settings: 'opsz' 72;
  letter-spacing: -0.03em;
  color: var(--tone-fg, var(--color-on-surface));
  line-height: 1.5;
}

.c-kpi-metric__subtitle {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
}

.c-kpi-metric__delta {
  font-size: var(--text-xs);
  font-weight: var(--fw-medium);
  display: inline-flex;
  align-items: center;
  gap: 3px;
}

.c-kpi-metric__delta--up { color: var(--color-success); }
.c-kpi-metric__delta--down { color: var(--color-error); }
.c-kpi-metric__delta--neutral { color: var(--color-on-surface-variant); }
</style>
