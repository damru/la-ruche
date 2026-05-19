<script setup lang="ts">
// Composant atomique — CProgressBar
// "Architectural Progress Bar" — DDR-001
import { computed } from 'vue'

type Variant = 'primary' | 'warning' | 'success' | 'error'

const props = withDefaults(defineProps<{
  value: number          // 0–100
  label?: string
  showValue?: boolean
  variant?: Variant
  animated?: boolean
}>(), {
  label: undefined,
  showValue: true,
  variant: 'primary',
  animated: true,
})

const clampedValue = computed(() => Math.min(100, Math.max(0, props.value)))

const fillStyle = computed(() => ({
  width: `${clampedValue.value}%`,
}))
</script>

<template>
  <div class="c-progress">
    <div
      v-if="label || showValue"
      class="c-progress__header"
    >
      <span
        v-if="label"
        class="c-progress__label"
      >{{ label }}</span>
      <span
        v-if="showValue"
        :class="[
          'c-progress__value font-mono',
          {
            'tone--default': variant === 'primary',
            'tone--warn': variant === 'warning',
            'tone--valid': variant === 'success',
            'tone--err': variant === 'error',
          },
        ]"
      >
        {{ clampedValue }}%
      </span>
    </div>
    <div
      class="c-progress__track"
      role="progressbar"
      :aria-valuenow="clampedValue"
      aria-valuemin="0"
      aria-valuemax="100"
      :aria-label="label"
    >
      <div
        class="c-progress__fill"
        :class="[`c-progress__fill--${variant}`, { 'c-progress__fill--animated': animated }]"
        :style="fillStyle"
      ></div>
    </div>
  </div>
</template>

<style scoped>
.c-progress { display: flex; flex-direction: column; gap: var(--space-2); }

.c-progress__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.c-progress__label {
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
  color: var(--color-on-surface);
}

.c-progress__value {
  font-size: var(--text-sm);
  font-weight: var(--fw-semibold);
  color: var(--tone-fg, var(--color-primary));
}

.c-progress__track {
  height: 10px;
  border-radius: var(--radius-full);
  background: var(--color-surface-container-highest);
  overflow: hidden;
}

.c-progress__fill {
  height: 100%;
  border-radius: var(--radius-full);
}

.c-progress__fill--animated {
  transition: width 0.8s cubic-bezier(0.34, 1.3, 0.64, 1);
}

.c-progress__fill--primary { background: var(--gradient-primary); }
.c-progress__fill--warning  { background: linear-gradient(135deg, #e65100, #ff9800); }
.c-progress__fill--success  { background: linear-gradient(135deg, #1b5e20, #43a047); }
.c-progress__fill--error    { background: linear-gradient(135deg, #93000a, #ba1a1a); }
</style>
