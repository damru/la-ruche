<script setup lang="ts">
// Composant molécule — CalendarListMode
// Liste chronologique des événements groupée par jour.
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CEmptyState from '@/components/atoms/CEmptyState.vue'
import type { CalendarEvent } from '@/types/domain'

const props = defineProps<{
  events: CalendarEvent[]
}>()

const router = useRouter()
const { t } = useI18n()
const { formatDate } = useFormatters()

const sourceVariant: Record<CalendarEvent['source'], 'accent' | 'default' | 'warn' | 'err' | 'valid' | 'primary'> = {
  meeting: 'primary',
  contract: 'warn',
  task: 'valid',
  claim: 'err',
  maintenance: 'default',
  manual: 'accent',
}

const groupedByDay = computed(() => {
  const groups = new Map<string, CalendarEvent[]>()
  for (const event of props.events) {
    const list = groups.get(event.date) ?? []
    list.push(event)
    groups.set(event.date, list)
  }
  return [...groups.entries()].sort(([left], [right]) => left.localeCompare(right))
})

function goToEvent(event: CalendarEvent) {
  router.push(event.link)
}
</script>

<template>
  <CEmptyState
    v-if="events.length === 0"
    :title="t('calendar.empty')"
  />
  <div
    v-else
    class="flex flex-col gap-4"
  >
    <div
      v-for="[date, dayEvents] in groupedByDay"
      :key="date"
    >
      <h3 class="text-sm font-semibold text-on-surface-variant mb-2">
        {{ formatDate(date, 'long') }}
      </h3>
      <div class="flex flex-col gap-2">
        <CCard
          v-for="event in dayEvents"
          :key="event.id"
          :level="3"
          class="cursor-pointer"
          @click="goToEvent(event)"
        >
          <div class="flex items-center gap-2 justify-between">
            <span class="font-medium">{{ event.title }}</span>
            <CBadge
              :variant="sourceVariant[event.source]"
              dot
            >
              {{ t(`calendar.sources.${event.source}`) }}
            </CBadge>
          </div>
        </CCard>
      </div>
    </div>
  </div>
</template>
