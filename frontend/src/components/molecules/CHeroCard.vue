<script setup lang="ts">
import CCard from '@/components/molecules/CCard.vue'
import CKpiMetric from '@/components/atoms/CKpiMetric.vue'

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
  <CCard
    class="hero-card"
    padding="md"
  >
    <div class="hero-card__header">
      <p class="hero-card__label">
        {{ label }}
      </p>
      <div
        v-if="$slots.icon"
        :class="['hero-card__icon', 'kpi-icon', `tone--${tone}`]"
      >
        <slot name="icon"></slot>
      </div>
    </div>
    <CKpiMetric
      :data="value"
      :class="{ [`tone--${tone}`]: emphasizeValue }"
    >
      <template #subtitle>
        <slot name="subtitle"></slot>
      </template>
    </CKpiMetric>
    <div
      v-if="$slots.footer"
      class="hero-card__footer"
    >
      <slot name="footer"></slot>
    </div>
  </CCard>
</template>

<style scoped>
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

.hero-card__icon {
  flex-shrink: 0;
}
.hero-card :deep(.c-kpi-metric__subtitle) {
  margin-top: 0;
}

.hero-card__footer {
  margin-top: var(--space-3);
}
</style>
