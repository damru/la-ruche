<script setup lang="ts">
// DashboardAgendaCard — EP-014-S05. Reads the same `/calendar/events` feed as
// S01, truncated to the next 5 events over 30 days. Fetch is delegated to the
// parent (DashboardView's `Promise.allSettled`, same `valueOr` pattern as the
// other cards) — this component only renders `events`.
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import type { CalendarEvent, CalendarSource } from '@/types/domain'

const props = defineProps<{
  events: CalendarEvent[]
}>()

const router = useRouter()
const { t } = useI18n()
const { formatRelative } = useFormatters()

const SOURCE_BADGE_VARIANT: Record<CalendarSource, 'primary' | 'accent' | 'warn' | 'err' | 'default' | 'valid'> = {
  meeting: 'primary',
  contract: 'warn',
  task: 'valid',
  claim: 'err',
  maintenance: 'accent',
  manual: 'default',
}

const sorted = computed(() =>
  [...props.events].sort((a, b) => a.date.localeCompare(b.date) || (a.time ?? '').localeCompare(b.time ?? '')),
)
const visibleEvents = computed(() => sorted.value.slice(0, 5))
const extraCount = computed(() => Math.max(0, sorted.value.length - 5))

function onEventClick(event: CalendarEvent) {
  router.push(event.link)
}
</script>

<template>
  <CCard>
    <CSectionHeader>
      <CHeading type="card">
        {{ t('dashboard.agenda.title') }}
      </CHeading>
      <template #actions>
        <CSectionLink to="/calendar">
          {{ t('dashboard.agenda.seeCalendar') }}
        </CSectionLink>
      </template>
    </CSectionHeader>

    <CEmptyState
      v-if="visibleEvents.length === 0"
      :title="t('dashboard.agenda.emptyTitle')"
      :description="t('dashboard.agenda.emptyDescription')"
    />
    <div
      v-else
      class="agenda-list"
    >
      <button
        v-for="event in visibleEvents"
        :key="event.id"
        type="button"
        class="agenda-item"
        @click="onEventClick(event)"
      >
        <CBadge :variant="SOURCE_BADGE_VARIANT[event.source]">
          {{ t(`calendar.sources.${event.source}`) }}
        </CBadge>
        <span class="agenda-title">{{ event.title }}</span>
        <span class="agenda-date">{{ formatRelative(event.date) }}</span>
      </button>
      <RouterLink
        v-if="extraCount > 0"
        to="/calendar"
        class="agenda-more"
      >
        {{ t('dashboard.agenda.moreEvents', { count: extraCount }) }}
      </RouterLink>
    </div>
  </CCard>
</template>

<style scoped>
.agenda-list { display: flex; flex-direction: column; gap: var(--space-1); }
.agenda-item {
  display: flex; align-items: center; gap: 10px; width: 100%; text-align: left;
  padding: 10px var(--space-3); border-radius: var(--radius-xl); background: none; border: none; cursor: pointer;
  transition: background var(--transition-fast);
}
.agenda-item:hover { background: var(--color-surface-container-low); }
.agenda-title { flex: 1; min-width: 0; font-size: var(--text-xs); font-weight: var(--fw-medium); color: var(--color-on-surface); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.agenda-date { font-size: var(--text-2xs); color: var(--color-on-surface-variant); flex-shrink: 0; }
.agenda-more { display: block; text-align: center; padding: var(--space-2); font-size: var(--text-xs); font-weight: var(--fw-medium); color: var(--color-primary); }
</style>
