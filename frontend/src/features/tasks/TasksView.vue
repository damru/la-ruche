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
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { buildingApi, contractorsApi, tasksApi, usersApi } from '@/api'
import type { Contractor, Task, TaskStatus, TaskPriority, User } from '@/types/domain'

const { t } = useI18n()
const { formatDate } = useFormatters()

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const allTasks = ref<Task[]>([])
const allContractors = ref<Contractor[]>([])
const allUsers = ref<User[]>([])
const statusFilter = ref<'all' | TaskStatus>('all')

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

function priorityTone(p: string): string {
  if (p === 'urgent') return 'tone--err'
  if (p === 'high') return 'tone--warn'
  if (p === 'normal') return 'tone--accent'
  return 'tone--valid'
}

function creatorName(userId?: string): string {
  const user = userId ? usersById.value.get(userId) : undefined
  return user ? `${user.first_name} ${user.last_name}` : t('common.noData')
}

// ── Task modal actions ────────────────────────────────────────────
function openCreateModal() {
  editingTask.value = null
  taskForm.value = defaultForm()
  selectedContractor.value = null
  contractorSearch.value = ''
  createSuggestedContractor.value = false
  contractorDraft.value = createDefaultContractorDraft()
  assignMode.value = 'none'
  taskFeedback.reset()
  showTaskModal.value = true
}

function openEditModal(task: Task) {
  editingTask.value = task
  taskForm.value = {
    title: task.title,
    description: task.description ?? '',
    priority: task.priority,
  }
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
}

async function saveTask() {
  if (taskFeedback.status.value === 'success') { forceTaskClose(); return }
  if (!taskForm.value.title.trim()) return
  savingTask.value = true
  taskFeedback.reset()
  try {
    const { contractorId, contractorName } = await resolveContractorSelection()

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
    } else {
      await tasksApi.create({
        title: taskForm.value.title,
        description: taskForm.value.description || undefined,
        priority: taskForm.value.priority as TaskPriority,
        contractor_id: assignMode.value === 'contractor' ? contractorId : null,
        contractor_name: assignMode.value === 'contractor' ? contractorName : null,
      })
    }
    taskFeedback.setSuccess(t('common.saved'))
    scheduleTaskClose(() => { showTaskModal.value = false; reload() })
  } catch (err) {
    taskFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    savingTask.value = false
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

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text).catch(() => {})
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
            <div class="activity-card__head">
              <div
                class="activity-card__icon"
                :class="isTerminal(task.status) ? 'tone--disabled' : priorityTone(task.priority)"
              >
                <CIconWrench />
              </div>
              <div class="activity-card__stack">
                <div class="activity-card__chips">
                  <CBadge
                    :variant="isTerminal(task.status) ? 'disabled' : statusVariant(task.status)"
                    dot
                  >
                    {{ t(`tasks.statuses.${task.status}`) }}
                  </CBadge>
                  <CBadge :variant="isTerminal(task.status) ? 'disabled' : priorityVariant(task.priority)">
                    {{ t(`tasks.priorities.${task.priority}`) }}
                  </CBadge>
                  <CBadge
                    v-if="task.has_vendor_token"
                    variant="default"
                  >
                    {{ t('tasks.vendorToken') }}
                  </CBadge>
                </div>

                <RouterLink
                  :to="{ name: 'task-detail', params: { id: task.id } }"
                  class="title-sm activity-card__title task-title-link"
                >
                  {{ task.title }}
                </RouterLink>

                <div class="activity-card__subtitle">
                  <span
                    v-if="task.contractor_id && contractorsById.get(task.contractor_id)"
                    class="activity-card__line"
                  >
                    <CIconLocation />
                    <RouterLink
                      :to="{ name: 'contractor-detail', params: { id: task.contractor_id } }"
                      class="task-assignment-link"
                    >
                      {{ contractorsById.get(task.contractor_id)!.name }}
                    </RouterLink>
                  </span>
                  <span
                    v-else
                    class="activity-card__line"
                  >
                    <CIconLocation />
                    {{ contractorDisplayName(task) || t('tasks.contractorUnassigned') }}
                  </span>
                </div>

                <p
                  v-if="task.description"
                  class="body-sm activity-card__description"
                >
                  {{ task.description }}
                </p>

                <div class="activity-card__meta">
                  <span class="activity-card__line">
                    <CIconCalendar />
                    {{ t('tasks.updatedAt', { date: formatDate(task.updated_at) }) }}
                  </span>
                  <span class="activity-card__line">
                    <CIconUser />
                    {{ creatorName(task.created_by) }}
                  </span>
                  <span class="activity-card__line">
                    <CIconFile />
                    {{ t('tasks.documentsCount', task.document_count) }}
                  </span>
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
                  v-if="task.has_vendor_token || vendorLinks[task.id] || task.closed_at || task.is_in_logbook"
                  class="activity-card__note"
                >
                  <span
                    v-if="task.has_vendor_token"
                    class="activity-card__line"
                  >
                    <CIconLink />
                    {{ t('tasks.vendorTokenActive', { date: task.token_expires_at ? formatDate(task.token_expires_at) : '' }) }}
                  </span>
                  <span
                    v-if="task.closed_at"
                    class="activity-card__line"
                  >
                    <CIconCalendar />
                    {{ t('tasks.closedAt', { date: formatDate(task.closed_at) }) }}
                  </span>
                  <span
                    v-if="task.is_in_logbook"
                    class="activity-card__line"
                  >
                    <CIconBook />
                    {{ t('maintenance.title') }}
                  </span>
                  <code
                    v-if="vendorLinks[task.id]"
                    class="token-url"
                  >{{ vendorLinks[task.id].url }}</code>
                  <button
                    v-if="vendorLinks[task.id]"
                    class="icon-btn"
                    :title="t('tasks.vendorTokenCopied')"
                    @click="copyToClipboard(vendorLinks[task.id].url)"
                  >
                    <CIconCopy
                      width="14"
                      height="14"
                    />
                  </button>
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
          :loading="savingTask"
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
  </div>
</template>

<style scoped>
.task-list { display: flex; flex-direction: column; gap: 10px; }

.task-card { overflow: hidden; }
.task-title-link {
  color: inherit;
  text-decoration: none;
  font-weight: var(--fw-semibold);
}
.task-title-link:hover { text-decoration: underline; color: var(--color-primary); }
.task-assignment-link {
  color: var(--color-primary);
  text-decoration: none;
  font-weight: var(--fw-medium);
}
.task-assignment-link:hover { text-decoration: underline; }
.token-url {
  font-size: var(--text-2xs);
  word-break: break-all;
  flex: 1;
}

.assignment-options {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-2) 0;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  border: none;
  background: var(--color-surface-container-high);
  color: var(--color-on-surface);
  cursor: pointer;
  min-width: unset;
  min-height: unset;
}

.icon-btn:hover {
  background: var(--color-secondary-container);
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
</style>
