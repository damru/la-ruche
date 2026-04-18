<script setup lang="ts">
// CalendarView — EP-014-S01. Monthly / weekly / agenda toggle with period nav.
// Visibility rules are enforced server-side (`/calendar/events` filters by
// caller role); this view only renders whatever events come back.
// EP-014-S03: manual event create/edit/delete (Manager/BoardMember only).

import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { calendarApi } from '@/api'
import { usePageLoader } from '@/composables/usePageLoader'
import { useAuthStore } from '@/stores/auth'
import { useBuildingStore } from '@/stores/building'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CMetaLine from '@/components/molecules/CMetaLine.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'
import CSeparator from '@/components/atoms/CSeparator.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconArrowLeft from '@/components/atoms/CIconArrowLeft.svg'
import CIconArrowRight from '@/components/atoms/CIconArrowRight.svg'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CalendarGrid from './CalendarGrid.vue'
import CalendarAgendaMode from './CalendarAgendaMode.vue'
import EventFormModal from './EventFormModal.vue'
import EventDetailModal from './EventDetailModal.vue'
import type { CalendarEvent, ManualCalendarEvent } from '@/types/domain'

type Mode = 'month' | 'week' | 'agenda'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const mode = ref<Mode>('month')
const anchor = ref(new Date())
const events = ref<CalendarEvent[]>([])
const buildingName = ref('')

const canManage = auth.hasAnyRole(['manager', 'board_member'])

const detailOpen = ref(false)
const detailEventId = ref<string | null>(null)
const formOpen = ref(false)
const editingEvent = ref<ManualCalendarEvent | null>(null)
const exporting = ref(false)

function periodRange(date: Date, currentMode: Mode): { from: Date; to: Date } {
  if (currentMode === 'week') {
    const day = (date.getDay() + 6) % 7 // Monday = 0
    const from = new Date(date.getFullYear(), date.getMonth(), date.getDate() - day)
    const to = new Date(from)
    to.setDate(from.getDate() + 6)
    return { from, to }
  }
  // month + agenda share the same monthly fetch window
  const from = new Date(date.getFullYear(), date.getMonth(), 1)
  const to = new Date(date.getFullYear(), date.getMonth() + 1, 0)
  return { from, to }
}

function toIsoDate(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}

async function loadData() {
  const { from, to } = periodRange(anchor.value, mode.value)
  const [building, response] = await Promise.all([
    useBuildingStore().fetch(),
    calendarApi.events(toIsoDate(from), toIsoDate(to)),
  ])
  buildingName.value = building.name
  events.value = response.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

const periodLabel = computed(() => {
  if (mode.value === 'week') {
    const { from, to } = periodRange(anchor.value, mode.value)
    return `${from.toLocaleDateString('fr-FR', { day: 'numeric', month: 'short' })} – ${to.toLocaleDateString('fr-FR', { day: 'numeric', month: 'short', year: 'numeric' })}`
  }
  return anchor.value.toLocaleDateString('fr-FR', { month: 'long', year: 'numeric' })
})

function shiftPeriod(delta: number) {
  const next = new Date(anchor.value)
  if (mode.value === 'week') {
    next.setDate(next.getDate() + delta * 7)
  } else {
    next.setMonth(next.getMonth() + delta)
  }
  anchor.value = next
  reload()
}

function onModeChange() {
  reload()
}

function onShowDay(date: string) {
  mode.value = 'agenda'
  anchor.value = new Date(`${date}T00:00:00`)
  reload()
}

// Manual events navigate to `/calendar?event={id}` (db/calendar.rs link).
watch(() => route.query.event, (eventId) => {
  if (typeof eventId === 'string') {
    detailEventId.value = eventId
    detailOpen.value = true
  }
}, { immediate: true })

function closeDetail(open: boolean) {
  detailOpen.value = open
  if (!open && route.query.event) {
    router.replace({ query: { ...route.query, event: undefined } })
  }
}

function openCreate() {
  editingEvent.value = null
  formOpen.value = true
}

function openEdit(event: ManualCalendarEvent) {
  editingEvent.value = event
  detailOpen.value = false
  formOpen.value = true
}

function onSaved() {
  reload()
}

function onDeleted() {
  reload()
}

async function exportCalendar() {
  if (exporting.value) return
  exporting.value = true
  try {
    const { from, to } = periodRange(anchor.value, mode.value)
    const response = await calendarApi.exportIcs(toIsoDate(from), toIsoDate(to))
    const url = URL.createObjectURL(response.data)
    const link = document.createElement('a')
    link.href = url
    link.download = 'laruche-calendar.ics'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
  } catch {
    // Download errors are non-critical in this context.
  } finally {
    exporting.value = false
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName || '...' }}
          <CIconChevronRight aria-hidden="true" />
          <span>{{ t('calendar.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canManage"
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('calendar.newEvent')"
          shrink="sm"
          @click="openCreate"
        />
      </template>
    </CTopbar>

    <div class="view-container">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else>
        <header class="view-header">
          <CEyebrow>
            {{ t('page.eyebrows.calendar') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('calendar.title') }}
          </CHeading>
          <CMetaLine>
            <CMetaItem variant="sub">
              {{ periodLabel }}
            </CMetaItem>
            <CSeparator />
            <CMetaItem variant="sub">
              {{ t('calendar.count', events.length) }}
            </CMetaItem>
          </CMetaLine>
        </header>

        <CTabs
          v-model="mode"
          variant="filter"
          :tabs="(['month', 'week', 'agenda'] as const).map(m => ({ value: m, label: t(`calendar.modes.${m}`) }))"
          @update:model-value="onModeChange"
        />

        <div class="flex items-center justify-between gap-3">
          <div
            v-if="mode !== 'agenda'"
            class="flex items-center gap-3"
          >
            <CBtn
              variant="icon"
              size="sm"
              :icon="CIconArrowLeft"
              :title="t('calendar.previousPeriod')"
              :aria-label="t('calendar.previousPeriod')"
              @click="shiftPeriod(-1)"
            />
            <span class="title-sm">{{ periodLabel }}</span>
            <CBtn
              variant="icon"
              size="sm"
              :icon="CIconArrowRight"
              :title="t('calendar.nextPeriod')"
              :aria-label="t('calendar.nextPeriod')"
              @click="shiftPeriod(1)"
            />
          </div>
          <CBtn
            variant="ghost"
            size="sm"
            :icon="CIconDownload"
            :title="t('calendar.export')"
            :aria-label="t('calendar.export')"
            :loading="exporting"
            @click="exportCalendar"
          />
        </div>

        <CalendarGrid
          v-if="mode === 'month' || mode === 'week'"
          :mode="mode"
          :anchor="anchor"
          :events="events"
          @show-day="onShowDay"
        />
        <CalendarAgendaMode
          v-else
          :events="events"
        />
      </template>
    </div>

    <EventDetailModal
      :open="detailOpen"
      :event-id="detailEventId"
      @update:open="closeDetail"
      @edit="openEdit"
      @deleted="onDeleted"
    />
    <EventFormModal
      :open="formOpen"
      :event="editingEvent"
      @update:open="formOpen = $event"
      @saved="onSaved"
    />
  </div>
</template>
