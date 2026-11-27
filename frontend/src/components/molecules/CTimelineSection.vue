<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CTimelineList from '@/components/molecules/CTimelineList.vue'
import CEmptyState from '@/components/atoms/CEmptyState.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CIconRefresh from '@/components/atoms/CIconRefresh.svg'
import type { TimelineEvent } from '@/types/domain'

withDefaults(defineProps<{
  events: TimelineEvent[]
  loading: boolean
  error: boolean
  linkTasks?: boolean
  currentTaskId?: string
  currentReportId?: string
}>(), { linkTasks: true, currentTaskId: undefined, currentReportId: undefined })

defineEmits<{ retry: [] }>()

const { t } = useI18n()
</script>

<template>
  <CPageLoader v-if="loading" />
  <CCard
    v-else-if="error"
    :level="3"
    class="flex flex-col items-start gap-3"
  >
    <p class="text-sm text-on-surface-variant m-0">
      {{ t('timeline.error') }}
    </p>
    <CBtn
      variant="ghost"
      size="sm"
      :icon="CIconRefresh"
      :title="t('timeline.retry')"
      @click="$emit('retry')"
    />
  </CCard>
  <CEmptyState
    v-else-if="!events.length"
    :title="t('timeline.empty')"
    size="compact"
  />
  <CTimelineList
    v-else
    :events="events"
    :link-tasks="linkTasks"
    :current-task-id="currentTaskId"
    :current-report-id="currentReportId"
  />
</template>
