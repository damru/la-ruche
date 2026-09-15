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
      class="flex flex-col gap-1"
    >
      <button
        v-for="event in visibleEvents"
        :key="event.id"
        type="button"
        class="flex items-center gap-2.5 w-full text-left px-3 py-2.5 rounded-xl border-none bg-transparent cursor-pointer transition-colors duration-150 hover:bg-surface-container-low"
        @click="onEventClick(event)"
      >
        <CBadge :variant="SOURCE_BADGE_VARIANT[event.source]">
          {{ t(`calendar.sources.${event.source}`) }}
        </CBadge>
        <span class="flex-1 min-w-0 text-xs font-medium text-on-surface overflow-hidden text-ellipsis whitespace-nowrap">{{ event.title }}</span>
        <span class="text-2xs text-on-surface-variant shrink-0">{{ formatRelative(event.date) }}</span>
      </button>
      <RouterLink
        v-if="extraCount > 0"
        to="/calendar"
        class="block text-center p-2 text-xs font-medium text-primary"
      >
        {{ t('dashboard.agenda.moreEvents', { count: extraCount }) }}
      </RouterLink>
    </div>
  </CCard>
</template>
