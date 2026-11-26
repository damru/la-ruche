<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
import { usePermissions } from '@/composables/usePermissions'
import { useReportLabels } from '@/composables/useReportLabels'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CCard from '@/components/molecules/CCard.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'
import CMetaLine from '@/components/molecules/CMetaLine.vue'
import CTimelineList from '@/components/molecules/CTimelineList.vue'
import CEmptyState from '@/components/atoms/CEmptyState.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconArrowLeft from '@/components/atoms/CIconArrowLeft.svg'
import CIconCalendar from '@/components/atoms/CIconCalendar.svg'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import CIconRefresh from '@/components/atoms/CIconRefresh.svg'
import CIconUser from '@/components/atoms/CIconUser.svg'
import { reportsApi, unitsApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import type { Report, TimelineEvent, Unit } from '@/types/domain'

const { t } = useI18n()
const { formatDate } = useFormatters()
const { authorLabel, statusVariant } = useReportLabels()
const perms = usePermissions()
const route = useRoute()
const router = useRouter()

const reportId = route.params.id as string
const buildingName = ref('')
const report = ref<Report | null>(null)
const unitsById = ref(new Map<string, Unit>())

const timeline = ref<TimelineEvent[]>([])
const timelineLoading = ref(false)
const timelineError = ref<string | null>(null)

const canLinkTask = computed(() => perms.canManageTask.value)

async function loadData() {
  const [building, reportRes, unitsRes] = await Promise.all([
    useBuildingStore().fetch(),
    reportsApi.get(reportId),
    unitsApi.list(),
  ])
  buildingName.value = building.name
  report.value = reportRes.data
  unitsById.value = new Map(unitsRes.data.map(unit => [unit.id, unit]))
}

async function loadTimeline() {
  timelineLoading.value = true
  timelineError.value = null
  try {
    timeline.value = (await reportsApi.timeline(reportId)).data
  } catch {
    timelineError.value = t('timeline.error')
  } finally {
    timelineLoading.value = false
  }
}

const { loading, error, reload } = usePageLoader(async () => {
  try {
    await loadData()
  } catch (e: unknown) {
    if ((e as { response?: { status?: number } })?.response?.status === 404) {
      router.replace({ name: 'reports' })
      return
    }
    throw e
  }
})
onMounted(() => {
  reload()
  loadTimeline()
})

function unitLabel(unitId?: string): string {
  if (!unitId) return t('claims.commonAreas')
  const unit = unitsById.value.get(unitId)
  return unit ? t('claims.unitLabel', { number: unit.unit_number }) : '—'
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight />
          <RouterLink
            :to="{ name: 'reports' }"
            class="topbar-breadcrumb-link"
          >
            {{ t('nav.reports') }}
          </RouterLink>
          <CIconChevronRight />
          <span>{{ report?.title ?? '…' }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="ghost"
          size="sm"
          :icon="CIconArrowLeft"
          :title="t('common.back')"
          @click="router.push({ name: 'reports' })"
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
      <template v-else-if="report">
        <header class="view-header">
          <CEyebrow>
            <CBadge
              :variant="statusVariant(report.status)"
              dot
            >
              {{ t(`reports.statuses.${report.status}`) }}
            </CBadge>
            {{ t('page.eyebrows.reports') }}
          </CEyebrow>
          <CHeading
            type="header"
            class="m-0"
          >
            {{ report.title }}
          </CHeading>
          <p
            v-if="report.description"
            class="body-md text-on-surface-variant m-0"
          >
            {{ report.description }}
          </p>
          <CMetaLine class="[&_svg]:size-4 text-xs">
            <CMetaItem>
              <template #icon>
                <CIconUser />
              </template>
              {{ authorLabel(report) }}
            </CMetaItem>
            <CMetaItem>
              <template #icon>
                <CIconCalendar />
              </template>
              {{ formatDate(report.created_at) }}
            </CMetaItem>
            <CMetaItem>
              {{ unitLabel(report.unit_id) }}
            </CMetaItem>
          </CMetaLine>
        </header>

        <CCard
          v-if="report.task_id"
          :level="3"
          class="flex flex-wrap items-center justify-between gap-3"
        >
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-sm font-medium text-on-surface">{{ t('reports.convertedTitle') }}</span>
            <CBadge
              v-if="report.task_status"
              variant="open"
            >
              {{ t(`tasks.statuses.${report.task_status}`) }}
            </CBadge>
          </div>
          <RouterLink
            v-if="canLinkTask && report.task_status"
            :to="{ name: 'task-detail', params: { id: report.task_id } }"
            class="text-sm font-medium text-primary hover:underline"
          >
            {{ t('reports.viewTask') }}
          </RouterLink>
        </CCard>

        <h2 class="text-xl font-semibold text-on-surface-variant mt-0 mb-4">
          {{ t('timeline.title') }}
        </h2>

        <CPageLoader v-if="timelineLoading" />
        <CCard
          v-else-if="timelineError"
          :level="3"
          class="flex flex-col items-start gap-3"
        >
          <p class="text-sm text-on-surface-variant m-0">
            {{ timelineError }}
          </p>
          <CBtn
            variant="ghost"
            size="sm"
            :icon="CIconRefresh"
            :title="t('timeline.retry')"
            @click="loadTimeline"
          />
        </CCard>
        <CEmptyState
          v-else-if="!timeline.length"
          :title="t('timeline.empty')"
          size="compact"
        />
        <CTimelineList
          v-else
          :events="timeline"
          :link-tasks="canLinkTask"
        />
      </template>
    </div>
  </div>
</template>
