<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
import { useRole } from '@/composables/useRole'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CCardActions from '@/components/molecules/CCardActions.vue'
import CMetaLine from '@/components/molecules/CMetaLine.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import CIconCheckCircle from '@/components/atoms/CIconCheckCircle.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconRefresh from '@/components/atoms/CIconRefresh.svg'
import CreateReportModal from './CreateReportModal.vue'
import UpdateReportStatusModal from './UpdateReportStatusModal.vue'
import ConvertReportToTaskModal from './ConvertReportToTaskModal.vue'
import EditReportModal from './EditReportModal.vue'
import DeleteReportModal from './DeleteReportModal.vue'
import ReportPhotosModal from './ReportPhotosModal.vue'
import { reportsApi, unitsApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import type { Report, ReportStatus, Unit } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()
const router = useRouter()
const { hasAnyRole } = useRole()

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const allReports = ref<Report[]>([])
const allUnits = ref<Unit[]>([])
const statusFilter = ref<'all' | ReportStatus>('all')

// ── RBAC — only manager and board_member can act on reports ────────
const canManage = computed(() => hasAnyRole(['manager', 'board_member']))

// ── Modal state ───────────────────────────────────────────────────
const showCreateModal = ref(false)
const reportToUpdate = ref<Report | null>(null)
const reportToConvert = ref<Report | null>(null)
const reportToEdit = ref<Report | null>(null)
const reportToDelete = ref<Report | null>(null)
const reportPhotoGalleryTarget = ref<Report | null>(null)

function openCreateModal() { showCreateModal.value = true }
function openStatusModal(report: Report) { reportToUpdate.value = report }
function openConvertModal(report: Report) { reportToConvert.value = report }
function openEditModal(report: Report) { reportToEdit.value = report }
function openReportPhotos(report: Report) { reportPhotoGalleryTarget.value = report }
function handleEditDelete() {
  const r = reportToEdit.value
  reportToEdit.value = null
  if (r) reportToDelete.value = r
}

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [building, reportsRes, unitsRes] = await Promise.all([
    useBuildingStore().fetch(),
    reportsApi.list(),
    unitsApi.list(),
  ])
  buildingName.value = building.name
  allReports.value = reportsRes.data
  allUnits.value = unitsRes.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

// ── Derived ───────────────────────────────────────────────────────
const reports = computed(() =>
  (statusFilter.value === 'all'
    ? allReports.value
    : allReports.value.filter(r => r.status === statusFilter.value))
    .slice()
    .sort((left, right) => right.updated_at.localeCompare(left.updated_at)),
)

const counts = computed(() => ({
  open: allReports.value.filter(r => r.status === 'open').length,
  in_progress: allReports.value.filter(r => r.status === 'in_progress').length,
  closed: allReports.value.filter(r => r.status === 'closed').length,
}))

const activeCount = computed(() =>
  allReports.value.filter(r => r.status !== 'closed').length,
)

const unitsById = computed(() =>
  new Map(allUnits.value.map(u => [u.id, u])),
)

const unitOptions = computed(() => [
  { value: '', label: t('reports.unitPlaceholder') },
  ...allUnits.value.map(u => ({
    value: u.id,
    label: t('common.unitLabel', { number: u.unit_number }),
  })),
])

// ── Helpers ───────────────────────────────────────────────────────
function statusVariant(s: string): 'valid' | 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'open') return 'pending'
  if (s === 'in_progress') return 'open'
  if (s === 'closed') return 'closed'
  return 'default'
}

function unitLabel(unitId?: string): string {
  if (!unitId) return t('common.commonAreas')
  const u = unitsById.value.get(unitId)
  return u ? t('common.unitLabel', { number: u.unit_number }) : '—'
}

function authorName(r: Report): string {
  return `${r.author_first_name} ${r.author_last_name}`
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight />
          <span>{{ t('common.reports') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('common.newReport')"
          @click="openCreateModal"
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
            {{ t('page.eyebrows.reports') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('common.reports') }}
          </CHeading>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('reports.activeCount', activeCount) }}
          </p>
        </header>

        <CTabs
          v-model="statusFilter"
          variant="filter"
          :tabs="[
            { value: 'all', label: t('common.all'), count: allReports.length },
            { value: 'open', label: t('reports.filters.open'), count: counts.open },
            { value: 'in_progress', label: t('reports.filters.inProgress'), count: counts.in_progress },
            { value: 'closed', label: t('reports.filters.closed'), count: counts.closed },
          ]"
        />

        <template v-if="!reports.length">
          <CEmptyState
            :title="t('reports.empty')"
          />
        </template>
        <template v-else>
          <div class="flex flex-col gap-3">
            <CCard
              v-for="r in reports"
              :key="r.id"
              :shadow="r.status !== 'closed'"
              :class="['gap-y-4', { 'outline outline-1 outline-outline-variant': r.status === 'closed' }]"
            >
              <div class="flex items-center gap-3 min-w-0">
                <CMetaLine 
                  class="[&_svg]:icon-sm text-xs"
                >
                  <CBadge
                    :variant="statusVariant(r.status)"
                    dot
                  >
                    {{ t(`reports.statuses.${r.status}`) }}
                  </CBadge>
                  <CMetaItem
                    variant="sub" 
                  >
                    <template #icon>
                      <CIconClock />
                    </template>
                    {{ formatRelative(r.updated_at) }}
                  </CMetaItem>
                </CMetaLine>
              </div>
  
              <div class="flex flex-col gap-3">
                <CFlexGroup
                  direction="row"
                >
                  <h3 class="text-lg font-semibold text-on-surface m-0 leading-snug min-w-0">
                    {{ r.title }}
                  </h3>
                  
                  <CMetaItem
                    v-if="r.document_count > 0"
                    variant="sub"
                    as="button"
                    @click="openReportPhotos(r)"
                  >
                    <template #icon>
                      <CIconFile />
                    </template>
                    {{ t('common.photosCount', r.document_count) }}
                  </CMetaItem>
                </CFlexGroup>
  
                <p
                  v-if="r.description"
                >
                  {{ r.description }}
                </p>
                <CMetaLine>
                  <CMetaItem>
                    <template #icon>
                      <CIconUser />
                    </template>
                    {{ authorName(r) }}
                  </CMetaItem>
                  <CMetaItem>
                    <template #icon>
                      <CIconHome />
                    </template>
                    {{ unitLabel(r.unit_id) }}
                  </CMetaItem>
                  <CMetaItem>
                    <template #icon>
                      <CIconCalendar />
                    </template>
                    {{ formatDate(r.created_at) }}
                  </CMetaItem>
                </CMetaLine>
  
                <div
                  v-if="r.status !== 'closed'"
                  class="flex flex-wrap gap-x-2 gap-y-1"
                >
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :icon="CIconRefresh"
                    :title="t('common.changeStatus')"
                    @click="openStatusModal(r)"
                  />
                </div>
  
                <template v-if="canManage">
                  <CCardActions>
                    <CBtn
                      variant="ghost"
                      size="sm"
                      :icon="CIconEdit"
                      :title="t('common.edit')"
                      @click="openEditModal(r)"
                    />
                    <CBtn
                      v-if="r.task_id"
                      variant="primary"
                      size="sm"
                      :icon="CIconCheckCircle"
                      :title="t('reports.viewTask')"
                      @click="router.push({ name: 'task-detail', params: { id: r.task_id } })"
                    />
                    <CBtn
                      v-else-if="r.status !== 'closed'"
                      variant="primary"
                      size="sm"
                      :icon="CIconCheckCircle"
                      :title="t('common.convertToTask')"
                      @click="openConvertModal(r)"
                    />
                  </CCardActions>
                </template>
              </div>
            </CCard>
          </div>
        </template>
      </template>
    </div>

    <CreateReportModal
      :open="showCreateModal"
      :unit-options="unitOptions"
      @update:open="showCreateModal = $event"
      @created="reload"
    />

    <UpdateReportStatusModal
      :open="!!reportToUpdate"
      :report="reportToUpdate"
      @update:open="v => !v && (reportToUpdate = null)"
      @updated="() => { reportToUpdate = null; reload() }"
    />

    <ConvertReportToTaskModal
      :open="!!reportToConvert"
      :report="reportToConvert"
      @update:open="v => !v && (reportToConvert = null)"
      @converted="() => { reportToConvert = null; reload() }"
    />

    <EditReportModal
      :open="!!reportToEdit"
      :report="reportToEdit"
      :unit-options="unitOptions"
      @update:open="v => !v && (reportToEdit = null)"
      @saved="() => { reportToEdit = null; reload() }"
      @delete="handleEditDelete"
    />

    <DeleteReportModal
      :open="!!reportToDelete"
      :report="reportToDelete"
      @update:open="v => !v && (reportToDelete = null)"
      @deleted="() => { reportToDelete = null; reload() }"
    />

    <ReportPhotosModal
      :open="!!reportPhotoGalleryTarget"
      :report="reportPhotoGalleryTarget"
      @update:open="v => !v && (reportPhotoGalleryTarget = null)"
    />
  </div>
</template>
