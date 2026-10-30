<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
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
import CIconMessageSquare from '@/components/atoms/CIconMessageSquare.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconRefresh from '@/components/atoms/CIconRefresh.svg'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CreateReportModal from './CreateReportModal.vue'
import UpdateReportStatusModal from './UpdateReportStatusModal.vue'
import ConvertReportToTaskModal from './ConvertReportToTaskModal.vue'
import EditReportModal from './EditReportModal.vue'
import DeleteReportModal from './DeleteReportModal.vue'
import ReportPhotosModal from './ReportPhotosModal.vue'
import { reportsApi, unitsApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import { useBuildingStore } from '@/stores/building'
import type { Report, ReportComment, ReportStatus, Unit } from '@/types/domain'
import { ALL_FILTER, REPORT_STATUS_CLOSED, REPORT_STATUS_IN_PROGRESS, REPORT_STATUS_OPEN } from '@/utils/constants'
import { usePermissions } from '@/composables/usePermissions'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()
const router = useRouter()
const perms = usePermissions()


// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const allReports = ref<Report[]>([])
const allUnits = ref<Unit[]>([])
const statusFilter = ref<typeof ALL_FILTER | ReportStatus>(ALL_FILTER)

// ── RBAC — only manager and board_member can act on reports ────────
const canManage = perms.canManageReport
const canAdminister = perms.canAdministerReport
const canComment = perms.canCommentReport
const authStore = useAuthStore()

// ── Modal state ───────────────────────────────────────────────────
const showCreateModal = ref(false)
const reportToUpdate = ref<Report | null>(null)
const reportToConvert = ref<Report | null>(null)
const reportToEdit = ref<Report | null>(null)
const reportToDelete = ref<Report | null>(null)
const reportPhotoGalleryTarget = ref<Report | null>(null)

// ── Follow-up comments ────────────────────────────────────────────────────────
const expandedCommentsReportId = ref<string | null>(null)
const commentsByReport = ref<Record<string, ReportComment[]>>({})
const loadingComments = ref(false)
const newCommentBody = ref('')
const submittingComment = ref(false)

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
  (statusFilter.value === ALL_FILTER
    ? allReports.value
    : allReports.value.filter(r => r.status === statusFilter.value))
    .slice()
    .sort((left, right) => right.updated_at.localeCompare(left.updated_at)),
)

const counts = computed(() => ({
  open: allReports.value.filter(r => r.status === REPORT_STATUS_OPEN).length,
  in_progress: allReports.value.filter(r => r.status === REPORT_STATUS_IN_PROGRESS).length,
  closed: allReports.value.filter(r => r.status === REPORT_STATUS_CLOSED).length }))

const activeCount = computed(() =>
  allReports.value.filter(r => r.status !== REPORT_STATUS_CLOSED).length,
)

const unitsById = computed(() =>
  new Map(allUnits.value.map(u => [u.id, u])),
)

const unitOptions = computed(() => [
  { value: '', label: t('reports.unitPlaceholder') },
  ...allUnits.value.map(u => ({
    value: u.id,
    label: t('claims.unitLabel', { number: u.unit_number }) })),
])

// ── Helpers ───────────────────────────────────────────────────────
function statusVariant(s: string): 'valid' | 'open' | 'pending' | 'closed' | 'default' {
  if (s === REPORT_STATUS_OPEN) return 'pending'
  if (s === REPORT_STATUS_IN_PROGRESS) return 'open'
  if (s === REPORT_STATUS_CLOSED) return 'closed'
  return 'default'
}

function unitLabel(unitId?: string): string {
  if (!unitId) return t('claims.commonAreas')
  const u = unitsById.value.get(unitId)
  return u ? t('claims.unitLabel', { number: u.unit_number }) : '—'
}

function authorName(r: Report): string {
  return `${r.author_first_name} ${r.author_last_name}`
}

function canReadComments(r: Report): boolean {
  return canComment.value || r.author_id === authStore.user?.user_id
}

async function toggleComments(r: Report) {
  if (expandedCommentsReportId.value === r.id) {
    expandedCommentsReportId.value = null
    return
  }
  expandedCommentsReportId.value = r.id
  if (!commentsByReport.value[r.id]) {
    loadingComments.value = true
    try {
      const response = await reportsApi.listComments(r.id)
      commentsByReport.value[r.id] = response.data
    } catch {
      expandedCommentsReportId.value = null
    } finally {
      loadingComments.value = false
    }
  }
}

async function submitComment(r: Report) {
  const body = newCommentBody.value.trim()
  if (!body) return
  submittingComment.value = true
  try {
    const response = await reportsApi.addComment(r.id, body)
    if (!commentsByReport.value[r.id]) commentsByReport.value[r.id] = []
    commentsByReport.value[r.id].push(response.data)
    newCommentBody.value = ''
  } catch {
    // error handled by interceptor
  } finally {
    submittingComment.value = false
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight />
          <span>{{ t('nav.reports') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('reports.create')"
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
            {{ t('nav.reports') }}
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
            { value: ALL_FILTER, label: t('common.allMasc'), count: allReports.length },
            { value: REPORT_STATUS_OPEN, label: t('reports.filters.open'), count: counts.open },
            { value: REPORT_STATUS_IN_PROGRESS, label: t('reports.filters.inProgress'), count: counts.in_progress },
            { value: REPORT_STATUS_CLOSED, label: t('reports.filters.closed'), count: counts.closed },
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
              :shadow="r.status !== REPORT_STATUS_CLOSED"
              :class="['gap-y-4', { 'outline outline-1 outline-outline-variant': r.status === REPORT_STATUS_CLOSED }]"
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
                    {{ t('reports.linkedPhotosCount', r.document_count) }}
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
                  v-if="canAdminister && r.status !== REPORT_STATUS_CLOSED"
                  class="flex flex-wrap gap-x-2 gap-y-1"
                >
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :icon="CIconRefresh"
                    :title="t('tasks.updateStatus')"
                    @click="openStatusModal(r)"
                  />
                </div>
  
                <div
                  v-if="canReadComments(r)"
                  class="flex items-center"
                >
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :icon="CIconMessageSquare"
                    :title="t('reports.commentsTitle')"
                    @click="toggleComments(r)"
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
                      v-else-if="canAdminister && r.status !== REPORT_STATUS_CLOSED"
                      variant="primary"
                      size="sm"
                      :icon="CIconCheckCircle"
                      :title="t('reports.convertToTask')"
                      @click="openConvertModal(r)"
                    />
                  </CCardActions>
                </template>

                <div
                  v-if="expandedCommentsReportId === r.id"
                  class="flex flex-col gap-3 border-t border-outline-variant pt-3"
                >
                  <h4 class="text-sm font-semibold text-on-surface m-0">
                    {{ t('reports.commentsTitle') }}
                  </h4>
                  <CPageLoader v-if="loadingComments" />
                  <template v-else>
                    <p
                      v-if="!commentsByReport[r.id]?.length"
                      class="text-sm text-on-surface-variant m-0"
                    >
                      {{ t('reports.commentsEmpty') }}
                    </p>
                    <ul
                      v-else
                      class="flex flex-col gap-2 m-0 p-0 list-none"
                    >
                      <li
                        v-for="c in commentsByReport[r.id]"
                        :key="c.id"
                        class="flex flex-col gap-0-5 text-sm"
                      >
                        <span class="text-xs text-on-surface-variant">
                          {{ c.author_first_name }} {{ c.author_last_name }} · {{ formatDate(c.created_at, 'datetime') }}
                        </span>
                        <span class="text-on-surface whitespace-pre-wrap">{{ c.body }}</span>
                      </li>
                    </ul>
                  </template>
                  <div
                    v-if="canComment"
                    class="flex flex-col gap-2"
                  >
                    <CFormTextarea
                      v-model="newCommentBody"
                      :placeholder="t('reports.commentPlaceholder')"
                      :rows="2"
                    />
                    <div class="flex justify-end">
                      <CBtn
                        variant="primary"
                        size="sm"
                        :loading="submittingComment"
                        :disabled="!newCommentBody.trim()"
                        :title="t('reports.commentAdd')"
                        @click="submitComment(r)"
                      />
                    </div>
                  </div>
                </div>
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
      @updated="reload"
    />

    <ConvertReportToTaskModal
      :open="!!reportToConvert"
      :report="reportToConvert"
      @update:open="v => !v && (reportToConvert = null)"
      @converted="reload"
    />

    <EditReportModal
      :open="!!reportToEdit"
      :report="reportToEdit"
      :unit-options="unitOptions"
      @update:open="v => !v && (reportToEdit = null)"
      @saved="reload"
      @delete="handleEditDelete"
    />

    <DeleteReportModal
      :open="!!reportToDelete"
      :report="reportToDelete"
      @update:open="v => !v && (reportToDelete = null)"
      @deleted="reload"
    />

    <ReportPhotosModal
      :open="!!reportPhotoGalleryTarget"
      :report="reportPhotoGalleryTarget"
      @update:open="v => !v && (reportPhotoGalleryTarget = null)"
    />
  </div>
</template>
