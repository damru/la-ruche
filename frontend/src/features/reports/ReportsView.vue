<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useRole } from '@/composables/useRole'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CMetaLine from '@/components/molecules/CMetaLine.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CDocumentLinkActionModal from '@/components/organisms/CDocumentLinkActionModal.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import { buildingApi, documentsApi, reportsApi, unitsApi } from '@/api'
import type { Document, Report, ReportStatus, TaskPriority, Unit } from '@/types/domain'

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

// Status values as a const to avoid inline repetition in the template and ensure type-safety.
const REPORT_STATUSES: ReportStatus[] = ['open', 'in_progress', 'closed']
const DEFAULT_CONVERT_PRIORITY: TaskPriority = 'normal'

// ── Create report modal ───────────────────────────────────────────
const showCreateModal = ref(false)
const createFeedback = useFormFeedback()
const saving = ref(false)
const createForm = ref({ title: '', description: '', unit_id: '' })

// Photo selection state (create modal)
const selectedPhotos = ref<File[]>([])
const photoInputRef = ref<HTMLInputElement | null>(null)
const photoDragOver = ref(false)
const uploadingPhotos = ref(false)

// Photo selection state (edit modal)
const editSelectedPhotos = ref<File[]>([])
const editPhotoInputRef = ref<HTMLInputElement | null>(null)
const editPhotoDragOver = ref(false)
const editUploadingPhotos = ref(false)
const editAttachedPhotos = ref<{ doc: Document; url: string }[]>([])
const loadingEditAttachedPhotos = ref(false)

const { schedule: scheduleCreateClose, forceExpire: forceCreateClose } = useAutoClose()
watch(showCreateModal, (open) => { if (!open) forceCreateClose() })

function openCreateModal() {
  createForm.value = { title: '', description: '', unit_id: '' }
  selectedPhotos.value = []
  photoDragOver.value = false
  createFeedback.reset()
  showCreateModal.value = true
}

function onPhotoChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files) {
    selectedPhotos.value = [...selectedPhotos.value, ...Array.from(input.files)]
  }
  // Reset so the same file can be added again if needed
  if (photoInputRef.value) photoInputRef.value.value = ''
}

function onPhotoDrop(e: DragEvent) {
  photoDragOver.value = false
  const files = Array.from(e.dataTransfer?.files ?? []).filter(f => f.type.startsWith('image/'))
  selectedPhotos.value = [...selectedPhotos.value, ...files]
}

function removePhoto(idx: number) {
  selectedPhotos.value = selectedPhotos.value.filter((_, i) => i !== idx)
}

function onEditPhotoChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files) {
    editSelectedPhotos.value = [...editSelectedPhotos.value, ...Array.from(input.files)]
  }
  if (editPhotoInputRef.value) editPhotoInputRef.value.value = ''
}

function onEditPhotoDrop(e: DragEvent) {
  editPhotoDragOver.value = false
  const files = Array.from(e.dataTransfer?.files ?? []).filter(f => f.type.startsWith('image/'))
  editSelectedPhotos.value = [...editSelectedPhotos.value, ...files]
}

function removeEditPhoto(idx: number) {
  editSelectedPhotos.value = editSelectedPhotos.value.filter((_, i) => i !== idx)
}

function openEditPhotoAction(document: Document) {
  if (!reportToEdit.value) return
  openPhotoActionModal(reportToEdit.value.id, document)
}

/** Upload a single photo and return its document_id. */
async function uploadPhoto(reportId: string, file: File): Promise<void> {
  const ext = file.name.split('.').pop() ?? 'jpg'
  const mimeType = file.type || 'image/jpeg'
  const urlRes = await documentsApi.getUploadUrl(ext, mimeType, file.size)
  const { url, key, bucket } = urlRes.data
  await documentsApi.uploadToS3(file, url, mimeType)
  const docRes = await documentsApi.confirmUpload({
    key,
    bucket,
    original_name: file.name,
    mime_type: mimeType,
    size_bytes: file.size,
    visible_roles: ['manager', 'board_member', 'co_owner'],
  })
  await reportsApi.attachDocument(reportId, docRes.data.id)
}

async function submitCreate() {
  if (createFeedback.status.value === 'success') { forceCreateClose(); return }
  if (!createForm.value.title.trim()) return
  saving.value = true
  createFeedback.reset()
  try {
    const reportRes = await reportsApi.create({
      title: createForm.value.title.trim(),
      description: createForm.value.description.trim() || undefined,
      unit_id: createForm.value.unit_id || undefined,
    })
    const reportId = reportRes.data.id
    if (selectedPhotos.value.length) {
      uploadingPhotos.value = true
      const errors: string[] = []
      for (const photo of selectedPhotos.value) {
        try {
          await uploadPhoto(reportId, photo)
        } catch {
          errors.push(photo.name)
        }
      }
      uploadingPhotos.value = false
      if (errors.length) {
        createFeedback.setError(t('reports.photoUploadError') + ': ' + errors.join(', '))
        // Still reload — report was created
        setTimeout(() => { showCreateModal.value = false; reload() }, 2000)
        return
      }
    }

    createFeedback.setSuccess(t('reports.createSuccess'))
    scheduleCreateClose(() => { showCreateModal.value = false; reload() })
  } catch (err) {
    createFeedback.setError(toErrorMessage(err))
  } finally {
    saving.value = false
    uploadingPhotos.value = false
  }
}

// ── Update status modal ───────────────────────────────────────────
const reportToUpdate = ref<Report | null>(null)
const statusForm = ref<ReportStatus>('in_progress')
const updateFeedback = useFormFeedback()
const updating = ref(false)
const { schedule: scheduleUpdateClose, forceExpire: forceUpdateClose } = useAutoClose()
watch(reportToUpdate, (r) => { if (!r) forceUpdateClose() })

function openStatusModal(report: Report) {
  reportToUpdate.value = report
  statusForm.value = report.status === 'closed' ? 'open' : (report.status === 'open' ? 'in_progress' : 'closed')
  updateFeedback.reset()
}

async function submitUpdateStatus() {
  if (updateFeedback.status.value === 'success') { forceUpdateClose(); return }
  if (!reportToUpdate.value) return
  updating.value = true
  updateFeedback.reset()
  try {
    await reportsApi.updateStatus(reportToUpdate.value.id, statusForm.value)
    updateFeedback.setSuccess(t('reports.updateSuccess'))
    scheduleUpdateClose(() => { reportToUpdate.value = null; reload() })
  } catch (err) {
    updateFeedback.setError(toErrorMessage(err))
  } finally {
    updating.value = false
  }
}

// ── Convert to task modal ─────────────────────────────────────────
const reportToConvert = ref<Report | null>(null)
const convertPriority = ref<TaskPriority>(DEFAULT_CONVERT_PRIORITY)
const convertFeedback = useFormFeedback()
const converting = ref(false)
const { schedule: scheduleConvertClose, forceExpire: forceConvertClose } = useAutoClose()
watch(reportToConvert, (r) => { if (!r) forceConvertClose() })

function openConvertModal(report: Report) {
  reportToConvert.value = report
  convertPriority.value = DEFAULT_CONVERT_PRIORITY
  convertFeedback.reset()
}

async function submitConvert() {
  if (convertFeedback.status.value === 'success') { forceConvertClose(); return }
  if (!reportToConvert.value) return
  converting.value = true
  convertFeedback.reset()
  try {
    const res = await reportsApi.convertToTask(reportToConvert.value.id, {
      priority: convertPriority.value,
    })
    convertFeedback.setSuccess(t('reports.convertSuccess'))
    scheduleConvertClose(() => {
      reportToConvert.value = null
      router.push({ name: 'task-detail', params: { id: res.data.task_id } })
    })
  } catch (err) {
    convertFeedback.setError(toErrorMessage(err))
  } finally {
    converting.value = false
  }
}

// ── Edit report modal ─────────────────────────────────────────────
const reportToEdit = ref<Report | null>(null)
const editForm = ref({ title: '', description: '', unit_id: '' })
const editFeedback = useFormFeedback()
const editSaving = ref(false)
const { schedule: scheduleEditClose, forceExpire: forceEditClose } = useAutoClose()
watch(reportToEdit, (r) => { if (!r) forceEditClose() })

function openEditModal(report: Report) {
  void loadEditAttachedPhotos(report.id)
  reportToEdit.value = report
  editForm.value = {
    title: report.title,
    description: report.description ?? '',
    unit_id: report.unit_id ?? '',
  }
  editSelectedPhotos.value = []
  editPhotoDragOver.value = false
  editFeedback.reset()
}

async function submitEdit() {
  if (editFeedback.status.value === 'success') { forceEditClose(); return }
  if (!reportToEdit.value) return
  if (!editForm.value.title.trim()) return
  editSaving.value = true
  editFeedback.reset()
  try {
    await reportsApi.update(reportToEdit.value.id, {
      title: editForm.value.title.trim(),
      description: editForm.value.description.trim() || null,
      unit_id: editForm.value.unit_id || null,
    })

    if (editSelectedPhotos.value.length) {
      editUploadingPhotos.value = true
      const photoErrors: string[] = []
      for (const photo of editSelectedPhotos.value) {
        try { await uploadPhoto(reportToEdit.value.id, photo) } catch { photoErrors.push(photo.name) }
      }
      editUploadingPhotos.value = false
      if (photoErrors.length) {
        editFeedback.setError(t('reports.photoUploadError') + ': ' + photoErrors.join(', '))
        setTimeout(() => { reportToEdit.value = null; reload() }, 2000)
        return
      }
    }

    editFeedback.setSuccess(t('reports.editSuccess'))
    scheduleEditClose(() => { reportToEdit.value = null; reload() })
  } catch (err) {
    editFeedback.setError(toErrorMessage(err))
  } finally {
    editSaving.value = false
    editUploadingPhotos.value = false
  }
}

// ── Delete report modal ───────────────────────────────────────────
const reportToDelete = ref<Report | null>(null)
const deleteFeedback = useFormFeedback()
const deleting = ref(false)
const { schedule: scheduleDeleteClose, forceExpire: forceDeleteClose } = useAutoClose()
watch(reportToDelete, (r) => { if (!r) forceDeleteClose() })

function openDeleteModal(report: Report) {
  reportToDelete.value = report
  deleteFeedback.reset()
}

/** Called from the edit modal footer — close edit, open delete confirmation. */
function openDeleteFromEdit() {
  const report = reportToEdit.value
  reportToEdit.value = null
  if (report) openDeleteModal(report)
}

async function submitDelete() {
  if (deleteFeedback.status.value === 'success') { forceDeleteClose(); return }
  if (!reportToDelete.value) return
  deleting.value = true
  deleteFeedback.reset()
  try {
    await reportsApi.delete(reportToDelete.value.id)
    deleteFeedback.setSuccess(t('reports.deleteSuccess'))
    scheduleDeleteClose(() => { reportToDelete.value = null; reload() })
  } catch (err) {
    deleteFeedback.setError(toErrorMessage(err))
  } finally {
    deleting.value = false
  }
}

// ── Linked documents modal ────────────────────────────────────────
const deletingPhoto = ref(false)
const photoActionTarget = ref<{ reportId: string; document: Document } | null>(null)
const photoActionFeedback = useFormFeedback()
const showPhotoActionModal = computed(() => photoActionTarget.value !== null)
const reportPhotoGalleryTarget = ref<Report | null>(null)
const reportPhotos = ref<Array<{ doc: Document; url: string }>>([])
const reportPhotosLoading = ref(false)
const activeReportPhotoIndex = ref<number | null>(null)

function updateReportDocumentCount(reportId: string, delta: number) {
  const report = allReports.value.find(currentReport => currentReport.id === reportId)
  if (report && typeof report.document_count === 'number') {
    report.document_count = Math.max(0, report.document_count + delta)
  }
}

async function fetchReportPhotos(reportId: string): Promise<Array<{ doc: Document; url: string }>> {
  const docsRes = await reportsApi.listDocuments(reportId)
  return Promise.all(
    docsRes.data.map(async (document) => {
      const urlRes = await documentsApi.getDownloadUrl(document.id)
      return { doc: document, url: urlRes.data.url }
    }),
  )
}

async function loadEditAttachedPhotos(reportId: string) {
  loadingEditAttachedPhotos.value = true

  try {
    editAttachedPhotos.value = await fetchReportPhotos(reportId)
  } catch {
    editAttachedPhotos.value = []
  } finally {
    loadingEditAttachedPhotos.value = false
  }
}

function openPhotoActionModal(reportId: string, document: Document) {
  photoActionTarget.value = { reportId, document }
  photoActionFeedback.reset()
}

function closePhotoActionModal() {
  if (deletingPhoto.value) return
  photoActionTarget.value = null
  photoActionFeedback.reset()
}

async function handlePhotoAction(deleteDocument: boolean) {
  if (!photoActionTarget.value || deletingPhoto.value) return

  deletingPhoto.value = true
  photoActionFeedback.reset()

  try {
    await reportsApi.detachDocument(photoActionTarget.value.reportId, photoActionTarget.value.document.id)
    if (deleteDocument) {
      await documentsApi.delete(photoActionTarget.value.document.id)
    }

    editAttachedPhotos.value = editAttachedPhotos.value.filter(item => item.doc.id !== photoActionTarget.value?.document.id)
    updateReportDocumentCount(photoActionTarget.value.reportId, -1)

    photoActionTarget.value = null
  } catch {
    photoActionFeedback.setError(t('common.error'))
  } finally {
    deletingPhoto.value = false
  }
}

function closeReportPhotoGallery() {
  reportPhotoGalleryTarget.value = null
  reportPhotos.value = []
  reportPhotosLoading.value = false
  activeReportPhotoIndex.value = null
}

async function openReportPhotos(report: Report) {
  reportPhotoGalleryTarget.value = report
  reportPhotos.value = []
  reportPhotosLoading.value = true

  try {
    reportPhotos.value = await fetchReportPhotos(report.id)
  } catch {
    photoActionFeedback.setError(t('common.error'))
    closeReportPhotoGallery()
  } finally {
    reportPhotosLoading.value = false
  }
}

function openReportPhotoViewer(index: number) {
  activeReportPhotoIndex.value = index
}

function closeReportPhotoViewer() {
  activeReportPhotoIndex.value = null
}

function showPreviousReportPhoto() {
  if (activeReportPhotoIndex.value === null || reportPhotos.value.length <= 1) return
  activeReportPhotoIndex.value = (activeReportPhotoIndex.value - 1 + reportPhotos.value.length) % reportPhotos.value.length
}

function showNextReportPhoto() {
  if (activeReportPhotoIndex.value === null || reportPhotos.value.length <= 1) return
  activeReportPhotoIndex.value = (activeReportPhotoIndex.value + 1) % reportPhotos.value.length
}

const activeReportPhoto = computed(() =>
  activeReportPhotoIndex.value === null ? null : reportPhotos.value[activeReportPhotoIndex.value] ?? null,
)

const activeReportPhotoPosition = computed(() =>
  activeReportPhotoIndex.value === null ? 0 : activeReportPhotoIndex.value + 1,
)

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [buildingRes, reportsRes, unitsRes] = await Promise.all([
    buildingApi.get(),
    reportsApi.list(),
    unitsApi.list(),
  ])
  buildingName.value = buildingRes.data.name
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
    label: t('reports.unitLabel', { number: u.unit_number }),
  })),
])

const reportStatusOptions = computed(() =>
  REPORT_STATUSES.map(s => ({ value: s, label: t(`reports.statuses.${s}`) })),
)

const taskPriorityOptions = computed(() =>
  (['low', 'normal', 'high', 'urgent'] as TaskPriority[]).map(priority => ({
    value: priority,
    label: t(`tasks.priorities.${priority}`),
  })),
)

// ── Helpers ───────────────────────────────────────────────────────
function statusVariant(s: string): 'valid' | 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'open') return 'pending'
  if (s === 'in_progress') return 'open'
  if (s === 'closed') return 'closed'
  return 'default'
}

function unitLabel(unitId?: string): string {
  if (!unitId) return t('reports.noUnit')
  const u = unitsById.value.get(unitId)
  return u ? t('reports.unitLabel', { number: u.unit_number }) : '—'
}

function authorName(r: Report): string {
  return `${r.author_first_name} ${r.author_last_name}`
}

function photoPreview(file: File): string {
  return URL.createObjectURL(file)
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight />
          <span>{{ t('reports.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          @click="openCreateModal"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('reports.newReport') }}</span>
        </CBtn>
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
          <CHeading size="lg" as="h1">
            {{ t('reports.title') }}
          </CHeading>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('reports.activeCount', activeCount) }}
          </p>
        </header>

        <!-- Filters -->
        <CTabs
          v-model="statusFilter"
          :tabs="[
            { value: 'all', label: t('common.all'), count: allReports.length },
            { value: 'open', label: t('reports.filters.open'), count: counts.open },
            { value: 'in_progress', label: t('reports.filters.inProgress'), count: counts.in_progress },
            { value: 'closed', label: t('reports.filters.closed'), count: counts.closed },
          ]"
        />

        <!-- List -->
        <div class="report-list">
          <CCard
            v-for="r in reports"
            :key="r.id"
            :shadow="r.status !== 'closed'"
            :class="['gap-y-4', { 'card--terminated': r.status === 'closed' }]"
          >
            <div class="activity-card__header-row activity-card__header-row--with-icon">
              <div class="activity-card__icon tone--default">
                <CIconInfo />
              </div>
              
              <CMetaLine class="[&_svg]:w-(--icon-sm) [&_svg]:h-(--icon-sm) text-xs">
                <CBadge
                  :variant="statusVariant(r.status)"
                  dot
                >
                  {{ t(`reports.statuses.${r.status}`) }}
                </CBadge>
                <CMetaItem>
                  <template #icon>
                    <CIconClock />
                  </template>
                  {{ formatRelative(r.updated_at) }}
                </CMetaItem>
              </CMetaLine>
            </div>

            <div class="activity-card__stack">
              <CFlexGroup
                direction="row"
              >
                <h3 class="activity-card__title">
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

              <div class="activity-card__secondary-actions">
                <div
                  v-if="r.status !== 'closed'"
                  class="activity-card__secondary-actions"
                >
                  <CBtn
                    variant="ghost"
                    size="sm"
                    @click="openStatusModal(r)"
                  >
                    <CIconRefresh />
                    {{ t('reports.editStatus') }}
                  </CBtn>
                </div>
              </div>

              <template v-if="canManage">
                <div class="activity-card__actions">
                  <CBtn
                    variant="ghost"
                    size="sm"
                    @click="openEditModal(r)"
                  >
                    <CIconEdit />
                    {{ t('common.edit') }}
                  </CBtn>
                  <CBtn
                    v-if="r.task_id"
                    variant="primary"
                    size="sm"
                    @click="router.push({ name: 'task-detail', params: { id: r.task_id } })"
                  >
                    <CIconCheckCircle />
                    {{ t('reports.viewTask') }}
                  </CBtn>
                  <CBtn
                    v-else-if="r.status !== 'closed'"
                    variant="primary"
                    size="sm"
                    @click="openConvertModal(r)"
                  >
                    <CIconCheckCircle />
                    {{ t('reports.convertToTask') }}
                  </CBtn>
                </div>
              </template>
            </div>
          </CCard>

          <div
            v-if="!reports.length"
            class="empty-state"
          >
            {{ t('reports.empty') }}
          </div>
        </div>
      </template>
    </div>

    <!-- Create report modal -->
    <CModal
      :open="showCreateModal"
      :title="t('reports.newReport')"
      size="md"
      :closable="!saving && !uploadingPhotos"
      @update:open="showCreateModal = $event"
    >
      <form
        class="form-grid"
        @submit.prevent="submitCreate"
      >
        <CFormInput
          v-model="createForm.title"
          :label="t('reports.titleLabel')"
          :placeholder="t('reports.titleLabel')"
          required
        />
        <CFormTextarea
          v-model="createForm.description"
          :label="t('reports.descriptionLabel')"
          :rows="3"
        />
        <CFormSelect
          v-model="createForm.unit_id"
          :label="t('reports.unitLabel2')"
          :options="unitOptions"
        />

        <!-- Photo picker -->
        <div class="form-field">
          <label class="form-label">{{ t('reports.photos') }}</label>
          <!-- Hidden real file input -->
          <input
            ref="photoInputRef"
            type="file"
            accept="image/*"
            multiple
            style="display:none"
            @change="onPhotoChange"
          />
          <!-- Drop zone -->
          <div
            :class="['dropzone', { 'dropzone--over': photoDragOver }]"
            role="button"
            tabindex="0"
            @click="photoInputRef?.click()"
            @keydown.enter="photoInputRef?.click()"
            @dragover.prevent="photoDragOver = true"
            @dragleave.prevent="photoDragOver = false"
            @drop.prevent="onPhotoDrop"
          >
            <CIconUpload class="dropzone__icon" />
            <span class="body-xs dropzone__hint">{{ t('reports.photoHint') }}</span>
          </div>
          <!-- Preview strip -->
          <div
            v-if="selectedPhotos.length"
            class="photo-preview-strip"
          >
            <div
              v-for="(photo, idx) in selectedPhotos"
              :key="idx"
              class="photo-preview-item"
            >
              <img
                :src="photoPreview(photo)"
                :alt="photo.name"
                class="photo-thumb"
              />
              <button
                type="button"
                class="photo-remove"
                :title="t('reports.removePhoto')"
                @click="removePhoto(idx)"
              >
                ×
              </button>
            </div>
          </div>
          <span
            v-if="uploadingPhotos"
            class="body-xs"
            style="color:var(--color-on-surface-variant)"
          >{{ t('reports.photoUploading') }}</span>
        </div>
      </form>
      <template #footer>
        <CFormFeedback
          :status="createFeedback.status.value"
          :message="createFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showCreateModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="saving || uploadingPhotos"
          :closing="createFeedback.status.value === 'success'"
          @click="submitCreate"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Update status modal -->
    <CModal
      :open="!!reportToUpdate"
      :title="t('reports.confirmStatusTitle')"
      size="sm"
      @update:open="v => !v && (reportToUpdate = null)"
    >
      <CFormSelect
        v-model="statusForm"
        :label="t('reports.statusLabel')"
        :options="reportStatusOptions"
      />
      <template #footer>
        <CFormFeedback
          :status="updateFeedback.status.value"
          :message="updateFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="reportToUpdate = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="updating"
          :closing="updateFeedback.status.value === 'success'"
          @click="submitUpdateStatus"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Convert to task modal -->
    <CModal
      :open="!!reportToConvert"
      :title="t('reports.confirmConvertTitle')"
      size="sm"
      @update:open="v => !v && (reportToConvert = null)"
    >
      <p class="body-xs">
        {{ t('reports.confirmConvertMessage', { title: reportToConvert?.title ?? '' }) }}
      </p>
      <CFormSelect
        v-model="convertPriority"
        :label="t('tasks.priorityLabel')"
        :options="taskPriorityOptions"
      />
      <template #footer>
        <CFormFeedback
          :status="convertFeedback.status.value"
          :message="convertFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="reportToConvert = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="converting"
          :closing="convertFeedback.status.value === 'success'"
          @click="submitConvert"
        >
          {{ t('reports.convertToTask') }}
        </CBtn>
      </template>
    </CModal>
    <!-- Edit report modal -->
    <CModal
      :open="!!reportToEdit"
      :title="t('reports.editReport')"
      size="md"
      :closable="!editSaving && !editUploadingPhotos"
      @update:open="v => !v && (reportToEdit = null)"
    >
      <form
        class="form-grid"
        @submit.prevent="submitEdit"
      >
        <CFormInput
          v-model="editForm.title"
          :label="t('reports.titleLabel')"
          :placeholder="t('reports.titleLabel')"
          required
        />
        <CFormTextarea
          v-model="editForm.description"
          :label="t('reports.descriptionLabel')"
          :rows="3"
        />
        <CFormSelect
          v-model="editForm.unit_id"
          :label="t('reports.unitLabel2')"
          :options="unitOptions"
        />

        <!-- Photo picker (add photos to existing report) -->
        <div class="form-field">
          <label class="form-label">{{ t('reports.addPhotos') }}</label>
          <input
            ref="editPhotoInputRef"
            type="file"
            accept="image/*"
            multiple
            style="display:none"
            @change="onEditPhotoChange"
          />
          <div
            :class="['dropzone', { 'dropzone--over': editPhotoDragOver }]"
            role="button"
            tabindex="0"
            @click="editPhotoInputRef?.click()"
            @keydown.enter="editPhotoInputRef?.click()"
            @dragover.prevent="editPhotoDragOver = true"
            @dragleave.prevent="editPhotoDragOver = false"
            @drop.prevent="onEditPhotoDrop"
          >
            <CIconUpload class="dropzone__icon" />
            <span class="body-xs dropzone__hint">{{ t('reports.photoHint') }}</span>
          </div>
          <div
            v-if="editSelectedPhotos.length"
            class="photo-preview-strip"
          >
            <div
              v-for="(photo, idx) in editSelectedPhotos"
              :key="idx"
              class="photo-preview-item"
            >
              <img
                :src="photoPreview(photo)"
                :alt="photo.name"
                class="photo-thumb"
              />
              <button
                type="button"
                class="photo-remove"
                :title="t('reports.removePhoto')"
                @click="removeEditPhoto(idx)"
              >
                ×
              </button>
            </div>
          </div>
          <span
            v-if="editUploadingPhotos"
            class="body-xs"
            style="color:var(--color-on-surface-variant)"
          >{{ t('reports.photoUploading') }}</span>
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('reports.attachedPhotos') }}</label>
          <CPageLoader v-if="loadingEditAttachedPhotos" />
          <div
            v-else-if="editAttachedPhotos.length"
            class="photo-gallery"
          >
            <div
              v-for="item in editAttachedPhotos"
              :key="item.doc.id"
              class="photo-gallery-item"
            >
              <img
                :src="item.url"
                :alt="item.doc.original_name"
                class="photo-gallery-thumb"
              />
              <button
                type="button"
                class="photo-gallery-delete"
                :title="t('reports.deletePhoto')"
                :disabled="deletingPhoto"
                @click="openEditPhotoAction(item.doc)"
              >
                <CIconTrash />
              </button>
            </div>
          </div>
          <span
            v-else
            class="body-xs"
            style="color:var(--color-on-surface-variant)"
          >{{ t('reports.noAttachedPhotos') }}</span>
        </div>
      </form>
      <template #footer>
        <CFormFeedback
          :status="editFeedback.status.value"
          :message="editFeedback.message.value"
        />
        <CBtn
          v-if="reportToEdit"
          variant="danger"
          @click="openDeleteFromEdit"
        >
          <CIconTrash />
          {{ t('common.delete') }}
        </CBtn>
        <CBtn
          variant="ghost"
          @click="reportToEdit = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="editSaving || editUploadingPhotos"
          :closing="editFeedback.status.value === 'success'"
          @click="submitEdit"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Delete report confirmation modal -->
    <CModal
      :open="!!reportToDelete"
      :title="t('reports.confirmDeleteTitle')"
      size="sm"
      @update:open="v => !v && (reportToDelete = null)"
    >
      <p class="body-xs">
        {{ t('reports.confirmDeleteMessage', { title: reportToDelete?.title ?? '' }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="deleteFeedback.status.value"
          :message="deleteFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="reportToDelete = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="deleting"
          :closing="deleteFeedback.status.value === 'success'"
          @click="submitDelete"
        >
          {{ t('common.delete') }}
        </CBtn>
      </template>
    </CModal>
    <CModal
      :open="!!reportPhotoGalleryTarget"
      :title="reportPhotoGalleryTarget?.title ?? ''"
      size="md"
      @update:open="value => !value && closeReportPhotoGallery()"
    >
      <CPageLoader v-if="reportPhotosLoading" />
      <div
        v-else-if="reportPhotos.length"
        class="report-photo-gallery-modal"
      >
        <button
          v-for="(item, index) in reportPhotos"
          :key="item.doc.id"
          type="button"
          class="report-photo-gallery-modal__item"
          @click="openReportPhotoViewer(index)"
        >
          <img
            :src="item.url"
            :alt="item.doc.original_name"
            class="report-photo-gallery-modal__thumb"
          />
        </button>
      </div>
      <p
        v-else
        class="body-xs report-photo-gallery-modal__empty"
      >
        {{ t('reports.noPhotos') }}
      </p>
    </CModal>

    <CModal
      :open="activeReportPhotoIndex !== null"
      :title="activeReportPhoto?.doc.original_name ?? reportPhotoGalleryTarget?.title ?? ''"
      size="xl"
      @update:open="value => !value && closeReportPhotoViewer()"
    >
      <div
        v-if="activeReportPhoto"
        class="report-photo-viewer"
      >
        <img
          :src="activeReportPhoto.url"
          :alt="activeReportPhoto.doc.original_name"
          class="report-photo-viewer__image"
        />
        <div class="report-photo-viewer__nav-row">
          <CBtn
            variant="ghost"
            size="sm"
            class="report-photo-viewer__nav"
            :disabled="reportPhotos.length <= 1"
            @click="showPreviousReportPhoto"
          >
            <CIconArrowLeft />
          </CBtn>
          <CBtn
            variant="ghost"
            size="sm"
            class="report-photo-viewer__nav"
            :disabled="reportPhotos.length <= 1"
            @click="showNextReportPhoto"
          >
            <CIconArrowRight />
          </CBtn>
        </div>
      </div>
      <p
        v-if="activeReportPhoto"
        class="body-xs report-photo-viewer__caption"
      >
        {{ activeReportPhotoPosition }} / {{ reportPhotos.length }} · {{ activeReportPhoto.doc.original_name }}
      </p>
    </CModal>

    <CDocumentLinkActionModal
      :open="showPhotoActionModal"
      :title="t('reports.deletePhoto')"
      :description="t('reports.deletePhotoDetachMessage', { name: photoActionTarget?.document.original_name ?? '', title: reportToEdit?.title ?? '' })"
      :warning="t('reports.deletePhotoConfirm')"
      :loading="deletingPhoto"
      :status="photoActionFeedback.status.value"
      :message="photoActionFeedback.message.value"
      @update:open="value => !value && closePhotoActionModal()"
      @unlink="handlePhotoAction(false)"
      @unlink-and-delete="handlePhotoAction(true)"
    />
  </div>
</template>

<style scoped>
/* ── Report cards ───────────────────────────────────────────────── */
.report-list { display: flex; flex-direction: column; gap: var(--space-3); }
.photo-preview-strip {
  display: flex; flex-wrap: wrap; gap: var(--space-2); margin-top: var(--space-2);
}
.photo-preview-item {
  position: relative; width: 64px; height: 64px; flex-shrink: 0;
}
.photo-thumb {
  width: 64px; height: 64px; object-fit: cover;
  border-radius: var(--radius-sm); border: 1px solid var(--color-outline-variant);
}
.photo-gallery-thumb {
  width: 120px; height: 120px; object-fit: cover; display: block;
  border-radius: var(--radius-sm); border: 1px solid var(--color-outline-variant);
  cursor: pointer; transition: opacity 0.15s;
}
.photo-gallery-thumb:hover { opacity: 0.85; }
.photo-gallery-delete {
  position: absolute; top: -6px; right: -6px;
  background: var(--color-error); color: #fff; border: none;
  border-radius: 50%; width: 22px; height: 22px;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; padding: 3px;
  min-height: unset; min-width: unset;
  opacity: 0.9; transition: opacity 0.15s;
}
.photo-gallery-delete svg { width: var(--icon-xs); height: var(--icon-xs); }
.photo-gallery-delete:hover { opacity: 1; }

.report-photo-gallery-modal {
  --report-photo-thumb-size: 120px;
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.report-photo-gallery-modal__item {
  border: none;
  background: none;
  padding: 0;
  cursor: pointer;
  max-width: var(--report-photo-thumb-size);
  flex: 0 0 var(--report-photo-thumb-size);
}

.report-photo-gallery-modal__thumb {
  width: var(--report-photo-thumb-size);
  height: var(--report-photo-thumb-size);
  object-fit: cover;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-outline-variant);
  transition: opacity 0.15s;
}

.report-photo-gallery-modal__thumb:hover {
  opacity: 0.85;
}

.report-photo-gallery-modal__empty {
  margin: 0;
  color: var(--color-on-surface-variant);
}

.report-photo-viewer {
  --report-photo-viewer-height: min(78vh, 760px);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  align-items: center;
}

.report-photo-viewer__image {
  width: 100%;
  max-height: var(--report-photo-viewer-height);
  object-fit: contain;
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
}

.report-photo-viewer__nav-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
}

.report-photo-viewer__nav {
  align-self: center;
}

.report-photo-viewer__caption {
  margin: var(--space-3) 0 0;
  text-align: center;
  color: var(--color-on-surface-variant);
}

@media (max-width: 720px) {
  .report-photo-gallery-modal {
    justify-content: flex-start;
  }
}
</style>
