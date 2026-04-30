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
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { buildingApi, documentsApi, reportsApi, unitsApi } from '@/api'
import type { Document, Report, ReportStatus, TaskPriority, Unit } from '@/types/domain'

const { t } = useI18n()
const { formatRelative } = useFormatters()
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

    // Upload photos sequentially — failure is reported but does not hide the success
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
    createFeedback.setError(toErrorMessage(err, t('common.error')))
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
    updateFeedback.setError(toErrorMessage(err, t('common.error')))
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
    convertFeedback.setError(toErrorMessage(err, t('common.error')))
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
    editFeedback.setError(toErrorMessage(err, t('common.error')))
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
    deleteFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    deleting.value = false
  }
}

// ── View photos modal ─────────────────────────────────────────────
const viewPhotosReport = ref<Report | null>(null)
const viewPhotosList = ref<{ doc: Document; url: string }[]>([])
const viewPhotosLoading = ref(false)
const lightboxIndex = ref<number | null>(null)
const deletingPhoto = ref(false)

// ── Delete photo confirmation ─────────────────────────────────────
const photoToDeleteIdx = ref<number | null>(null)
const deletePhotoFeedback = useFormFeedback()

function confirmDeletePhoto(idx: number) {
  photoToDeleteIdx.value = idx
  deletePhotoFeedback.reset()
}

async function executeDeletePhoto() {
  if (photoToDeleteIdx.value === null || !viewPhotosReport.value || deletingPhoto.value) return
  const idx = photoToDeleteIdx.value
  const item = viewPhotosList.value[idx]
  if (!item) return
  deletingPhoto.value = true
  deletePhotoFeedback.reset()
  try {
    await reportsApi.detachDocument(viewPhotosReport.value.id, item.doc.id)
    await documentsApi.delete(item.doc.id)
    viewPhotosList.value.splice(idx, 1)
    // Update document_count on the card
    const r = allReports.value.find(x => x.id === viewPhotosReport.value!.id)
    if (r && typeof r.document_count === 'number' && r.document_count > 0) {
      r.document_count -= 1
    }
    // Adjust lightbox index if needed
    if (lightboxIndex.value !== null) {
      if (viewPhotosList.value.length === 0) {
        lightboxIndex.value = null
      } else if (lightboxIndex.value >= viewPhotosList.value.length) {
        lightboxIndex.value = viewPhotosList.value.length - 1
      }
    }
    photoToDeleteIdx.value = null
  } catch {
    deletePhotoFeedback.setError(t('common.error'))
  } finally {
    deletingPhoto.value = false
  }
}

async function openViewPhotos(report: Report) {
  viewPhotosReport.value = report
  viewPhotosList.value = []
  viewPhotosLoading.value = true
  try {
    const docsRes = await reportsApi.listDocuments(report.id)
    const photos = await Promise.all(
      docsRes.data.map(async (d) => {
        const urlRes = await documentsApi.getDownloadUrl(d.id)
        return { doc: d, url: urlRes.data.url }
      }),
    )
    viewPhotosList.value = photos
  } catch {
    viewPhotosList.value = []
  } finally {
    viewPhotosLoading.value = false
  }
}

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

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
onMounted(reload)

// ── Derived ───────────────────────────────────────────────────────
const reports = computed(() =>
  statusFilter.value === 'all'
    ? allReports.value
    : allReports.value.filter(r => r.status === statusFilter.value),
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
          <CIconChevronRight
            width="14"
            height="14"
          />
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
        <div class="view-header">
          <div>
            <h1 class="display-md">
              {{ t('reports.title') }}
            </h1>
            <p
              class="body-md"
              style="color:var(--color-on-surface-variant)"
            >
              {{ t('reports.activeCount', activeCount) }}
            </p>
          </div>
        </div>

        <!-- Filters -->
        <div class="tabs">
          <button
            v-for="item in [
              { key: 'all', label: t('common.all'), count: allReports.length },
              { key: 'open', label: t('reports.filters.open'), count: counts.open },
              { key: 'in_progress', label: t('reports.filters.inProgress'), count: counts.in_progress },
              { key: 'closed', label: t('reports.filters.closed'), count: counts.closed },
            ]"
            :key="item.key"
            :class="['tab', { 'tab--active': statusFilter === item.key }]"
            @click="statusFilter = item.key as typeof statusFilter"
          >
            {{ item.label }}
            <span class="tab-count">{{ item.count }}</span>
          </button>
        </div>

        <!-- List -->
        <div class="report-list">
          <CCard
            v-for="r in reports"
            :key="r.id"
            :shadow="r.status !== 'closed'"
            :class="['report-card', { 'card--terminated': r.status === 'closed' }]"
          >
            <div class="report-header">
              <div class="report-icon">
                <CIconInfo />
              </div>
              <div class="report-meta-top">
                <CBadge
                  :variant="statusVariant(r.status)"
                  dot
                >
                  {{ t(`reports.statuses.${r.status}`) }}
                </CBadge>
                <span class="report-date">{{ formatRelative(r.created_at) }}</span>
              </div>
            </div>

            <h3 class="title-sm report-title">
              {{ r.title }}
            </h3>
            <p
              v-if="r.description"
              class="body-sm report-desc"
            >
              {{ r.description }}
            </p>

            <div class="report-footer">
              <span class="dim">👤 {{ authorName(r) }}</span>
              <span class="dim">🏠 {{ unitLabel(r.unit_id) }}</span>
              <button
                v-if="r.document_count > 0"
                class="dim photo-count photo-count--btn"
                @click="openViewPhotos(r)"
              >
                👁️ {{ t('reports.photoCount', r.document_count) }}
              </button>
            </div>

            <!-- Card actions (manager / board only) -->
            <div
              v-if="canManage"
              class="report-card-actions"
            >
              <!-- Row 1: status change (non-closed only) -->
              <div
                v-if="r.status !== 'closed'"
                class="report-card-actions-row"
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

              <!-- Row 2: edit + convert/view-task (non-closed) | edit + delete (closed) -->
              <div class="report-card-actions-row">
                <CBtn
                  variant="ghost"
                  size="sm"
                  @click="openEditModal(r)"
                >
                  <CIconEdit />
                  {{ t('common.edit') }}
                </CBtn>
                <!-- View Task if already converted -->
                <CBtn
                  v-if="r.task_id"
                  variant="ghost"
                  size="sm"
                  @click="router.push({ name: 'task-detail', params: { id: r.task_id } })"
                >
                  <CIconCheckCircle />
                  {{ t('reports.viewTask') }}
                </CBtn>
                <!-- Convert to Task if not yet converted and not closed -->
                <CBtn
                  v-else-if="r.status !== 'closed'"
                  variant="ghost"
                  size="sm"
                  @click="openConvertModal(r)"
                >
                  <CIconCheckCircle />
                  {{ t('reports.convertToTask') }}
                </CBtn>
                <!-- Delete: on card only when closed -->
                <CBtn
                  v-if="r.status === 'closed'"
                  variant="danger"
                  size="sm"
                  @click="openDeleteModal(r)"
                >
                  <CIconTrash />
                  {{ t('common.delete') }}
                </CBtn>
              </div>
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
        <div class="form-field">
          <label class="form-label">{{ t('reports.titleLabel') }}</label>
          <input
            v-model="createForm.title"
            type="text"
            class="form-input"
            :placeholder="t('reports.titleLabel')"
            required
          />
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('reports.descriptionLabel') }}</label>
          <textarea
            v-model="createForm.description"
            class="form-input"
            rows="3"
          ></textarea>
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('reports.unitLabel2') }}</label>
          <select
            v-model="createForm.unit_id"
            class="form-input"
          >
            <option value="">
              {{ t('reports.unitPlaceholder') }}
            </option>
            <option
              v-for="u in allUnits"
              :key="u.id"
              :value="u.id"
            >
              {{ t('reports.unitLabel', { number: u.unit_number }) }}
            </option>
          </select>
        </div>

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
            :class="['photo-dropzone', { 'photo-dropzone--over': photoDragOver }]"
            role="button"
            tabindex="0"
            @click="photoInputRef?.click()"
            @keydown.enter="photoInputRef?.click()"
            @dragover.prevent="photoDragOver = true"
            @dragleave.prevent="photoDragOver = false"
            @drop.prevent="onPhotoDrop"
          >
            <CIconUpload
              width="20"
              height="20"
              style="color:var(--color-on-surface-variant)"
            />
            <span class="body-xs photo-hint">{{ t('reports.photoHint') }}</span>
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
      <div class="form-field">
        <label class="form-label">{{ t('reports.statusLabel') }}</label>
        <select
          v-model="statusForm"
          class="form-input"
        >
          <option
            v-for="s in REPORT_STATUSES"
            :key="s"
            :value="s"
          >
            {{ t(`reports.statuses.${s}`) }}
          </option>
        </select>
      </div>
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
      <p class="body-sm">
        {{ t('reports.confirmConvertMessage', { title: reportToConvert?.title ?? '' }) }}
      </p>
      <div class="form-field">
        <label class="form-label">{{ t('tasks.priorityLabel') }}</label>
        <select
          v-model="convertPriority"
          class="form-input"
        >
          <option
            v-for="priority in ['low', 'normal', 'high', 'urgent'] as TaskPriority[]"
            :key="priority"
            :value="priority"
          >
            {{ t(`tasks.priorities.${priority}`) }}
          </option>
        </select>
      </div>
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
        <div class="form-field">
          <label class="form-label">{{ t('reports.titleLabel') }}</label>
          <input
            v-model="editForm.title"
            type="text"
            class="form-input"
            :placeholder="t('reports.titleLabel')"
            required
          />
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('reports.descriptionLabel') }}</label>
          <textarea
            v-model="editForm.description"
            class="form-input"
            rows="3"
          ></textarea>
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('reports.unitLabel2') }}</label>
          <select
            v-model="editForm.unit_id"
            class="form-input"
          >
            <option value="">
              {{ t('reports.unitPlaceholder') }}
            </option>
            <option
              v-for="u in allUnits"
              :key="u.id"
              :value="u.id"
            >
              {{ t('reports.unitLabel', { number: u.unit_number }) }}
            </option>
          </select>
        </div>

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
            :class="['photo-dropzone', { 'photo-dropzone--over': editPhotoDragOver }]"
            role="button"
            tabindex="0"
            @click="editPhotoInputRef?.click()"
            @keydown.enter="editPhotoInputRef?.click()"
            @dragover.prevent="editPhotoDragOver = true"
            @dragleave.prevent="editPhotoDragOver = false"
            @drop.prevent="onEditPhotoDrop"
          >
            <CIconUpload
              width="20"
              height="20"
              style="color:var(--color-on-surface-variant)"
            />
            <span class="body-xs photo-hint">{{ t('reports.photoHint') }}</span>
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
      </form>
      <template #footer>
        <CFormFeedback
          :status="editFeedback.status.value"
          :message="editFeedback.message.value"
        />
        <CBtn
          v-if="reportToEdit && reportToEdit.status !== 'closed'"
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
      <p class="body-sm">
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
    <!-- Delete photo confirmation modal -->
    <CModal
      :open="photoToDeleteIdx !== null"
      :title="t('reports.deletePhoto')"
      size="sm"
      @update:open="v => !v && (photoToDeleteIdx = null)"
    >
      <p class="body-sm">
        {{ t('reports.deletePhotoConfirm') }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="deletePhotoFeedback.status.value"
          :message="deletePhotoFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="photoToDeleteIdx = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="deletingPhoto"
          @click="executeDeletePhoto"
        >
          {{ t('common.delete') }}
        </CBtn>
      </template>
    </CModal>

    <!-- View photos modal -->
    <CModal
      :open="!!viewPhotosReport"
      :title="viewPhotosReport?.title ?? ''"
      size="md"
      @update:open="v => !v && (viewPhotosReport = null)"
    >
      <div
        v-if="viewPhotosLoading"
        class="photos-loading body-sm"
      >
        {{ t('common.loading') }}
      </div>
      <div
        v-else-if="viewPhotosList.length"
        class="photo-gallery"
      >
        <div
          v-for="(item, idx) in viewPhotosList"
          :key="item.doc.id"
          class="photo-gallery-item"
        >
          <img
            :src="item.url"
            :alt="item.doc.original_name"
            class="photo-gallery-thumb"
            @click="lightboxIndex = idx"
          />
          <button
            v-if="canManage"
            class="photo-gallery-delete"
            :title="t('reports.deletePhoto')"
            :disabled="deletingPhoto"
            @click.stop="confirmDeletePhoto(idx)"
          >
            <CIconTrash />
          </button>
        </div>
      </div>
      <p
        v-else
        class="body-sm"
        style="color:var(--color-on-surface-variant)"
      >
        {{ t('reports.noPhotos') }}
      </p>
      <template #footer>
        <CBtn
          variant="ghost"
          @click="viewPhotosReport = null"
        >
          {{ t('common.close') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Lightbox overlay -->
    <Teleport to="body">
      <div
        v-if="lightboxIndex !== null && viewPhotosList[lightboxIndex]"
        class="lightbox-overlay"
        @click.self="lightboxIndex = null"
      >
        <button
          class="lightbox-close"
          @click="lightboxIndex = null"
        >
          ✕
        </button>
        <button
          v-if="canManage"
          class="lightbox-delete"
          :title="t('reports.deletePhoto')"
          :disabled="deletingPhoto"
          @click.stop="confirmDeletePhoto(lightboxIndex!)"
        >
          <CIconTrash />
        </button>
        <button
          v-if="lightboxIndex > 0"
          class="lightbox-nav lightbox-nav--prev"
          @click="lightboxIndex--"
        >
          ‹
        </button>
        <img
          :src="viewPhotosList[lightboxIndex].url"
          :alt="viewPhotosList[lightboxIndex].doc.original_name"
          class="lightbox-img"
          @click.stop
        />
        <button
          v-if="lightboxIndex < viewPhotosList.length - 1"
          class="lightbox-nav lightbox-nav--next"
          @click="lightboxIndex++"
        >
          ›
        </button>
        <span class="lightbox-counter body-sm">
          {{ lightboxIndex + 1 }} / {{ viewPhotosList.length }}
        </span>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
/* ── Report cards ───────────────────────────────────────────────── */
.report-list { display: flex; flex-direction: column; gap: var(--space-3); }
.report-card { display: flex; flex-direction: column; gap: var(--space-2); }

.report-header { display: flex; align-items: center; gap: var(--space-3); }
.report-icon {
  width: 36px; height: 36px; border-radius: var(--radius-sm);
  background: var(--color-surface-container);
  display: flex; align-items: center; justify-content: center; flex-shrink: 0;
}
.report-icon svg { width: 18px; height: 18px; color: var(--color-on-surface-variant); }
.report-meta-top { display: flex; align-items: center; gap: var(--space-2); }
.report-date { font-size: var(--text-2xs); color: var(--color-on-surface-variant); }
.report-title { color: var(--color-on-surface); }
.report-desc { color: var(--color-on-surface-variant); }
.report-footer { display: flex; gap: 14px; font-size: var(--text-2xs); flex-wrap: wrap; }
.dim { color: var(--color-on-surface-variant); }
.photo-count { display: flex; align-items: center; gap: 4px; }
.photo-count--btn {
  background: none; border: none; padding: 0; cursor: pointer;
  font-size: inherit; color: var(--color-on-surface-variant);
  text-decoration: underline dotted;
  min-height: unset; min-width: unset;
}
.photo-count--btn:hover { color: var(--color-primary); }
.report-card-actions { display: flex; flex-direction: column; gap: var(--space-1); padding-top: var(--space-1); }
.report-card-actions-row { display: flex; gap: var(--space-2); flex-wrap: wrap; }

/* Photo picker */
.photo-dropzone {
  border: 1.5px dashed var(--color-outline);
  border-radius: var(--radius-sm);
  padding: var(--space-4) var(--space-3);
  display: flex; align-items: center; justify-content: center; gap: var(--space-2);
  cursor: pointer; transition: background 0.15s, border-color 0.15s;
  user-select: none;
}
.photo-dropzone:hover,
.photo-dropzone--over {
  background: var(--color-surface-container);
  border-color: var(--color-primary);
}
.photo-hint { color: var(--color-on-surface-variant); }

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
.photo-remove {
  position: absolute; top: -6px; right: -6px;
  width: 22px; height: 22px; border-radius: 50%;
  background: var(--color-error); color: #fff;
  border: none; cursor: pointer; font-size: 14px; line-height: 1;
  display: flex; align-items: center; justify-content: center;
  padding: 0;
  /* Override the global 44px touch-target minimum for this decorative badge */
  min-height: unset; min-width: unset;
}
.photo-remove:hover { background: var(--color-error-dark, var(--color-error)); opacity: 0.85; }

/* Photo gallery (view photos modal) */
.photos-loading { color: var(--color-on-surface-variant); padding: var(--space-4) 0; text-align: center; }
.photo-gallery { display: flex; flex-wrap: wrap; gap: var(--space-2); }
.photo-gallery-item {
  position: relative; border-radius: var(--radius-sm); overflow: visible;
  flex-shrink: 0;
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
.photo-gallery-delete svg { width: 14px; height: 14px; }
.photo-gallery-delete:hover { opacity: 1; }

/* Lightbox */
.lightbox-overlay {
  position: fixed; inset: 0; z-index: 9999;
  background: rgba(0, 0, 0, 0.92);
  display: flex; align-items: center; justify-content: center;
}
.lightbox-img {
  max-width: 90vw; max-height: 90vh;
  object-fit: contain; border-radius: var(--radius-sm);
  box-shadow: 0 8px 40px rgba(0,0,0,0.6);
}
.lightbox-close {
  position: absolute; top: 16px; right: 20px;
  background: none; border: none; color: #fff;
  font-size: 28px; cursor: pointer; line-height: 1;
  min-height: unset; min-width: unset;
  opacity: 0.8;
}
.lightbox-close:hover { opacity: 1; }
.lightbox-delete {
  position: absolute; top: 16px; right: 60px;
  background: var(--color-error); border: none; color: #fff;
  cursor: pointer;
  min-height: unset; min-width: unset;
  opacity: 0.85; width: 36px; height: 36px;
  border-radius: var(--radius-sm);
  display: flex; align-items: center; justify-content: center;
  padding: 8px;
}
.lightbox-delete svg { width: 18px; height: 18px; }
.lightbox-delete:hover { opacity: 1; }
.lightbox-delete:disabled { opacity: 0.4; cursor: default; }
.lightbox-nav {
  position: absolute;
  background: rgba(255,255,255,0.15); border: none; color: #fff;
  font-size: 48px; line-height: 1; padding: 8px 16px;
  cursor: pointer; border-radius: var(--radius-sm);
  min-height: unset; min-width: unset;
  opacity: 0.7; transition: opacity 0.15s;
}
.lightbox-nav:hover { opacity: 1; }
.lightbox-nav--prev { left: 16px; }
.lightbox-nav--next { right: 16px; }
.lightbox-counter {
  position: absolute; bottom: 16px; left: 50%; transform: translateX(-50%);
  color: rgba(255,255,255,0.7);
}
</style>
