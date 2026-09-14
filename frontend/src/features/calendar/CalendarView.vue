<script setup lang="ts">
// Vue principale — CalendarView (EP-014-S01)
// Toggle mensuel/hebdo/liste, navigation de période, fetch sur changement de plage.
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePageLoader } from '@/composables/usePageLoader'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTabs, { type Tab } from '@/components/atoms/CTabs.vue'
import CalendarGrid from './CalendarGrid.vue'
import CalendarListMode from './CalendarListMode.vue'
import { calendarApi } from '@/api'
import type { CalendarEvent } from '@/types/domain'

const { t } = useI18n()

type Mode = 'month' | 'week' | 'list'
const mode = ref<Mode>('month')
const cursor = ref(new Date())
const events = ref<CalendarEvent[]>([])

const modeTabs = computed<Tab[]>(() => [
  { value: 'month', label: t('calendar.modes.month') },
  { value: 'week', label: t('calendar.modes.week') },
  { value: 'list', label: t('calendar.modes.list') },
])

function toDateKey(date: Date): string {
  return date.toISOString().slice(0, 10)
}

function startOfWeek(date: Date): Date {
  const result = new Date(date)
  const day = (result.getDay() + 6) % 7 // Monday = 0
  result.setDate(result.getDate() - day)
  result.setHours(0, 0, 0, 0)
  return result
}

// 6-week grid anchored on the Monday preceding the 1st of the month.
const monthDays = computed<Date[]>(() => {
  const first = new Date(cursor.value.getFullYear(), cursor.value.getMonth(), 1)
  const start = startOfWeek(first)
  return Array.from({ length: 42 }, (_, i) => {
    const day = new Date(start)
    day.setDate(start.getDate() + i)
    return day
  })
})

const weekDays = computed<Date[]>(() => {
  const start = startOfWeek(cursor.value)
  return Array.from({ length: 7 }, (_, i) => {
    const day = new Date(start)
    day.setDate(start.getDate() + i)
    return day
  })
})

const range = computed(() => {
  if (mode.value === 'week') {
    const days = weekDays.value
    return { from: toDateKey(days[0]), to: toDateKey(days[6]) }
  }
  if (mode.value === 'list') {
    const from = new Date(cursor.value)
    const to = new Date(cursor.value)
    to.setDate(to.getDate() + 30)
    return { from: toDateKey(from), to: toDateKey(to) }
  }
  const days = monthDays.value
  return { from: toDateKey(days[0]), to: toDateKey(days[days.length - 1]) }
})

async function loadEvents() {
  const response = await calendarApi.events(range.value.from, range.value.to)
  events.value = response.data
}

const { loading, error, reload } = usePageLoader(loadEvents)
onMounted(reload)
watch(range, reload)

function goToday() {
  cursor.value = new Date()
}

function goPrevious() {
  const next = new Date(cursor.value)
  if (mode.value === 'week') next.setDate(next.getDate() - 7)
  else if (mode.value === 'list') next.setDate(next.getDate() - 30)
  else next.setMonth(next.getMonth() - 1)
  cursor.value = next
}

function goNext() {
  const next = new Date(cursor.value)
  if (mode.value === 'week') next.setDate(next.getDate() + 7)
  else if (mode.value === 'list') next.setDate(next.getDate() + 30)
  else next.setMonth(next.getMonth() + 1)
  cursor.value = next
}

const periodLabel = computed(() =>
  cursor.value.toLocaleDateString('fr-FR', { month: 'long', year: 'numeric' }),
)
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          <span>{{ t('calendar.title') }}</span>
        </div>
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
          <CEyebrow>{{ t('page.eyebrows.calendar') }}</CEyebrow>
          <CHeading type="header">
            {{ t('calendar.title') }}
          </CHeading>
        </header>

        <div class="flex items-center justify-between gap-3 flex-wrap">
          <CTabs
            v-model="mode"
            variant="filter"
            :tabs="modeTabs"
          />
          <div
            v-if="mode !== 'list'"
            class="flex items-center gap-2"
          >
            <CBtn
              variant="ghost"
              size="sm"
              :title="t('calendar.previous')"
              @click="goPrevious"
            />
            <CBtn
              variant="secondary"
              size="sm"
              :title="t('calendar.today')"
              @click="goToday"
            />
            <CBtn
              variant="ghost"
              size="sm"
              :title="t('calendar.next')"
              @click="goNext"
            />
            <span class="text-sm font-medium capitalize">{{ periodLabel }}</span>
          </div>
        </div>

        <CalendarGrid
          v-if="mode === 'month'"
          :days="monthDays"
          :events="events"
          :current-month="cursor.getMonth()"
        />
        <CalendarGrid
          v-else-if="mode === 'week'"
          :days="weekDays"
          :events="events"
        />
        <CalendarListMode
          v-else
          :events="events"
        />
      </template>
    </div>
  </div>
</template>
