<script setup lang="ts">
type Tone = 'default' | 'accent' | 'warn' | 'err' | 'valid' | 'muted'

withDefaults(defineProps<{
  label: string
  value: string | number
  tone?: Tone
  emphasizeValue?: boolean
}>(), {
  tone: 'default',
  emphasizeValue: false,
})
</script>

<template>
  <div class="hero-card">
    <div class="hero-card__header">
      <p class="hero-card__label">
        {{ label }}
      </p>
      <div
        v-if="$slots.icon"
        :class="['kpi-icon', `tone--${tone}`]"
      >
        <slot name="icon"></slot>
      </div>
    </div>

    <p
      class="hero-card__value"
      :class="{ [`tone--${tone}`]: emphasizeValue }"
    >
      {{ value }}
    </p>

    <div
      v-if="$slots.subtitle"
      class="hero-card__subtitle"
    >
      <slot name="subtitle"></slot>
    </div>
  </div>
</template>

<style scoped>
.hero-card {
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  padding: var(--space-5);
  box-shadow: var(--shadow-md);
}

.hero-card__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
}

.hero-card__label {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
}

.hero-card__value {
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: var(--fw-bold);
  font-variation-settings: 'opsz' 72;
  letter-spacing: -0.03em;
  color: var(--tone-fg, var(--color-on-surface));
  line-height: 1;
}

.hero-card__subtitle {
  margin-top: var(--space-1);
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
}
</style>
