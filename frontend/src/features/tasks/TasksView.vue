<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import CAlert from '@/components/atoms/CAlert.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CDocumentLinkActionModal from '@/components/molecules/CDocumentLinkActionModal.vue'
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CLinkedDocumentsModal from '@/components/molecules/CLinkedDocumentsModal.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { buildingApi, contractorsApi, documentsApi, tasksApi, usersApi } from '@/api'
import type { Contractor, Document, Task, TaskStatus, TaskPriority, User } from '@/types/domain'
import { extractFileExtension } from '@/utils/files'

const { t } = useI18n()
const { formatDate, formatFileSize, formatRelative } = useFormatters()

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const allTasks = ref<Task[]>([])
const allContractors = ref<Contractor[]>([])
const allUsers = ref<User[]>([])
const statusFilter = ref<'all' | TaskStatus>('all')
const TASK_MODAL_ERROR_CLOSE_DELAY_MS = 2200
const VENDOR_LINK_COPY_FEEDBACK_DELAY_MS = 1800

// ── Task form modal ───────────────────────────────────────────────
const showTaskModal = ref(false)
const taskFeedback = useFormFeedback()
const savingTask = ref(false)
const editingTask = ref<Task | null>(null)
const { schedule: scheduleTaskClose, forceExpire: forceTaskClose } = useAutoClose()
watch(showTaskModal, (open) => { if (!open) forceTaskClose() })

// Assignment mode within the modal for open tasks
// 'none' = not assigned | 'self' = self-assign (in_progress) | 'contractor' = assign contractor
const assignMode = ref<'none' | 'self' | 'contractor'>('none')

// Contractor autocomplete state
const contractorSearch = ref('')
const contractorSuggestions = ref<Contractor[]>([])
const showSuggestions = ref(false)
const selectedContractor = ref<Contractor | null>(null)
const createSuggestedContractor = ref(false)
function createDefaultContractorDraft() {
  return {
    name: '',
    email: '',
    phone: '',
  }
}
const contractorDraft = ref(createDefaultContractorDraft())

function emptyStringToUndefined(value: string) {
  return value.trim() || undefined
}

function contractorDomainLabel(c: Contractor): string {
  if (!c.domains.length) return ''
  return c.domains.map(d => t(`contractors.domains.${d}`, d)).join(', ')
}

// Max suggestions shown when typing (narrows the list) vs on focus (shows all)
const MAX_SUGGESTIONS = 10

function populateSuggestions(query: string) {
  const q = query.trim().toLowerCase()
  if (!q) {
    contractorSuggestions.value = allContractors.value.slice(0, MAX_SUGGESTIONS)
  } else {
    contractorSuggestions.value = allContractors.value
      .filter(c => c.name.toLowerCase().includes(q))
      .slice(0, MAX_SUGGESTIONS)
  }
  showSuggestions.value = contractorSuggestions.value.length > 0
}

// Offer inline contractor creation only when the user explicitly chose contractor
// assignment, typed something, and no known contractor currently matches.
const canSuggestContractorCreation = computed(() => (
  assignMode.value === 'contractor'
  && !selectedContractor.value
  && contractorSearch.value.trim().length > 0
  && contractorSuggestions.value.length === 0
))

function onContractorInput() {
  // Any manual text edit deselects the current contractor
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
  // Hide immediately — mousedown.prevent on the item ensures this fires before blur
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
  // Small delay so mousedown on a suggestion item fires before the list is hidden.
  // mousedown.prevent on each item prevents default focus steal, so the blur fires
  // after the click is processed by selectContractorSuggestion.
  window.setTimeout(() => { showSuggestions.value = false }, 100)
}

function clearContractor() {
  selectedContractor.value = null
  contractorSearch.value = ''
  createSuggestedContractor.value = false
  contractorDraft.value = createDefaultContractorDraft()
  showSuggestions.value = false
}

function contractorDisplayName(task: Task): string | undefined {
  return contractorsById.value.get(task.contractor_id ?? '')?.name ?? task.contractor_name
}

async function resolveContractorSelection() {
  if (assignMode.value !== 'contractor') {
    return {
      contractorId: null,
      contractorName: null,
    }
  }
  if (selectedContractor.value) {
    return {
      contractorId: selectedContractor.value.id,
      contractorName: selectedContractor.value.name,
    }
  }
  if (createSuggestedContractor.value && contractorDraft.value.name) {
    const createdContractor = await contractorsApi.create({
      name: contractorDraft.value.name,
      email: emptyStringToUndefined(contractorDraft.value.email),
      phone: emptyStringToUndefined(contractorDraft.value.phone),
    })
    allContractors.value = [createdContractor.data, ...allContractors.value]
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

// Auto-clear contractor when switching away from contractor mode
watch(assignMode, (newMode) => {
  if (newMode !== 'contractor') {
    clearContractor()
  }
})

// Map for looking up contractor name by id on task cards
const contractorsById = computed(() =>
  new Map(allContractors.value.map(c => [c.id, c])),
)

const usersById = computed(() =>
  new Map(allUsers.value.map(user => [user.id, user])),
)

const defaultForm = () => ({
  title: '',
  description: '',
  priority: 'normal' as string,
})
const taskForm = ref(defaultForm())
const selectedTaskDocuments = ref<File[]>([])
const taskDocumentInputRef = ref<HTMLInputElement | null>(null)
const taskDocumentsDragOver = ref(false)
const loadingTaskFormDocuments = ref(false)
const uploadingTaskDocuments = ref(false)
const existingTaskDocuments = ref<Document[]>([])
const pendingTaskDocument = ref<Document | null>(null)
const taskDocumentActionFeedback = useFormFeedback()
const taskDocumentActionLoading = ref(false)

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [usersRes, buildingRes, tasksRes, contractorsRes] = await Promise.all([
    usersApi.list(),
    buildingApi.get(),
    tasksApi.list(),
    contractorsApi.list(),
  ])
  allUsers.value = usersRes.data
  buildingName.value = buildingRes.data.name
  allTasks.value = tasksRes.data
  allContractors.value = contractorsRes.data
}

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
onMounted(reload)

// ── Derived ───────────────────────────────────────────────────────
const tasks = computed(() =>
  (statusFilter.value === 'all'
    ? allTasks.value
    : allTasks.value.filter(t => t.status === statusFilter.value))
    .slice()
    .sort((left, right) => right.updated_at.localeCompare(left.updated_at)),
)

const counts = computed(() => ({
  open: allTasks.value.filter(t => t.status === 'open').length,
  in_progress: allTasks.value.filter(t => t.status === 'in_progress').length,
  awaiting_vendor: allTasks.value.filter(t => t.status === 'awaiting_vendor').length,
  awaiting_validation: allTasks.value.filter(t => t.status === 'awaiting_validation').length,
  closed: allTasks.value.filter(t => t.status === 'closed').length,
}))

const activeCount = computed(() =>
  allTasks.value.filter(t => t.status !== 'closed' && t.status !== 'cancelled').length,
)

// ── Helpers ───────────────────────────────────────────────────────
/** Returns true for any terminal task status (closed or cancelled). */
function isTerminal(status: string): boolean {
  return status === 'closed' || status === 'cancelled'
}

function statusVariant(s: string): 'valid' | 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'open') return 'valid'
  if (s === 'in_progress') return 'open'
  if (s === 'awaiting_vendor') return 'default'
  if (s === 'awaiting_validation') return 'pending'
  if (s === 'closed') return 'closed'
  return 'default'
}

function closeActionLabel(task: Task): string {
  return task.status === 'awaiting_validation'
    ? t('tasks.validateTask')
    : t('tasks.closeTask')
}

function priorityVariant(p: string): 'err' | 'pending' | 'primary' | 'valid' | 'default' {
  if (p === 'urgent') return 'err'
  if (p === 'high') return 'pending'
  if (p === 'normal') return 'primary'
  if (p === 'low') return 'valid'
  return 'default'
}

function assignedUserName(task: Task): string | null {
  const assignedUser = task.assigned_to ? usersById.value.get(task.assigned_to) : undefined
  if (!assignedUser) return null
  return `${assignedUser.first_name} ${assignedUser.last_name}`
}

function hasAssignment(task: Task): boolean {
  return Boolean(contractorDisplayName(task) || assignedUserName(task))
}

function openTaskDocumentPicker() {
  taskDocumentInputRef.value?.click()
}

function appendTaskDocuments(files: File[]) {
  if (!files.length) return
  selectedTaskDocuments.value = [...selectedTaskDocuments.value, ...files]
}

function onTaskDocumentChange(event: Event) {
  const input = event.target as HTMLInputElement
  appendTaskDocuments(Array.from(input.files ?? []))
  if (taskDocumentInputRef.value) taskDocumentInputRef.value.value = ''
}

function onTaskDocumentDrop(event: DragEvent) {
  taskDocumentsDragOver.value = false
  appendTaskDocuments(Array.from(event.dataTransfer?.files ?? []))
}

function removeSelectedTaskDocument(index: number) {
  selectedTaskDocuments.value = selectedTaskDocuments.value.filter((_, currentIndex) => currentIndex !== index)
}

async function uploadTaskDocument(taskId: string, file: File) {
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

async function attachSelectedTaskDocuments(taskId: string) {
  const failedDocuments: string[] = []

  for (const document of selectedTaskDocuments.value) {
    try {
      await uploadTaskDocument(taskId, document)
    } catch {
      failedDocuments.push(document.name)
    }
  }

  return failedDocuments
}

async function loadTaskFormDocuments(taskId: string) {
  loadingTaskFormDocuments.value = true
  taskDocumentActionFeedback.reset()

  try {
    const response = await tasksApi.listDocuments(taskId)
    existingTaskDocuments.value = response.data
  } catch (err) {
    taskFeedback.setError(toErrorMessage(err, t('tasks.documentsLoadError')))
  } finally {
    loadingTaskFormDocuments.value = false
  }
}

function openTaskDocumentActionModal(document: Document) {
  pendingTaskDocument.value = document
  taskDocumentActionFeedback.reset()
}

function closeTaskDocumentActionModal() {
  if (taskDocumentActionLoading.value) return
  pendingTaskDocument.value = null
  taskDocumentActionFeedback.reset()
}

function updateTaskDocumentCount(taskId: string, delta: number) {
  const task = allTasks.value.find(currentTask => currentTask.id === taskId)
  if (task) {
    task.document_count = Math.max(0, task.document_count + delta)
  }
}

async function handleTaskDocumentAction(deleteDocument: boolean) {
  if (!editingTask.value || !pendingTaskDocument.value) return

  taskDocumentActionLoading.value = true
  taskDocumentActionFeedback.reset()

  try {
    if (deleteDocument) {
      await documentsApi.delete(pendingTaskDocument.value.id)
    } else {
      await tasksApi.detachDocument(editingTask.value.id, pendingTaskDocument.value.id)
    }

    existingTaskDocuments.value = existingTaskDocuments.value.filter(document => document.id !== pendingTaskDocument.value?.id)
    if (taskDocumentsTarget.value?.id === editingTask.value.id) {
      taskDocuments.value = taskDocuments.value.filter(document => document.id !== pendingTaskDocument.value?.id)
    }
    updateTaskDocumentCount(editingTask.value.id, -1)
    pendingTaskDocument.value = null
  } catch (err) {
    taskDocumentActionFeedback.setError(
      toErrorMessage(
        err,
        deleteDocument ? t('tasks.documentsDeleteError') : t('tasks.documentsDetachError'),
      ),
    )
  } finally {
    taskDocumentActionLoading.value = false
  }
}

async function openTaskDocuments(task: Task) {
  taskDocumentsTarget.value = task
  taskDocuments.value = []
  loadingTaskDocuments.value = true
  try {
    const response = await tasksApi.listDocuments(task.id)
    taskDocuments.value = response.data
  } catch (err) {
    pageError.value = toErrorMessage(err, t('common.error'))
    taskDocumentsTarget.value = null
  } finally {
    loadingTaskDocuments.value = false
  }
}

async function downloadTaskDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch (err) {
    pageError.value = toErrorMessage(err, t('common.error'))
  }
}

// ── Task modal actions ────────────────────────────────────────────
function openCreateModal() {
  editingTask.value = null
  taskForm.value = defaultForm()
  selectedTaskDocuments.value = []
  taskDocumentsDragOver.value = false
  existingTaskDocuments.value = []
  pendingTaskDocument.value = null
  selectedContractor.value = null
  contractorSearch.value = ''
  createSuggestedContractor.value = false
  contractorDraft.value = createDefaultContractorDraft()
  assignMode.value = 'none'
  taskFeedback.reset()
  showTaskModal.value = true
}

async function openEditModal(task: Task) {
  editingTask.value = task
  taskForm.value = {
    title: task.title,
    description: task.description ?? '',
    priority: task.priority,
  }
  selectedTaskDocuments.value = []
  taskDocumentsDragOver.value = false
  existingTaskDocuments.value = []
  pendingTaskDocument.value = null
  // Pre-fill contractor from loaded referential
  if (task.contractor_id) {
    const c = contractorsById.value.get(task.contractor_id)
    selectedContractor.value = c ?? null
    contractorSearch.value = c?.name ?? task.contractor_name ?? ''
    assignMode.value = 'contractor'
    createSuggestedContractor.value = false
    contractorDraft.value = createDefaultContractorDraft()
  } else if (task.contractor_name) {
    selectedContractor.value = null
    contractorSearch.value = task.contractor_name
    createSuggestedContractor.value = false
    contractorDraft.value = createDefaultContractorDraft()
    assignMode.value = 'contractor'
  } else if (task.status === 'in_progress') {
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
  taskFeedback.reset()
  showTaskModal.value = true
  await loadTaskFormDocuments(task.id)
}

async function saveTask() {
  if (taskFeedback.status.value === 'success') { forceTaskClose(); return }
  if (!taskForm.value.title.trim()) return
  savingTask.value = true
  taskFeedback.reset()
  try {
    const { contractorId, contractorName } = await resolveContractorSelection()
    let taskId: string

    if (editingTask.value) {
      // Reflect assignment mode change in DB:
      // 'self'       → in_progress (self-assigned, no contractor)
      // 'none'       → open (unassigned)
      // 'contractor' → keep current status (contractor_id drives the assignment)
      let statusPatch: string | undefined
      if (assignMode.value === 'self') {
        statusPatch = 'in_progress'
      } else if (assignMode.value === 'none') {
        statusPatch = 'open'
      }
      await tasksApi.update(editingTask.value.id, {
        title: taskForm.value.title,
        description: taskForm.value.description || null,
        priority: taskForm.value.priority,
        contractor_id: assignMode.value === 'contractor' ? contractorId : null,
        contractor_name: assignMode.value === 'contractor' ? contractorName : null,
        ...(statusPatch ? { status: statusPatch } : {}),
      })
      taskId = editingTask.value.id
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

    if (selectedTaskDocuments.value.length > 0) {
      uploadingTaskDocuments.value = true
      const failedDocuments = await attachSelectedTaskDocuments(taskId)
      uploadingTaskDocuments.value = false

      if (failedDocuments.length > 0) {
        taskFeedback.setError(`${t('tasks.documentUploadError')}: ${failedDocuments.join(', ')}`)
        if (editingTask.value) {
          await loadTaskFormDocuments(taskId)
          await reload()
        } else {
          window.setTimeout(() => {
            showTaskModal.value = false
            reload()
          }, TASK_MODAL_ERROR_CLOSE_DELAY_MS)
        }
        return
      }
    }

    taskFeedback.setSuccess(t('common.saved'))
    scheduleTaskClose(() => { showTaskModal.value = false; reload() })
  } catch (err) {
    taskFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    savingTask.value = false
    uploadingTaskDocuments.value = false
  }
}

// ── Close task from card (for any non-closed task) ────────────────
const closingTask = ref<Task | null>(null)
const closeFeedback = useFormFeedback()
const closingInProgress = ref(false)
const closeAmount = ref('')
const { schedule: scheduleCloseTask, forceExpire: forceCloseTask } = useAutoClose()
watch(closingTask, (task) => { if (!task) forceCloseTask() })

function openCloseModal(task: Task) {
  closingTask.value = task
  closeAmount.value = task.completion_cost != null ? String(task.completion_cost) : ''
  closeFeedback.reset()
}

async function confirmCloseTask() {
  if (closeFeedback.status.value === 'success') { forceCloseTask(); return }
  if (!closingTask.value) return
  closingInProgress.value = true
  closeFeedback.reset()
  try {
    await tasksApi.update(closingTask.value.id, {
      status: 'closed',
      completion_cost: closeAmount.value ? Number(closeAmount.value) : null,
    })
    closeFeedback.setSuccess(t('tasks.closeSuccess'))
    scheduleCloseTask(() => { closingTask.value = null; reload() })
  } catch (err) {
    closeFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    closingInProgress.value = false
  }
}

// ── Delete task modal ─────────────────────────────────────────────
const showDeleteModal = ref(false)
const taskToDelete = ref<Task | null>(null)
const deletingTask = ref(false)
const deleteTaskFeedback = useFormFeedback()
const pageError = ref('')
const taskDocumentsTarget = ref<Task | null>(null)
const taskDocuments = ref<Document[]>([])
const loadingTaskDocuments = ref(false)
const { schedule: scheduleDeleteTask, forceExpire: forceDeleteTask } = useAutoClose()
watch(showDeleteModal, (open) => { if (!open) forceDeleteTask() })

function openDeleteModal(task: Task) {
  // Close the edit modal if it is open to avoid modal stacking
  showTaskModal.value = false
  taskToDelete.value = task
  deleteTaskFeedback.reset()
  showDeleteModal.value = true
}

async function confirmDeleteTask() {
  if (deleteTaskFeedback.status.value === 'success') { forceDeleteTask(); return }
  if (!taskToDelete.value) return
  deletingTask.value = true
  deleteTaskFeedback.reset()
  try {
    await tasksApi.delete(taskToDelete.value.id)
    deleteTaskFeedback.setSuccess(t('tasks.deleteSuccess'))
    scheduleDeleteTask(() => { showDeleteModal.value = false; reload() })
  } catch (err) {
    deleteTaskFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    deletingTask.value = false
  }
}

// ── Vendor token actions ───────────────────────────────────────────
const generatingToken = ref<string | null>(null)
// vendorLinks: task.id → { url, expires_at } — populated on demand
const vendorLinks = ref<Record<string, { url: string; expires_at: string }>>({})

// ── Revoke token modal ─────────────────────────────────────────────
const revokingTask = ref<Task | null>(null)
const revokeFeedback = useFormFeedback()
const revoking = ref(false)
const { schedule: scheduleRevokeTask, forceExpire: forceRevokeTask } = useAutoClose()
watch(revokingTask, (task) => { if (!task) forceRevokeTask() })

function openRevokeModal(task: Task) {
  revokingTask.value = task
  revokeFeedback.reset()
}

async function confirmRevoke() {
  if (revokeFeedback.status.value === 'success') { forceRevokeTask(); return }
  if (!revokingTask.value) return
  revoking.value = true
  revokeFeedback.reset()
  try {
    await tasksApi.revokeToken(revokingTask.value.id)
    delete vendorLinks.value[revokingTask.value.id]
    revokeFeedback.setSuccess(t('tasks.revokeSuccess'))
    scheduleRevokeTask(() => { revokingTask.value = null; reload() })
  } catch (err) {
    revokeFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    revoking.value = false
  }
}

// ── Reopen task modal ─────────────────────────────────────────────
const reopeningTask = ref<Task | null>(null)
const reopenFeedback = useFormFeedback()
const reopening = ref(false)
const { schedule: scheduleReopenTask, forceExpire: forceReopenTask } = useAutoClose()
watch(reopeningTask, (task) => { if (!task) forceReopenTask() })

function openReopenModal(task: Task) {
  reopeningTask.value = task
  reopenFeedback.reset()
}

async function confirmReopen() {
  if (reopenFeedback.status.value === 'success') { forceReopenTask(); return }
  if (!reopeningTask.value) return
  reopening.value = true
  reopenFeedback.reset()
  try {
    await tasksApi.update(reopeningTask.value.id, { status: 'open' })
    reopenFeedback.setSuccess(t('tasks.reopenSuccess'))
    scheduleReopenTask(() => { reopeningTask.value = null; reload() })
  } catch (err) {
    reopenFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    reopening.value = false
  }
}

async function generateToken(task: Task) {
  generatingToken.value = task.id
  pageError.value = ''
  try {
    const res = await tasksApi.generateToken(task.id)
    // Rebuild with window.location.origin: the backend's public_url may differ from the
    // actual frontend origin in dev/staging/multi-tenant deployments. The token UUID is
    // what matters — the vendor route (/vendor/:token) is always on the frontend SPA.
    const tokenStr = res.data.token
    const url = `${window.location.origin}/vendor/${tokenStr}`
    vendorLinks.value[task.id] = { url, expires_at: res.data.expires_at }
    await reload()
  } catch (err) {
    pageError.value = toErrorMessage(err, t('common.error'))
  } finally {
    generatingToken.value = null
  }
}

async function showVendorLink(task: Task) {
  if (vendorLinks.value[task.id]) return // already loaded
  pageError.value = ''
  try {
    const res = await tasksApi.getVendorTokenUrl(task.id)
    // Parse the token from the backend URL using URL API for robustness,
    // then rebuild with window.location.origin (see generateToken for rationale).
    const parsedPath = new URL(res.data.url).pathname
    const tokenSegment = parsedPath.split('/vendor/')[1]
    const url = tokenSegment
      ? `${window.location.origin}/vendor/${tokenSegment}`
      : res.data.url
    vendorLinks.value[task.id] = { url, expires_at: res.data.expires_at }
  } catch (err) {
    pageError.value = toErrorMessage(err, t('common.error'))
  }
}

const copiedVendorLinkTaskId = ref<string | null>(null)

async function copyVendorLink(taskId: string) {
  const vendorLink = vendorLinks.value[taskId]
  if (!vendorLink) return

  try {
    await navigator.clipboard.writeText(vendorLink.url)
    copiedVendorLinkTaskId.value = taskId
    window.setTimeout(() => {
      if (copiedVendorLinkTaskId.value === taskId) {
        copiedVendorLinkTaskId.value = null
      }
    }, VENDOR_LINK_COPY_FEEDBACK_DELAY_MS)
  } catch (err) {
    pageError.value = toErrorMessage(err, t('common.error'))
  }
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
          <span>{{ t('tasks.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          @click="openCreateModal"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('tasks.newTask') }}</span>
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
              {{ t('tasks.title') }}
            </h1>
            <p
              class="body-md"
              style="color:var(--color-on-surface-variant)"
            >
              {{ t('tasks.activeSummary', { count: activeCount }) }}
            </p>
          </div>
        </div>

        <CAlert
          v-if="pageError"
          variant="error"
        >
          {{ pageError }}
        </CAlert>

        <!-- Filtres -->
        <div class="tabs">
          <button
            v-for="item in [
              { key: 'all', label: t('tasks.filters.all'), count: allTasks.length },
              { key: 'open', label: t('tasks.filters.open'), count: counts.open },
              { key: 'in_progress', label: t('tasks.filters.in_progress'), count: counts.in_progress },
              { key: 'awaiting_vendor', label: t('tasks.filters.awaiting_vendor'), count: counts.awaiting_vendor },
              { key: 'awaiting_validation', label: t('tasks.filters.awaiting_validation'), count: counts.awaiting_validation },
              { key: 'closed', label: t('tasks.filters.closed'), count: counts.closed },
            ]"
            :key="item.key"
            :class="['tab', { 'tab--active': statusFilter === item.key }]"
            @click="statusFilter = item.key as typeof statusFilter"
          >
            {{ item.label }}
            <span class="tab-count">{{ item.count }}</span>
          </button>
        </div>

        <!-- Liste -->
        <div class="task-list">
          <CCard
            v-for="task in tasks"
            :key="task.id"
            :shadow="!isTerminal(task.status)"
            :class="['activity-card', 'task-card', { 'card--terminated': isTerminal(task.status) }]"
          >
            <div class="activity-card__head task-card__head">
              <div class="task-card__header-row">
                <RouterLink
                  :to="{ name: 'task-detail', params: { id: task.id } }"
                  class="activity-card__title task-title-link"
                >
                  {{ task.title }}
                </RouterLink>
                <div class="task-card__header-badges">
                  <CBadge :variant="isTerminal(task.status) ? 'disabled' : priorityVariant(task.priority)">
                    {{ t(`tasks.priorities.${task.priority}`) }}
                  </CBadge>
                  <CBadge
                    :variant="isTerminal(task.status) ? 'disabled' : statusVariant(task.status)"
                    dot
                  >
                    {{ t(`tasks.statuses.${task.status}`) }}
                  </CBadge>
                </div>
              </div>

              <div class="activity-card__stack">
                <p
                  v-if="task.description"
                  class="activity-card__description"
                >
                  {{ task.description }}
                </p>

                <div class="activity-card__documents">
                  <button
                    v-if="task.document_count > 0"
                    class="activity-card__line task-card__documents-button"
                    @click="openTaskDocuments(task)"
                  >
                    <CIconFile />
                    {{ t('tasks.documentsCount', task.document_count) }}
                  </button>
                  <span
                    v-else
                    class="activity-card__line"
                  >
                    <CIconFile />
                    {{ t('tasks.documentsCount', task.document_count) }}
                  </span>
                </div>

                <div class="activity-card__meta">
                  <span
                    v-if="hasAssignment(task)"
                    class="activity-card__line"
                  >
                    <CIconHardHat v-if="contractorDisplayName(task)" />
                    <CIconUser v-else />
                    <RouterLink
                      v-if="task.contractor_id && contractorDisplayName(task)"
                      :to="{ name: 'contractor-detail', params: { id: task.contractor_id } }"
                      class="task-card__meta-link"
                    >
                      {{ contractorDisplayName(task) }}
                    </RouterLink>
                    <span v-else>
                      {{ contractorDisplayName(task) ?? assignedUserName(task) }}
                    </span>
                  </span>
                  <span class="activity-card__line">
                    <CIconCalendar />
                    {{ t('tasks.lifecycle.createdRelative', { relative: formatRelative(task.created_at) }) }}
                  </span>
                </div>

                <div
                  v-if="(contractorDisplayName(task) && !task.has_vendor_token && !isTerminal(task.status)) || task.has_vendor_token"
                  class="activity-card__secondary-actions"
                >
                  <CBtn
                    v-if="contractorDisplayName(task) && !task.has_vendor_token && !isTerminal(task.status)"
                    variant="ghost"
                    size="sm"
                    :disabled="generatingToken === task.id"
                    :aria-label="t('tasks.generateToken')"
                    @click="generateToken(task)"
                  >
                    <CIconLink />
                    {{ t('tasks.generateToken') }}
                  </CBtn>
                  <CBtn
                    v-if="task.has_vendor_token"
                    variant="ghost"
                    size="sm"
                    :aria-label="t('tasks.showLink')"
                    @click="showVendorLink(task)"
                  >
                    <CIconEye />
                    {{ t('tasks.showLink') }}
                  </CBtn>
                  <CBtn
                    v-if="task.has_vendor_token"
                    variant="ghost"
                    size="sm"
                    :aria-label="t('tasks.revokeToken')"
                    @click="openRevokeModal(task)"
                  >
                    <CIconX />
                    {{ t('tasks.revokeToken') }}
                  </CBtn>
                </div>

                <div
                  v-if="vendorLinks[task.id]"
                  class="task-card__vendor-link"
                >
                  <div class="task-card__vendor-link-meta">
                    <span class="body-xs">
                      {{ t('tasks.vendorTokenActive', { date: formatDate(vendorLinks[task.id].expires_at) }) }}
                    </span>
                    <a
                      :href="vendorLinks[task.id].url"
                      class="task-card__vendor-link-url"
                      target="_blank"
                      rel="noopener"
                    >
                      {{ vendorLinks[task.id].url }}
                    </a>
                  </div>
                  <CBtn
                    variant="ghost"
                    size="sm"
                    @click="copyVendorLink(task.id)"
                  >
                    <CIconCopy />
                    {{ copiedVendorLinkTaskId === task.id ? t('tasks.vendorTokenCopied') : t('common.copy') }}
                  </CBtn>
                </div>

                <div class="activity-card__separator"></div>

                <div class="activity-card__actions">
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :aria-label="t('tasks.editTask')"
                    @click="openEditModal(task)"
                  >
                    <CIconEdit />
                    {{ t('common.edit') }}
                  </CBtn>
                  <CBtn
                    v-if="task.status === 'closed'"
                    variant="primary"
                    size="sm"
                    :aria-label="t('tasks.reopenTask')"
                    @click="openReopenModal(task)"
                  >
                    <CIconRefresh />
                    {{ t('tasks.reopenTask') }}
                  </CBtn>
                  <CBtn
                    v-else-if="!isTerminal(task.status)"
                    variant="primary"
                    size="sm"
                    :aria-label="closeActionLabel(task)"
                    @click="openCloseModal(task)"
                  >
                    <CIconCheckCircle />
                    {{ closeActionLabel(task) }}
                  </CBtn>
                </div>
              </div>
            </div>
          </CCard>

          <div
            v-if="!tasks.length"
            class="empty-state"
          >
            {{ t('tasks.empty') }}
          </div>
        </div>
      </template>
    </div>

    <!-- Create / Edit task modal -->
    <CModal
      :open="showTaskModal"
      :title="editingTask ? t('tasks.editTask') : t('tasks.newTask')"
      size="md"
      :closable="!savingTask"
      @update:open="showTaskModal = $event"
    >
      <form
        class="form-grid"
        @submit.prevent="saveTask"
      >
        <div class="form-field">
          <label class="form-label">{{ t('tasks.titleLabel') }} *</label>
          <input
            v-model="taskForm.title"
            type="text"
            class="form-input"
            :placeholder="t('tasks.titleLabel')"
            required
          />
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('tasks.descriptionLabel') }}</label>
          <textarea
            v-model="taskForm.description"
            class="form-input"
            rows="3"
          ></textarea>
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('tasks.priorityLabel') }}</label>
          <select
            v-model="taskForm.priority"
            class="form-input"
          >
            <option
              v-for="p in ['low', 'normal', 'high', 'urgent']"
              :key="p"
              :value="p"
            >
              {{ t(`tasks.priorities.${p}`) }}
            </option>
          </select>
        </div>
        <!-- Contractor assignment with autocomplete (only when assignMode === 'contractor') -->
        <div
          v-if="assignMode === 'contractor'"
          class="form-field"
        >
          <label class="form-label">{{ t('tasks.contractorLabel') }}</label>
          <div style="position:relative">
            <input
              v-model="contractorSearch"
              type="text"
              class="form-input"
              :placeholder="t('tasks.contractorPlaceholder')"
              autocomplete="off"
              @focus="onContractorFocus"
              @input="onContractorInput"
              @blur="hideSuggestions"
            />
            <button
              v-if="selectedContractor"
              type="button"
              class="contractor-clear"
              :aria-label="t('common.clear')"
              @click="clearContractor"
            >
              ×
            </button>
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
          <span
            v-if="selectedContractor"
            class="field-hint"
          >
            {{ selectedContractor.email || selectedContractor.phone || '' }}
          </span>
          <span
            v-else-if="contractorSearch.trim()"
            class="field-hint"
          >
            {{ t('tasks.contractorFreeInputHint') }}
          </span>
        </div>
        <div
          v-if="assignMode === 'contractor' && createSuggestedContractor"
          class="form-field contractor-create-box"
        >
          <label class="form-label">{{ t('tasks.contractorCreateLabel') }}</label>
          <div class="form-grid-2col contractor-create-grid">
            <div class="form-field">
              <label class="form-label">{{ t('contractors.email') }}</label>
              <input
                v-model="contractorDraft.email"
                type="email"
                class="form-input"
                :placeholder="t('contractors.emailPlaceholder')"
              />
            </div>
            <div class="form-field">
              <label class="form-label">{{ t('contractors.phone') }}</label>
              <input
                v-model="contractorDraft.phone"
                type="tel"
                class="form-input"
                :placeholder="t('contractors.phonePlaceholder')"
              />
            </div>
          </div>
          <span class="field-hint">
            {{ t('tasks.contractorCreateHelp') }}
          </span>
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('tasks.documentsLabel') }}</label>
          <input
            ref="taskDocumentInputRef"
            type="file"
            multiple
            style="display:none"
            @change="onTaskDocumentChange"
          />
          <div
            :class="['task-dropzone', { 'task-dropzone--over': taskDocumentsDragOver }]"
            role="button"
            tabindex="0"
            @click="openTaskDocumentPicker"
            @keydown.enter.prevent="openTaskDocumentPicker"
            @keydown.space.prevent="openTaskDocumentPicker"
            @dragenter.prevent="taskDocumentsDragOver = true"
            @dragover.prevent="taskDocumentsDragOver = true"
            @dragleave.prevent="taskDocumentsDragOver = false"
            @drop.prevent="onTaskDocumentDrop"
          >
            <CIconUpload
              width="20"
              height="20"
              style="color:var(--color-on-surface-variant)"
            />
            <span class="body-xs task-dropzone__hint">
              {{ editingTask ? t('tasks.editDocumentHint') : t('tasks.documentHint') }}
            </span>
          </div>

          <div
            v-if="selectedTaskDocuments.length > 0"
            class="task-selected-documents"
          >
            <div
              v-for="(document, index) in selectedTaskDocuments"
              :key="`${document.name}-${index}`"
              class="task-selected-document"
            >
              <span class="task-selected-document__name">{{ document.name }}</span>
              <button
                type="button"
                class="task-selected-document__remove"
                :title="t('tasks.removeDocument')"
                :aria-label="t('tasks.removeDocument')"
                @click="removeSelectedTaskDocument(index)"
              >
                ×
              </button>
            </div>
          </div>

          <div
            v-if="editingTask"
            class="task-existing-documents"
          >
            <p class="field-hint task-existing-documents__hint">
              {{ t('tasks.attachedDocuments') }}
            </p>
            <CPageLoader v-if="loadingTaskFormDocuments" />
            <div
              v-else-if="existingTaskDocuments.length === 0"
              class="task-existing-documents__empty"
            >
              {{ t('tasks.noAttachedDocuments') }}
            </div>
            <div
              v-else
              class="task-existing-documents__list"
            >
              <div
                v-for="document in existingTaskDocuments"
                :key="document.id"
                class="task-existing-documents__row"
              >
                <div class="task-existing-documents__meta">
                  <div class="task-existing-documents__name">
                    <CIconFile
                      width="16"
                      height="16"
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
                    @click="downloadTaskDocument(document.id)"
                  >
                    <CIconDownload />
                    {{ t('common.download') }}
                  </CBtn>
                  <CBtn
                    variant="danger"
                    size="sm"
                    :loading="taskDocumentActionLoading && pendingTaskDocument?.id === document.id"
                    @click="openTaskDocumentActionModal(document)"
                  >
                    <CIconTrash />
                    {{ t('tasks.detachDocument') }}
                  </CBtn>
                </div>
              </div>
            </div>
          </div>
        </div>
        <!-- Assignment mode: radio (visible for new tasks, open tasks, and in_progress tasks) -->
        <div
          v-if="!editingTask || ['open', 'in_progress', 'awaiting_vendor', 'awaiting_validation'].includes(editingTask.status)"
          class="form-field"
        >
          <label class="form-label">{{ t('tasks.assignmentSection') }}</label>
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
      </form>
      <template #footer>
        <!-- Delete: visible from the modal only -->
        <CBtn
          v-if="editingTask"
          variant="danger"
          :loading="savingTask"
          style="margin-right:auto"
          @click="openDeleteModal(editingTask)"
        >
          <CIconTrash />
          {{ t('common.delete') }}
        </CBtn>
        <CFormFeedback
          :status="taskFeedback.status.value"
          :message="taskFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showTaskModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="savingTask || uploadingTaskDocuments"
          :closing="taskFeedback.status.value === 'success'"
          @click="saveTask"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Revoke token modal -->
    <CModal
      :open="!!revokingTask"
      :title="t('tasks.confirmRevokeTitle')"
      size="sm"
      @update:open="v => !v && (revokingTask = null)"
    >
      <p class="body-sm">
        {{ t('tasks.confirmRevokeToken') }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="revokeFeedback.status.value"
          :message="revokeFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="revokingTask = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="revoking"
          :closing="revokeFeedback.status.value === 'success'"
          @click="confirmRevoke"
        >
          {{ t('tasks.revokeToken') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Close task modal -->
    <CModal
      :open="!!closingTask"
      :title="closingTask?.status === 'awaiting_validation' ? t('tasks.confirmCloseValidationTitle') : t('tasks.confirmCloseTitle')"
      size="sm"
      @update:open="v => !v && (closingTask = null)"
    >
      <p class="body-sm">
        {{
          closingTask?.status === 'awaiting_validation'
            ? t('tasks.confirmCloseValidationMessage', { title: closingTask?.title ?? '' })
            : t('tasks.confirmCloseMessage', { title: closingTask?.title ?? '' })
        }}
      </p>
      <div class="form-field task-close-field">
        <label class="form-label">{{ t('tasks.completionCostLabel') }}</label>
        <input
          v-model="closeAmount"
          type="number"
          min="0"
          step="0.01"
          class="form-input"
        />
        <span class="field-hint">
          {{
            closingTask?.status === 'awaiting_validation' && closingTask?.completion_cost != null
              ? t('tasks.completionCostVendorHint')
              : t('tasks.completionCostHint')
          }}
        </span>
      </div>
      <template #footer>
        <CFormFeedback
          class="modal-footer-feedback"
          :status="closeFeedback.status.value"
          :message="closeFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="closingTask = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="closingInProgress"
          :closing="closeFeedback.status.value === 'success'"
          @click="confirmCloseTask"
        >
          {{ closingTask ? closeActionLabel(closingTask) : t('tasks.closeTask') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Reopen task modal -->
    <CModal
      :open="!!reopeningTask"
      :title="t('tasks.confirmReopenTitle')"
      size="sm"
      @update:open="v => !v && (reopeningTask = null)"
    >
      <p class="body-sm">
        {{ t('tasks.confirmReopenMessage', { title: reopeningTask?.title ?? '' }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="reopenFeedback.status.value"
          :message="reopenFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="reopeningTask = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="reopening"
          :closing="reopenFeedback.status.value === 'success'"
          @click="confirmReopen"
        >
          {{ t('tasks.reopenTask') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Delete task modal -->
    <CModal
      :open="showDeleteModal"
      :title="t('tasks.confirmDeleteTitle')"
      size="sm"
      @update:open="showDeleteModal = $event"
    >
      <p class="body-sm">
        {{
          taskToDelete?.is_in_logbook
            ? t('tasks.confirmDeleteLogbookMessage', { title: taskToDelete?.title ?? '' })
            : t('tasks.confirmDeleteMessage', { title: taskToDelete?.title ?? '' })
        }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="deleteTaskFeedback.status.value"
          :message="deleteTaskFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showDeleteModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="deletingTask"
          :closing="deleteTaskFeedback.status.value === 'success'"
          @click="confirmDeleteTask"
        >
          {{ t('common.deleteLabel') }}
        </CBtn>
      </template>
    </CModal>

    <CLinkedDocumentsModal
      :open="!!taskDocumentsTarget"
      :title="taskDocumentsTarget?.title ?? ''"
      :documents="taskDocuments"
      :loading="loadingTaskDocuments"
      :empty-message="t('tasks.noLinkedDocuments')"
      @update:open="value => !value && (taskDocumentsTarget = null)"
      @download="downloadTaskDocument"
    />
    <CDocumentLinkActionModal
      :open="!!pendingTaskDocument"
      :title="t('tasks.documentsDetachTitle')"
      :description="t('tasks.documentsDetachMessage', { name: pendingTaskDocument?.original_name ?? '', title: editingTask?.title ?? '' })"
      :warning="t('tasks.documentsDeleteWarning', { name: pendingTaskDocument?.original_name ?? '' })"
      :loading="taskDocumentActionLoading"
      :status="taskDocumentActionFeedback.status.value"
      :message="taskDocumentActionFeedback.message.value"
      @update:open="value => !value && closeTaskDocumentActionModal()"
      @unlink="handleTaskDocumentAction(false)"
      @unlink-and-delete="handleTaskDocumentAction(true)"
    />
  </div>
</template>

<style scoped>
.task-list { display: flex; flex-direction: column; gap: 10px; }

.task-card { overflow: hidden; }
.task-card__head { gap: var(--space-3); }
.task-card__header-row {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
}
.task-card__header-badges {
  margin-left: auto;
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--space-2);
}
.task-title-link {
  color: inherit;
  text-decoration: none;
  font-weight: var(--fw-semibold);
}
.task-title-link:hover { text-decoration: underline; color: var(--color-primary); }
.task-card__meta-link {
  color: inherit;
  text-decoration: none;
}
.task-card__meta-link:hover { color: var(--color-primary); text-decoration: underline; }
.task-card__documents-button {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  min-width: unset;
  min-height: unset;
}
.task-card__documents-button:hover { color: var(--color-primary); }
.task-card__vendor-link {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
}
.task-card__vendor-link-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
.task-card__vendor-link-url {
  color: var(--color-primary);
  text-decoration: none;
  word-break: break-all;
}
.task-card__vendor-link-url:hover { text-decoration: underline; }

.assignment-options {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-2) 0;
}

.empty-state { padding: var(--space-8); text-align: center; font-size: var(--text-md); color: var(--color-on-surface-variant); }

/* Contractor autocomplete */
.autocomplete-list {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-md);
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  list-style: none;
  margin: 2px 0 0;
  padding: 4px 0;
  z-index: 100;
  max-height: 200px;
  overflow-y: auto;
}

.autocomplete-item {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-2);
  padding: 8px 12px;
  cursor: pointer;
  font-size: var(--text-sm);
}

.autocomplete-item:hover { background: var(--color-surface-container); }
.autocomplete-name { font-weight: var(--fw-medium); }
.autocomplete-meta { font-size: var(--text-xs); color: var(--color-on-surface-variant); }
/* Vertical layout reserved for the "create contractor" suggestion row. */
.autocomplete-item--action {
  align-items: flex-start;
  flex-direction: column;
}

.contractor-clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  cursor: pointer;
  font-size: 16px;
  color: var(--color-on-surface-variant);
  padding: 0 4px;
}

.field-hint { font-size: var(--text-xs); color: var(--color-on-surface-variant); }
.contractor-create-box {
  padding: var(--space-3);
  border: 1px dashed var(--color-outline);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-lowest);
}
.contractor-create-grid { margin-bottom: 0; }
.task-close-field { margin-top: var(--space-4); }
.modal-footer-feedback { margin-right: auto; min-width: 0; }
.task-dropzone {
  margin-top: var(--space-2);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  min-height: 108px;
  padding: var(--space-4);
  border: 1px dashed color-mix(in srgb, var(--color-outline) 72%, transparent);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-lowest);
  cursor: pointer;
  transition: border-color 0.2s ease, background 0.2s ease;
}
.task-dropzone:hover,
.task-dropzone--over {
  border-color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 6%, var(--color-surface-container-lowest));
}
.task-dropzone__hint { color: var(--color-on-surface-variant); }
.task-selected-documents,
.task-existing-documents__list {
  margin-top: var(--space-3);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.task-selected-document,
.task-existing-documents__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
}
.task-selected-document__name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.task-selected-document__remove {
  border: none;
  background: none;
  color: var(--color-on-surface-variant);
  cursor: pointer;
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
  overflow: hidden;
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
