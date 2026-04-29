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
import type { Report, ReportStatus, Unit } from '@/types/domain'

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

// ── Create report modal ───────────────────────────────────────────
const showCreateModal = ref(false)
const createFeedback = useFormFeedback()
const saving = ref(false)
const createForm = ref({ title: '', description: '', unit_id: '' })

// Photo selection state
const selectedPhotos = ref<File[]>([])
const photoInputRef = ref<HTMLInputElement | null>(null)
const photoDragOver = ref(false)
const uploadingPhotos = ref(false)

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
const convertFeedback = useFormFeedback()
const converting = ref(false)
const { schedule: scheduleConvertClose, forceExpire: forceConvertClose } = useAutoClose()
watch(reportToConvert, (r) => { if (!r) forceConvertClose() })

function openConvertModal(report: Report) {
  reportToConvert.value = report
  convertFeedback.reset()
}

async function submitConvert() {
  if (convertFeedback.status.value === 'success') { forceConvertClose(); return }
  if (!reportToConvert.value) return
  converting.value = true
  convertFeedback.reset()
  try {
    const res = await reportsApi.convertToTask(reportToConvert.value.id)
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
    editFeedback.setSuccess(t('reports.editSuccess'))
    scheduleEditClose(() => { reportToEdit.value = null; reload() })
  } catch (err) {
    editFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    editSaving.value = false
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
              <span
                v-if="r.document_count > 0"
                class="dim photo-count"
              >
                📎 {{ t('reports.photoCount', r.document_count) }}
              </span>
            </div>

            <!-- Card actions (manager / board only) -->
            <div
              v-if="canManage"
              class="report-card-actions"
            >
              <!-- Edit: always available -->
              <CBtn
                variant="ghost"
                size="sm"
                @click="openEditModal(r)"
              >
                <CIconEdit />
                {{ t('common.edit') }}
              </CBtn>
              <!-- Status change: non-closed only -->
              <CBtn
                v-if="r.status !== 'closed'"
                variant="ghost"
                size="sm"
                @click="openStatusModal(r)"
              >
                <CIconRefresh />
                {{ t('reports.editStatus') }}
              </CBtn>
              <!-- View Task if already converted, Convert if not closed and not yet converted -->
              <CBtn
                v-if="r.task_id"
                variant="ghost"
                size="sm"
                @click="router.push({ name: 'task-detail', params: { id: r.task_id } })"
              >
                <CIconCheckCircle />
                {{ t('reports.viewTask') }}
              </CBtn>
              <CBtn
                v-else-if="r.status !== 'closed'"
                variant="ghost"
                size="sm"
                @click="openConvertModal(r)"
              >
                <CIconCheckCircle />
                {{ t('reports.convertToTask') }}
              </CBtn>
              <!-- Delete: on card only when closed (otherwise accessible from edit modal) -->
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
          :loading="editSaving"
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
.report-card-actions { display: flex; gap: var(--space-2); flex-wrap: wrap; padding-top: var(--space-1); }

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
  position: absolute; top: -4px; right: -4px;
  width: 14px; height: 14px; border-radius: 50%;
  background: var(--color-error); color: #fff;
  border: none; cursor: pointer; font-size: 10px; line-height: 1;
  display: flex; align-items: center; justify-content: center;
  padding: 0;
}
.photo-remove:hover { background: var(--color-error-dark, var(--color-error)); opacity: 0.85; }
</style>
