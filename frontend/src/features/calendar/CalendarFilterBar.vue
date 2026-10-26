<script setup lang="ts">
// CalendarFilterBar — toggle chips per event type, colored like the source
// badges. Only the sources visible for the caller's role are offered.

import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import CBadge from '@/components/atoms/CBadge.vue'
import { usePermissions } from '@/composables/usePermissions'
import { CALENDAR_SOURCES, SOURCE_BADGE_VARIANT } from './sources'
import type { CalendarSource } from '@/types/domain'

const props = defineProps<{
  modelValue: CalendarSource[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: CalendarSource[]]
}>()

const { t } = useI18n()

const perms = usePermissions()

const filters = computed(() =>
  CALENDAR_SOURCES.filter(source => perms.canViewCalendarSource(source)),
)

function isActive(source: CalendarSource): boolean {
  return props.modelValue.includes(source)
}

function toggle(source: CalendarSource) {
  if (isActive(source)) {
    emit('update:modelValue', props.modelValue.filter(s => s !== source))
  } else {
    emit('update:modelValue', [...props.modelValue, source])
  }
}
</script>

<template>
  <div
    class="flex flex-wrap items-center gap-2"
    role="group"
    :aria-label="t('calendar.filters.label')"
  >
    <button
      v-for="source in filters"
      :key="source"
      type="button"
      :aria-pressed="isActive(source)"
      class="cursor-pointer transition-opacity"
      :class="isActive(source) ? 'opacity-100' : 'opacity-40'"
      @click="toggle(source)"
    >
      <CBadge :variant="SOURCE_BADGE_VARIANT[source]">
        {{ t(`calendar.sources.${source}`) }}
      </CBadge>
    </button>
  </div>
</template>
