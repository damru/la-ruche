<script setup lang="ts">
// Composant molécule — CalendarGrid
// Grille réutilisée en vue mensuelle (6 semaines) et hebdomadaire (7 jours).
// Chaque événement = badge coloré par source + titre tronqué, clic → lien source (AC-CAL-001-B).
import { useRouter } from 'vue-router'
import CBadge from '@/components/atoms/CBadge.vue'
import type { CalendarEvent } from '@/types/domain'

const props = defineProps<{
  days: Date[]
  events: CalendarEvent[]
  currentMonth?: number
}>()

const router = useRouter()

const sourceVariant: Record<CalendarEvent['source'], 'accent' | 'default' | 'warn' | 'err' | 'valid' | 'primary'> = {
  meeting: 'primary',
  contract: 'warn',
  task: 'valid',
  claim: 'err',
  maintenance: 'default',
  manual: 'accent',
}

function dayKey(day: Date): string {
  return day.toISOString().slice(0, 10)
}

function eventsForDay(day: Date): CalendarEvent[] {
  const key = dayKey(day)
  return props.events.filter(event => event.date === key)
}

function isOtherMonth(day: Date): boolean {
  return props.currentMonth !== undefined && day.getMonth() !== props.currentMonth
}

function goToEvent(event: CalendarEvent) {
  router.push(event.link)
}
</script>

<template>
  <div class="calendar-grid">
    <div
      v-for="day in days"
      :key="dayKey(day)"
      class="calendar-grid__cell"
      :class="{ 'calendar-grid__cell--muted': isOtherMonth(day) }"
    >
      <span class="calendar-grid__day-number">{{ day.getDate() }}</span>
      <div class="calendar-grid__events">
        <button
          v-for="event in eventsForDay(day).slice(0, 3)"
          :key="event.id"
          type="button"
          class="calendar-grid__event"
          @click="goToEvent(event)"
        >
          <CBadge
            :variant="sourceVariant[event.source]"
            dot
          >
            {{ event.title }}
          </CBadge>
        </button>
        <span
          v-if="eventsForDay(day).length > 3"
          class="calendar-grid__more"
        >
          +{{ eventsForDay(day).length - 3 }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: var(--space-2);
}

.calendar-grid__cell {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  min-height: 96px;
  padding: var(--space-2);
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-low);
}

.calendar-grid__cell--muted {
  opacity: 0.45;
}

.calendar-grid__day-number {
  font-size: var(--text-xs);
  font-weight: var(--fw-medium);
  color: var(--color-on-surface-variant);
}

.calendar-grid__events {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.calendar-grid__event {
  display: block;
  width: 100%;
  text-align: left;
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
}

.calendar-grid__event :deep(.c-badge) {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: block;
}

.calendar-grid__more {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
}
</style>
