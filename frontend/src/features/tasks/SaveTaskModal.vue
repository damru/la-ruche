<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormRow from '@/components/molecules/CFormRow.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CDocumentLinkActionModal from '@/components/organisms/CDocumentLinkActionModal.vue'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import { contractorsApi, documentsApi, tasksApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormatters } from '@/composables/useFormatters'
import { extractFileExtension } from '@/utils/files'
import type { Contractor, Document, Task, TaskPriority } from '@/types/domain'

const props = defineProps<{
  open: boolean
  task: Task | null
  contractors: Contractor[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
  'delete': [task: Task]
  'document-detached': [taskId: string, documentId: string]
}>()

const { t } = useI18n()
const { formatDate, formatFileSize } = useFormatters()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const uploadingDocuments = ref(false)

const TASK_MODAL_ERROR_CLOSE_DELAY_MS = 2200

// ── Assignment mode ──────────────────────────────────────────────
const assignMode = ref<'none' | 'self' | 'contractor'>('none')

// ── Contractor autocomplete ──────────────────────────────────────
const contractorSearch = ref('')
const contractorSuggestions = ref<Contractor[]>([])
const showSuggestions = ref(false)
const selectedContractor = ref<Contractor | null>(null)
const createSuggestedContractor = ref(false)

function createDefaultContractorDraft() {
  return { name: '', email: '', phone: '' }
}
const contractorDraft = ref(createDefaultContractorDraft())

function emptyStringToUndefined(value: string) {
  return value.trim() || undefined
}

function contractorDomainLabel(c: Contractor): string {
  if (!c.domains.length) return ''
  return c.domains.map(d => t(`contractors.domains.${d}`, d)).join(', ')
}

const MAX_SUGGESTIONS = 10

function populateSuggestions(query: string) {
  const q = query.trim().toLowerCase()
  if (!q) {
    contractorSuggestions.value = props.contractors.slice(0, MAX_SUGGESTIONS)
  } else {
    contractorSuggestions.value = props.contractors
      .filter(c => c.name.toLowerCase().includes(q))
      .slice(0, MAX_SUGGESTIONS)
  }
  showSuggestions.value = contractorSuggestions.value.length > 0
}

const canSuggestContractorCreation = computed(() => (
  assignMode.value === 'contractor'
  && !selectedContractor.value
  && contractorSearch.value.trim().length > 0
  && contractorSuggestions.value.length === 0
))

function onContractorInput() {
  selectedContractor.value = null
  createSuggestedContractor.value = false
  populateSuggestions(contractorSearch.value)
}

function onContractorFocus() {
  populateSuggestions(contractorSearch.value)
}

function selectContractorSuggestion(c: Contractor) {
  selectedContractor.value = c
  contractorSearch.value = c.name
  createSuggestedContractor.value = false
  contractorDraft.value = createDefaultContractorDraft()
  showSuggestions.value = false
}

function suggestContractorCreation() {
  createSuggestedContractor.value = true
  contractorDraft.value = {
    name: contractorSearch.value.trim(),
    email: '',
    phone: '',
  }
  showSuggestions.value = false
}

function hideSuggestions() {
  window.setTimeout(() => { showSuggestions.value = false }, 100)
}

function clearContractor() {
  selectedContractor.value = null
  contractorSearch.value = ''
  createSuggestedContractor.value = false
  contractorDraft.value = createDefaultContractorDraft()
  showSuggestions.value = false
}

watch(assignMode, (newMode) => {
  if (newMode !== 'contractor') clearContractor()
})

async function resolveContractorSelection() {
  if (assignMode.value !== 'contractor') {
    return { contractorId: null, contractorName: null }
  }
  if (selectedContractor.value) {
    return { contractorId: selectedContractor.value.id, contractorName: selectedContractor.value.name }
  }
  if (createSuggestedContractor.value && contractorDraft.value.name) {
    const createdContractor = await contractorsApi.create({
      name: contractorDraft.value.name,
      email: emptyStringToUndefined(contractorDraft.value.email),
      phone: emptyStringToUndefined(contractorDraft.value.phone),
    })
    return {
      contractorId: createdContractor.data.id,
      contractorName: createdContractor.data.name,
    }
  }
  return {
    contractorId: null,
    contractorName: emptyStringToUndefined(contractorSearch.value) ?? null,
  }
}

// ── Form ─────────────────────────────────────────────────────────
const defaultForm = () => ({
  title: '',
  description: '',
  priority: 'normal' as string,
})

const taskPriorityOptions = computed(() =>
  (['low', 'normal', 'high', 'urgent'] as const).map(p => ({
    value: p,
    label: t(`tasks.priorities.${p}`),
  })),
)

const taskForm = ref(defaultForm())

// ── Documents ─────────────────────────────────────────────────────
const selectedDocuments = ref<File[]>([])
const loadingFormDocuments = ref(false)
const existingDocuments = ref<Document[]>([])
const pendingDocument = ref<Document | null>(null)
const documentActionFeedback = useFormFeedback()
const documentActionLoading = ref(false)


async function uploadDocument(taskId: string, file: File) {
  const extension = extractFileExtension(file.name)
  const mimeType = file.type || 'application/octet-stream'
  const uploadUrl = await documentsApi.getUploadUrl(extension, mimeType, file.size)
  await documentsApi.uploadToS3(file, uploadUrl.data.url, mimeType)
  const document = await documentsApi.confirmUpload({
    key: uploadUrl.data.key,
    bucket: uploadUrl.data.bucket,
    original_name: file.name,
    mime_type: mimeType,
    size_bytes: file.size,
    visible_roles: ['manager', 'board_member'],
  })
  await tasksApi.attachDocument(taskId, document.data.id)
}

async function attachSelectedDocuments(taskId: string) {
  const failed: string[] = []
  for (const doc of selectedDocuments.value) {
    try {
      await uploadDocument(taskId, doc)
    } catch {
      failed.push(doc.name)
    }
  }
  return failed
}

async function loadFormDocuments(taskId: string) {
  loadingFormDocuments.value = true
  documentActionFeedback.reset()
  try {
    const response = await tasksApi.listDocuments(taskId)
    existingDocuments.value = response.data
  } catch (err) {
    feedback.setError(toErrorMessage(err, t('tasks.documentsLoadError')))
  } finally {
    loadingFormDocuments.value = false
  }
}

async function downloadDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch {
    // download errors are non-critical in modal context
  }
}

function openDocumentActionModal(document: Document) {
  pendingDocument.value = document
  documentActionFeedback.reset()
}

function closeDocumentActionModal() {
  if (documentActionLoading.value) return
  pendingDocument.value = null
  documentActionFeedback.reset()
}

async function handleDocumentAction(deleteDocument: boolean) {
  if (!props.task || !pendingDocument.value) return
  documentActionLoading.value = true
  documentActionFeedback.reset()
  try {
    if (deleteDocument) {
      await documentsApi.delete(pendingDocument.value.id)
    } else {
      await tasksApi.detachDocument(props.task.id, pendingDocument.value.id)
    }
    const removedId = pendingDocument.value.id
    existingDocuments.value = existingDocuments.value.filter(d => d.id !== removedId)
    emit('document-detached', props.task.id, removedId)
    pendingDocument.value = null
  } catch (err) {
    documentActionFeedback.setError(
      toErrorMessage(
        err,
        deleteDocument ? t('common.documentDeleteError') : t('tasks.documentsDetachError'),
      ),
    )
  } finally {
    documentActionLoading.value = false
  }
}

// ── Open / reset ──────────────────────────────────────────────────
watch(() => props.open, async (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
  selectedDocuments.value = []
  existingDocuments.value = []
  pendingDocument.value = null
  if (props.task) {
    taskForm.value = {
      title: props.task.title,
      description: props.task.description ?? '',
      priority: props.task.priority,
    }
    if (props.task.contractor_id) {
      const c = props.contractors.find(c => c.id === props.task!.contractor_id) ?? null
      selectedContractor.value = c
      contractorSearch.value = c?.name ?? props.task.contractor_name ?? ''
      assignMode.value = 'contractor'
      createSuggestedContractor.value = false
      contractorDraft.value = createDefaultContractorDraft()
    } else if (props.task.contractor_name) {
      selectedContractor.value = null
      contractorSearch.value = props.task.contractor_name
      createSuggestedContractor.value = false
      contractorDraft.value = createDefaultContractorDraft()
      assignMode.value = 'contractor'
    } else if (props.task.status === 'in_progress') {
      selectedContractor.value = null
      contractorSearch.value = ''
      createSuggestedContractor.value = false
      contractorDraft.value = createDefaultContractorDraft()
      assignMode.value = 'self'
    } else {
      selectedContractor.value = null
      contractorSearch.value = ''
      createSuggestedContractor.value = false
      contractorDraft.value = createDefaultContractorDraft()
      assignMode.value = 'none'
    }
    await loadFormDocuments(props.task.id)
  } else {
    taskForm.value = defaultForm()
    selectedContractor.value = null
    contractorSearch.value = ''
    createSuggestedContractor.value = false
    contractorDraft.value = createDefaultContractorDraft()
    assignMode.value = 'none'
  }
})

// ── Save ──────────────────────────────────────────────────────────
async function handleSubmit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!taskForm.value.title.trim()) return
  saving.value = true
  feedback.reset()
  try {
    const { contractorId, contractorName } = await resolveContractorSelection()
    let taskId: string

    if (props.task) {
      let statusPatch: string | undefined
      if (assignMode.value === 'self') {
        statusPatch = 'in_progress'
      } else if (assignMode.value === 'none') {
        statusPatch = 'open'
      }
      await tasksApi.update(props.task.id, {
        title: taskForm.value.title,
        description: taskForm.value.description || null,
        priority: taskForm.value.priority,
        contractor_id: assignMode.value === 'contractor' ? contractorId : null,
        contractor_name: assignMode.value === 'contractor' ? contractorName : null,
        ...(statusPatch ? { status: statusPatch } : {}),
      })
      taskId = props.task.id
    } else {
      const response = await tasksApi.create({
        title: taskForm.value.title,
        description: taskForm.value.description || undefined,
        priority: taskForm.value.priority as TaskPriority,
        contractor_id: assignMode.value === 'contractor' ? contractorId : null,
        contractor_name: assignMode.value === 'contractor' ? contractorName : null,
      })
      taskId = response.data.id
    }

    if (selectedDocuments.value.length > 0) {
      uploadingDocuments.value = true
      const failed = await attachSelectedDocuments(taskId)
      uploadingDocuments.value = false

      if (failed.length > 0) {
        feedback.setError(`${t('tasks.documentUploadError')}: ${failed.join(', ')}`)
        if (props.task) {
          await loadFormDocuments(taskId)
          emit('saved')
        } else {
          window.setTimeout(() => {
            emit('update:open', false)
            emit('saved')
          }, TASK_MODAL_ERROR_CLOSE_DELAY_MS)
        }
        return
      }
    }

    feedback.setSuccess(t('common.saved'))
    emit('saved')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    saving.value = false
    uploadingDocuments.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="task ? t('tasks.editTask') : t('tasks.create')"
    size="md"
    :closable="!saving"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="task-form"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="taskForm.title"
        :label="t('common.title')"
        :placeholder="t('common.title')"
        required
      />
      <CFormTextarea
        v-model="taskForm.description"
        :label="t('common.description')"
        :rows="3"
      />
      <CFormSelect
        v-model="taskForm.priority"
        :label="t('tasks.priorityLabel')"
        :options="taskPriorityOptions"
      />
      <div
        v-if="assignMode === 'contractor' && createSuggestedContractor"
        class="flex flex-col gap-1 contractor-create-box"
      >
        <label class="text-sm font-medium text-on-surface-variant">{{ t('tasks.contractorCreateLabel') }}</label>
        <CFormRow :cols="2">
          <CFormInput
            v-model="contractorDraft.email"
            :label="t('contractors.email')"
            type="email"
            :placeholder="t('contractors.emailPlaceholder')"
          />
          <CFormInput
            v-model="contractorDraft.phone"
            :label="t('common.phone')"
            type="tel"
            :placeholder="t('common.phonePlaceholder')"
          />
        </CFormRow>
        <span class="text-xs text-on-surface-variant">
          {{ t('tasks.contractorCreateHelp') }}
        </span>
      </div>
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('common.attachments') }}</label>
        <CFileDropzone
          v-model:files="selectedDocuments"
          :hint="task ? t('claims.editDocumentHint') : t('tasks.documentHint')"
          :remove-label="t('common.remove')"
        />

        <div
          v-if="task"
          class="task-existing-documents"
        >
          <p class="text-xs text-on-surface-variant task-existing-documents__hint">
            {{ t('tasks.attachedDocuments') }}
          </p>
          <CPageLoader v-if="loadingFormDocuments" />
          <div
            v-else-if="existingDocuments.length === 0"
            class="task-existing-documents__empty"
          >
            {{ t('tasks.noAttachedDocuments') }}
          </div>
          <div
            v-else
            class="task-existing-documents__list"
          >
            <div
              v-for="document in existingDocuments"
              :key="document.id"
              class="task-existing-documents__row"
            >
              <div class="task-existing-documents__meta">
                <div class="task-existing-documents__name">
                  <CIconFile
                    class="icon-sm"
                  />
                  <span>{{ document.original_name }}</span>
                </div>
                <div class="task-existing-documents__details">
                  <span>{{ formatFileSize(document.size_bytes) }}</span>
                  <span>·</span>
                  <span>{{ formatDate(document.created_at) }}</span>
                </div>
              </div>
              <div class="task-existing-documents__actions">
                <CBtn
                  variant="ghost"
                  size="sm"
                  :icon="CIconDownload"
                  :title="t('common.download')"
                  @click="downloadDocument(document.id)"
                />
                <CBtn
                  variant="danger"
                  size="sm"
                  :icon="CIconTrash"
                  :title="t('tasks.detachDocument')"
                  :loading="documentActionLoading && pendingDocument?.id === document.id"
                  @click="openDocumentActionModal(document)"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
      <!-- Assignment mode: radio (visible for new tasks, open tasks, and in_progress tasks) -->
      <div
        v-if="!task || ['open', 'in_progress', 'awaiting_vendor', 'awaiting_validation'].includes(task.status)"
        class="flex flex-col gap-1"
      >
        <label class="text-sm font-medium text-on-surface-variant">{{ t('tasks.assignmentSection') }}</label>
        <div class="assignment-options">
          <label class="checkbox-label">
            <input
              v-model="assignMode"
              type="radio"
              value="none"
            />
            {{ t('tasks.assignNone') }}
          </label>
          <label class="checkbox-label">
            <input
              v-model="assignMode"
              type="radio"
              value="self"
            />
            {{ t('tasks.assignSelf') }}
          </label>
          <label class="checkbox-label">
            <input
              v-model="assignMode"
              type="radio"
              value="contractor"
            />
            {{ t('tasks.assignContractor') }}
          </label>
        </div>
      </div>

      <!-- Contractor assignment with autocomplete (only when assignMode === 'contractor') -->
      <div
        v-if="assignMode === 'contractor'"
        class="flex flex-col gap-1 contractor-autocomplete"
      >
        <CFormInput
          v-model="contractorSearch"
          :label="t('tasks.contractorLabel')"
          :placeholder="t('tasks.contractorPlaceholder')"
          autocomplete="off"
          :hint="selectedContractor
            ? (selectedContractor.email || selectedContractor.phone || '')
            : (contractorSearch.trim() ? t('tasks.contractorFreeInputHint') : '')"
          @focus="onContractorFocus"
          @input="onContractorInput"
          @blur="hideSuggestions"
        >
          <template
            v-if="selectedContractor"
            #suffix
          >
            <button
              type="button"
              class="contractor-clear"
              :aria-label="t('common.clear')"
              @click="clearContractor"
            >
              ×
            </button>
          </template>
        </CFormInput>
        <ul
          v-if="showSuggestions || canSuggestContractorCreation"
          class="autocomplete-list"
        >
          <li
            v-for="c in contractorSuggestions"
            :key="c.id"
            class="autocomplete-item"
            @mousedown.prevent="selectContractorSuggestion(c)"
          >
            <span class="autocomplete-name">{{ c.name }}</span>
            <span
              v-if="c.domains.length"
              class="autocomplete-meta"
            >{{ contractorDomainLabel(c) }}</span>
          </li>
          <li
            v-if="canSuggestContractorCreation"
            class="autocomplete-item autocomplete-item--action"
            @mousedown.prevent="suggestContractorCreation"
          >
            <span class="autocomplete-name">{{ t('tasks.contractorCreateSuggestion', { name: contractorSearch.trim() }) }}</span>
            <span class="autocomplete-meta">{{ t('tasks.contractorCreateSuggestionHint') }}</span>
          </li>
        </ul>
      </div>
    </CForm>
    <template #footer>
      <CFormActions
        :status="feedback.status.value"
        :message="feedback.message.value"
      >
        <CBtn
          v-if="task"
          variant="danger"
          :icon="CIconTrash"
          :title="t('common.delete')"
          :loading="saving"
          class="mr-auto"
          @click="$emit('delete', task)"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="saving || uploadingDocuments"
          :closing="feedback.status.value === 'success'"
          type="submit"
          form="task-form"
        />
      </CFormActions>
    </template>
  </CModal>

  <CDocumentLinkActionModal
    :open="!!pendingDocument"
    :title="t('tasks.documentsDetachTitle')"
    :description="t('tasks.documentsDetachMessage', { name: pendingDocument?.original_name ?? '', title: task?.title ?? '' })"
    :warning="t('common.documentDeleteWarning', { name: pendingDocument?.original_name ?? '' })"
    :loading="documentActionLoading"
    :status="documentActionFeedback.status.value"
    :message="documentActionFeedback.message.value"
    @update:open="value => !value && closeDocumentActionModal()"
    @unlink="handleDocumentAction(false)"
    @unlink-and-delete="handleDocumentAction(true)"
  />
</template>

<style scoped>
.assignment-options {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-2) 0;
}

.contractor-autocomplete { position: relative; }

.autocomplete-list {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-md);
  list-style: none;
  margin: 2px 0 0;
  padding: var(--space-1) 0;
  z-index: var(--z-nav);
  max-height: 200px;
  overflow-y: auto;
}

.autocomplete-item {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  cursor: pointer;
  font-size: var(--text-sm);
}

.autocomplete-item:hover { background: var(--color-surface-container); }
.autocomplete-name { font-weight: var(--fw-medium); }
.autocomplete-meta { font-size: var(--text-xs); color: var(--color-on-surface-variant); }
.autocomplete-item--action {
  align-items: flex-start;
  flex-direction: column;
}

.contractor-clear {
  background: none;
  border: none;
  cursor: pointer;
  font-size: var(--text-lg);
  color: var(--color-on-surface-variant);
  padding: 0 var(--space-1);
  pointer-events: auto;
  min-height: unset;
  min-width: unset;
}

.contractor-create-box {
  padding: var(--space-3);
  border: 1px dashed var(--color-outline);
  border-radius: var(--radius-xl);
  background: var(--color-surface-container-lowest);
}

.task-existing-documents__list {
  margin-top: var(--space-3);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.task-existing-documents__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-xl);
  background: var(--color-surface-container-low);
}

.task-existing-documents {
  margin-top: var(--space-3);
}

.task-existing-documents__meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.task-existing-documents__name {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
}

.task-existing-documents__name span {
  overflow: clip;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-existing-documents__details {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

.task-existing-documents__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.task-existing-documents__empty {
  margin-top: var(--space-2);
  color: var(--color-on-surface-variant);
}
</style>
