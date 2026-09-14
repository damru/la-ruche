<script setup lang="ts">
// CalendarListMode — chronological list grouped by day (EP-014-S01, AC-CAL-001-C).

import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import CCard from '@/components/molecules/CCard.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CEmptyState from '@/components/atoms/CEmptyState.vue'
import type { CalendarEvent, CalendarSource } from '@/types/domain'

const props = defineProps<{
  events: CalendarEvent[]
}>()

const router = useRouter()
const { t } = useI18n()

const SOURCE_BADGE_VARIANT: Record<CalendarSource, 'primary' | 'accent' | 'warn' | 'err' | 'default' | 'valid'> = {
  meeting: 'primary',
  contract: 'warn',
  task: 'valid',
  claim: 'err',
  maintenance: 'accent',
  manual: 'default',
}

interface DayGroup {
  date: string
  events: CalendarEvent[]
}

const groups = computed<DayGroup[]>(() => {
  const map = new Map<string, CalendarEvent[]>()
  for (const event of props.events) {
    const list = map.get(event.date) ?? []
    list.push(event)
    map.set(event.date, list)
  }
  return Array.from(map.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([date, events]) => ({ date, events }))
})

function formatDay(date: string): string {
  return new Date(`${date}T00:00:00`).toLocaleDateString('fr-FR', {
    weekday: 'long',
    day: 'numeric',
    month: 'long',
    year: 'numeric',
  })
}

function onEventClick(event: CalendarEvent) {
  router.push(event.link)
}
</script>

<template>
  <CEmptyState
    v-if="groups.length === 0"
    :title="t('calendar.empty.title')"
    :description="t('calendar.empty.description')"
  />
  <div
    v-else
    class="flex flex-col gap-4"
  >
    <div
      v-for="group in groups"
      :key="group.date"
      class="flex flex-col gap-2"
    >
      <p class="label-md text-on-surface-variant uppercase">
        {{ formatDay(group.date) }}
      </p>
      <CCard
        :level="3"
        gap="sm"
      >
        <button
          v-for="event in group.events"
          :key="event.id"
          type="button"
          class="flex items-center gap-3 text-left w-full"
          @click="onEventClick(event)"
        >
          <CBadge :variant="SOURCE_BADGE_VARIANT[event.source]">
            {{ t(`calendar.sources.${event.source}`) }}
          </CBadge>
          <span class="body-sm text-on-surface">{{ event.title }}</span>
          <span
            v-if="event.time"
            class="body-sm text-on-surface-variant ml-auto"
          >
            {{ event.time.slice(0, 5) }}
          </span>
        </button>
      </CCard>
    </div>
  </div>
</template>
