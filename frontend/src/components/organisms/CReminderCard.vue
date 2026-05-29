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
    <div class="flex flex-col gap-2">
      <CCard
        v-for="row in rows"
        :key="row.id"
        :level="4"
        :shadow="false"
        gap="sm"
        class="flex-row! flex-wrap lg:flex-nowrap items-center"
        :class="[row.tone ? `tone--${row.tone}` : null]"
      >
        <CIconBell class="icon-md shrink-0"/>
        <div class="flex flex-col flex-1 min-w-0">
          <span class="c-reminder-card__row-title">{{ row.title }}</span>
          <span
            v-if="row.subtitle"
            class="c-reminder-card__subtitle"
          >{{ row.subtitle }}</span>
        </div>
        <div
          v-if="row.meta || row.emphasis"
          class="flex flex-col shrink-0 items-end"
        >
          <span
            v-if="row.meta"
            class="c-reminder-card__meta"
          >{{ row.meta }}</span>
          <span
            v-if="row.emphasis"
            :class="['c-reminder-card__emphasis', row.tone ? `tone--${row.tone}` : null]"
          >{{ row.emphasis }}</span>
        </div>
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
  </div>
</template>

<style scoped>
.c-reminder-card__row-title {
  white-space: nowrap;
  font-weight: var(--fw-medium);
}

.c-reminder-card__subtitle {
  font-size: var(--text-2xs);
}

.c-reminder-card__meta {
  font-size: var(--text-xs);
  flex-shrink: 0;
}

.c-reminder-card__emphasis {
  font-size: var(--text-sm);
  font-weight: var(--fw-semibold);
  color: var(--tone-fg, var(--color-on-surface-variant));
}

.c-reminder-card__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

</style>