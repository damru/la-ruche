<script setup lang="ts">
// CalendarGrid — shared month/week grid (EP-014-S01, AC-CAL-001-A/B).
// Renders 6 weeks (monthly) or 1 week (weekly). Each event is a colored
// badge by source; clicking navigates to `event.link`. Beyond 3 events in a
// monthly cell, a "+N" link routes to the agenda mode for that day.

import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import CBadge from '@/components/atoms/CBadge.vue'
import type { CalendarEvent, CalendarSource } from '@/types/domain'

const MAX_EVENTS_PER_CELL = 3

const props = defineProps<{
  mode: 'month' | 'week'
  /** Any date within the period to render (month or week). */
  anchor: Date
  events: CalendarEvent[]
}>()

const emit = defineEmits<{
  (e: 'show-day', date: string): void
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

function toDateKey(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}

function startOfWeek(date: Date): Date {
  const d = new Date(date.getFullYear(), date.getMonth(), date.getDate())
  const day = (d.getDay() + 6) % 7 // Monday = 0
  d.setDate(d.getDate() - day)
  return d
}

const days = computed<Date[]>(() => {
  if (props.mode === 'week') {
    const start = startOfWeek(props.anchor)
    return Array.from({ length: 7 }, (_, i) => {
      const d = new Date(start)
      d.setDate(start.getDate() + i)
      return d
    })
  }

  // Monthly: 6 full weeks (42 days) starting on the Monday of the first week.
  const monthStart = new Date(props.anchor.getFullYear(), props.anchor.getMonth(), 1)
  const gridStart = startOfWeek(monthStart)
  return Array.from({ length: 42 }, (_, i) => {
    const d = new Date(gridStart)
    d.setDate(gridStart.getDate() + i)
    return d
  })
})

const eventsByDay = computed<Map<string, CalendarEvent[]>>(() => {
  const map = new Map<string, CalendarEvent[]>()
  for (const event of props.events) {
    const list = map.get(event.date) ?? []
    list.push(event)
    map.set(event.date, list)
  }
  return map
})

function eventsFor(date: Date): CalendarEvent[] {
  return eventsByDay.value.get(toDateKey(date)) ?? []
}

function isCurrentMonth(date: Date): boolean {
  return date.getMonth() === props.anchor.getMonth()
}

function isToday(date: Date): boolean {
  const today = new Date()
  return date.toDateString() === today.toDateString()
}

function onEventClick(event: CalendarEvent) {
  router.push(event.link)
}

function onShowMore(date: Date) {
  emit('show-day', toDateKey(date))
}

const weekdayLabels = computed(() => [
  t('calendar.weekdays.mon'),
  t('calendar.weekdays.tue'),
  t('calendar.weekdays.wed'),
  t('calendar.weekdays.thu'),
  t('calendar.weekdays.fri'),
  t('calendar.weekdays.sat'),
  t('calendar.weekdays.sun'),
])
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="grid grid-cols-7 gap-1">
      <span
        v-for="label in weekdayLabels"
        :key="label"
        class="text-center text-xs font-medium text-on-surface-variant uppercase"
      >
        {{ label }}
      </span>
    </div>
    <div
      class="grid grid-cols-7 gap-1"
      :class="mode === 'week' ? 'grid-rows-1' : 'grid-rows-6'"
    >
      <div
        v-for="day in days"
        :key="toDateKey(day)"
        class="flex flex-col gap-1 rounded-lg border border-outline-variant p-2 min-h-24"
        :class="[
          mode === 'month' && !isCurrentMonth(day) ? 'opacity-40' : '',
          isToday(day) ? 'border-primary bg-primary-container/40' : '',
        ]"
      >
        <span class="text-xs font-medium text-on-surface-variant">{{ day.getDate() }}</span>
        <div class="flex flex-col gap-1">
          <button
            v-for="event in eventsFor(day).slice(0, MAX_EVENTS_PER_CELL)"
            :key="event.id"
            type="button"
            class="text-left"
            @click="onEventClick(event)"
          >
            <CBadge :variant="SOURCE_BADGE_VARIANT[event.source]">
              {{ event.title }}
            </CBadge>
          </button>
          <button
            v-if="eventsFor(day).length > MAX_EVENTS_PER_CELL"
            type="button"
            class="text-left text-xs font-medium text-on-surface-variant hover:text-on-surface"
            @click="onShowMore(day)"
          >
            {{ t('calendar.moreEvents', { count: eventsFor(day).length - MAX_EVENTS_PER_CELL }) }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
