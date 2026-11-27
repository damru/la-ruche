<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
import { useTimelineFeed } from '@/composables/useTimelineFeed'
import { taskPriorityVariant, taskStatusVariant } from '@/composables/useStatusLabels'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CTimelineSection from '@/components/molecules/CTimelineSection.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconArrowLeft from '@/components/atoms/CIconArrowLeft.svg'
import CIconRefresh from '@/components/atoms/CIconRefresh.svg'
import CPageHeader from '@/components/molecules/CPageHeader.vue'
import CTopbarBreadcrumb from '@/components/molecules/CTopbarBreadcrumb.vue'
import { contractorsApi, maintenanceApi, tasksApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import type { Contractor, MaintenanceEntry, Task } from '@/types/domain'
import { TASK_STATUS_CLOSED } from '@/utils/constants'

const { t } = useI18n()
const { formatDate, formatRelative, formatCurrency } = useFormatters()
const route = useRoute()
const router = useRouter()

const taskId = ref(route.params.id as string)
const buildingName = ref('')
const task = ref<Task | null>(null)
const contractor = ref<Contractor | null>(null)
const interventions = ref<MaintenanceEntry[]>([])

const {
  events: timeline,
  loading: timelineLoading,
  error: timelineError,
  reload: reloadTimeline,
} = useTimelineFeed(async () => (await tasksApi.timeline(taskId.value)).data)

async function loadData() {
  const [building, taskRes, entriesRes] = await Promise.all([
    useBuildingStore().fetch(),
    tasksApi.get(taskId.value),
    maintenanceApi.list(),
  ])
  buildingName.value = building.name
  task.value = taskRes.data

  if (taskRes.data.contractor_id) {
    const contractorRes = await contractorsApi.get(taskRes.data.contractor_id)
    contractor.value = contractorRes.data
  } else {
    contractor.value = null
  }

  interventions.value = entriesRes.data
    .filter(entry => entry.task_id === taskId.value)
    .sort((left, right) => left.intervention_date.localeCompare(right.intervention_date))
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(() => {
  reload()
  reloadTimeline()
})

watch(() => route.params.id, id => {
  taskId.value = id as string
  reload()
  reloadTimeline()
})

</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <CTopbarBreadcrumb
          :crumbs="[
            { label: buildingName || '…' },
            { label: t('nav.tasks'), to: { name: 'tasks' } },
            { label: task?.title ?? '…' },
          ]"
        />
      </template>
      <template #right>
        <CBtn
          variant="ghost"
          size="sm"
          :icon="CIconArrowLeft"
          :title="t('common.back')"
          @click="router.push({ name: 'tasks' })"
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
      <template v-else-if="task">
        <CPageHeader
          :title="task.title"
        >
          <template #eyebrow>
            <CBadge :variant="taskPriorityVariant(task.priority)">
              {{ t(`tasks.priorities.${task.priority}`) }}
            </CBadge>
            <CBadge
              :variant="taskStatusVariant(task.status)"
              dot
            >
              {{ t(`tasks.statuses.${task.status}`) }}
            </CBadge>
            {{ t('tasks.lifecycle.title') }}
          </template>
          <p
            v-if="task.description"
            class="body-md text-on-surface-variant m-0"
          >
            {{ task.description }}
          </p>
          <div class="text-2xs text-on-surface-variant mt-1">
            <span>{{ t('tasks.lifecycle.createdRelative', { relative: formatRelative(task.created_at) }) }}</span>
            <span v-if="task.closed_at">
              · {{ t('tasks.closedAt', { date: formatDate(task.closed_at) }) }}
            </span>
            <span v-if="contractor || task.contractor_name">
              · <RouterLink
                v-if="contractor"
                :to="{ name: 'contractor-detail', params: { id: contractor.id } }"
                class="text-primary font-medium hover:underline"
              >🔧 {{ contractor.name }}</RouterLink>
              <span
                v-else
                class="text-primary font-medium"
              >🔧 {{ task.contractor_name }}</span>
            </span>
          </div>
        </CPageHeader>

        <h2 class="text-xl font-semibold text-on-surface-variant mt-0 mb-4">
          {{ t('tasks.lifecycle.title') }}
        </h2>

        <CTimelineSection
          :events="timeline"
          :loading="timelineLoading"
          :error="timelineError"
          :current-task-id="taskId"
          @retry="reloadTimeline"
        />

        <section
          v-if="interventions.length"
          class="mt-6"
        >
          <h2 class="text-xl font-semibold text-on-surface-variant mt-0 mb-4">
            {{ t('tasks.interventions') }}
          </h2>
          <div class="flex flex-col gap-2">
            <CCard
              v-for="entry in interventions"
              :key="entry.id"
              :level="3"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="flex flex-col gap-1 min-w-0">
                  <span class="text-sm font-medium text-on-surface">{{ entry.title }}</span>
                  <span
                    v-if="entry.vendor_name || entry.cost != null"
                    class="text-xs text-on-surface-variant"
                  >
                    <span v-if="entry.vendor_name">🔧 {{ entry.vendor_name }}</span>
                    <span v-if="entry.cost != null"> · {{ formatCurrency(entry.cost) }}</span>
                  </span>
                  <span
                    v-if="entry.description"
                    class="text-xs text-on-surface-variant"
                  >
                    {{ entry.description }}
                  </span>
                </div>
                <span class="text-xs text-on-surface-variant whitespace-nowrap">
                  {{ formatDate(entry.intervention_date) }}
                </span>
              </div>
            </CCard>
          </div>
        </section>

        <div
          v-if="task.status === TASK_STATUS_CLOSED"
          class="flex items-center gap-1 text-xs text-on-surface-variant mt-2"
        >
          <CIconRefresh class="size-4" />
          {{ t('tasks.lifecycle.closedNote') }}
        </div>
      </template>
    </div>
  </div>
</template>
