<script setup lang="ts">
import CCard from '@/components/molecules/CCard.vue'

type ReminderTone = 'default' | 'info' | 'accent' | 'warn' | 'err' | 'disabled' | 'muted' | 'valid'

type ReminderRow = {
  id: string
  title: string
  subtitle?: string
  meta?: string
  emphasis?: string
  tone?: ReminderTone
}

defineProps<{
  title: string
  rows: ReminderRow[]
}>()
</script>

<template>
  <div class="flex flex-col gap-3">
    <CHeading>
      {{ title }}
    </CHeading>
    <CCard
      v-for="row in rows"
      :key="row.id"
      :level="4"
      :shadow="false"
      class="flex-row! flex-wrap lg:flex-nowrap items-start"
      :class="[row.tone ? `tone--${row.tone}` : null]"
    >
      <div class="c-reminder-card__dot self-center"></div>
      <div class="c-reminder-card__info">
        <span class="c-reminder-card__row-title">{{ row.title }}</span>
        <span
          v-if="row.subtitle"
          class="c-reminder-card__subtitle"
        >{{ row.subtitle }}</span>
      </div>
      <span
        v-if="row.meta"
        class="c-reminder-card__meta"
      >{{ row.meta }}</span>
      <span
        v-if="row.emphasis"
        :class="['c-reminder-card__emphasis', row.tone ? `tone--${row.tone}` : null]"
      >{{ row.emphasis }}</span>
      <div
        v-if="$slots.actions"
        class="c-reminder-card__actions"
      >
        <slot
          name="actions"
          :row="row"
        ></slot>
      </div>
    </CCard>
  </div>
</template>

<style scoped>
.c-reminder-card__title {
  font-size: var(--text-lg);
  font-weight: var(--fw-semibold);
  color: var(--color-on-surface);
}

.c-reminder-card__dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
  flex-shrink: 0;
}

.c-reminder-card__info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.c-reminder-card__row-title {
  white-space: nowrap;
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
}

.c-reminder-card__subtitle {
  font-size: var(--text-2xs);
}

.c-reminder-card__meta {
  font-size: var(--text-sm);
  flex-shrink: 0;
}

.c-reminder-card__emphasis {
  font-size: var(--text-xs);
  font-weight: var(--fw-semibold);
  color: var(--tone-fg, var(--color-on-surface-variant));
}

.c-reminder-card__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-left: auto;
}

</style>