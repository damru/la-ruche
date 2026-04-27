<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback, SUCCESS_AUTO_CLEAR_MS } from '@/composables/useFormFeedback'
import { useConfirm } from '@/composables/useConfirm'
import CAlert from '@/components/atoms/CAlert.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { buildingApi, contractorsApi, tasksApi } from '@/api'
import type { Contractor, Task, TaskStatus, TaskPriority } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()
const { confirm } = useConfirm()

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const allTasks = ref<Task[]>([])
const allContractors = ref<Contractor[]>([])
const statusFilter = ref<'all' | TaskStatus>('all')

// ── Task form modal ───────────────────────────────────────────────
const showTaskModal = ref(false)
const taskFeedback = useFormFeedback()
const savingTask = ref(false)
const editingTask = ref<Task | null>(null)

// Contractor autocomplete state
const contractorSearch = ref('')
const contractorSuggestions = ref<Contractor[]>([])
const showSuggestions = ref(false)
const selectedContractor = ref<Contractor | null>(null)

function onContractorInput() {
  const q = contractorSearch.value.trim().toLowerCase()
  if (!q) {
    contractorSuggestions.value = []
    showSuggestions.value = false
    selectedContractor.value = null
    return
  }
  contractorSuggestions.value = allContractors.value
    .filter(c => c.name.toLowerCase().includes(q))
    .slice(0, 8)
  showSuggestions.value = contractorSuggestions.value.length > 0
}

function selectContractorSuggestion(c: Contractor) {
  selectedContractor.value = c
  contractorSearch.value = c.name
  // Hide immediately — mousedown.prevent on the item ensures this fires before blur
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
  showSuggestions.value = false
}

// Map for looking up contractor name by id on task cards
const contractorsById = computed(() =>
  new Map(allContractors.value.map(c => [c.id, c])),
)

const defaultForm = () => ({
  title: '',
  description: '',
  priority: 'normal' as string,
})
const taskForm = ref(defaultForm())

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [buildingRes, tasksRes, contractorsRes] = await Promise.all([
    buildingApi.get(),
    tasksApi.list(),
    contractorsApi.list(),
  ])
  buildingName.value = buildingRes.data.name
  allTasks.value = tasksRes.data
  allContractors.value = contractorsRes.data
}

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
onMounted(reload)

// ── Derived ───────────────────────────────────────────────────────
const tasks = computed(() =>
  statusFilter.value === 'all'
    ? allTasks.value
    : allTasks.value.filter(t => t.status === statusFilter.value),
)

const counts = computed(() => ({
  open: allTasks.value.filter(t => t.status === 'open').length,
  in_progress: allTasks.value.filter(t => t.status === 'in_progress').length,
  awaiting_vendor: allTasks.value.filter(t => t.status === 'awaiting_vendor').length,
  closed: allTasks.value.filter(t => t.status === 'closed').length,
}))

const activeCount = computed(() =>
  allTasks.value.filter(t => t.status !== 'closed' && t.status !== 'cancelled').length,
)

// ── Helpers ───────────────────────────────────────────────────────
function statusVariant(s: string): 'valid' | 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'open') return 'valid'
  if (s === 'in_progress') return 'open'
  if (s === 'awaiting_vendor') return 'default'
  if (s === 'closed') return 'closed'
  return 'default'
}

function priorityVariant(p: string): 'err' | 'pending' | 'primary' | 'valid' | 'default' {
  if (p === 'urgent') return 'err'
  if (p === 'high') return 'pending'
  if (p === 'normal') return 'primary'
  if (p === 'low') return 'valid'
  return 'default'
}

function priorityColor(p: string): string {
  if (p === 'urgent') return 'var(--color-error)'
  if (p === 'high') return 'var(--color-warning)'
  if (p === 'normal') return 'var(--color-primary)'
  return 'var(--color-success)'
}

// ── Task modal actions ────────────────────────────────────────────
function openCreateModal() {
  editingTask.value = null
  taskForm.value = defaultForm()
  selectedContractor.value = null
  contractorSearch.value = ''
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
    contractorSearch.value = c?.name ?? ''
  } else {
    selectedContractor.value = null
    contractorSearch.value = ''
  }
  taskFeedback.reset()
  showTaskModal.value = true
}

async function saveTask() {
  if (!taskForm.value.title.trim()) return
  savingTask.value = true
  taskFeedback.reset()
  try {
    if (editingTask.value) {
      await tasksApi.update(editingTask.value.id, {
        title: taskForm.value.title,
        description: taskForm.value.description || null,
        priority: taskForm.value.priority,
        contractor_id: selectedContractor.value?.id ?? null,
      })
    } else {
      await tasksApi.create({
        title: taskForm.value.title,
        description: taskForm.value.description || undefined,
        priority: taskForm.value.priority as TaskPriority,
        contractor_id: selectedContractor.value?.id ?? null,
      })
    }
    taskFeedback.setSuccess(t('common.saved'))
    setTimeout(() => {
      showTaskModal.value = false
      reload()
    }, SUCCESS_AUTO_CLEAR_MS)
  } catch (err) {
    taskFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    savingTask.value = false
  }
}

// ── Delete task ───────────────────────────────────────────────────
const pageError = ref('')

async function deleteTask(task: Task) {
  const ok = await confirm({
    title: t('tasks.confirmDeleteTitle'),
    message: t('tasks.confirmDeleteMessage', { title: task.title }),
    confirmLabel: t('common.delete'),
    variant: 'danger',
  })
  if (!ok) return
  pageError.value = ''
  try {
    await tasksApi.delete(task.id)
    await reload()
  } catch (err) {
    pageError.value = toErrorMessage(err, t('common.error'))
  }
}

// ── Quick status actions ──────────────────────────────────────────
const updatingStatus = ref<string | null>(null)

async function updateTaskStatus(taskId: string, status: string) {
  updatingStatus.value = taskId
  pageError.value = ''
  try {
    await tasksApi.update(taskId, { status })
    await reload()
  } catch (err) {
    pageError.value = toErrorMessage(err, t('common.error'))
  } finally {
    updatingStatus.value = null
  }
}

async function takeCharge(task: Task) {
  openTakeChargeModal(task)
}

async function closeTaskManually(task: Task) {
  const ok = await confirm({
    title: t('tasks.confirmCloseTitle'),
    message: t('tasks.confirmCloseMessage', { title: task.title }),
    confirmLabel: t('tasks.closeTask'),
    variant: 'primary',
  })
  if (!ok) return
  await updateTaskStatus(task.id, 'closed')
}

// ── Vendor token actions ───────────────────────────────────────────
const generatingToken = ref<string | null>(null)
// vendorLinks: task.id → { url, expires_at } — populated on demand
const vendorLinks = ref<Record<string, { url: string; expires_at: string }>>({})

// ── Revoke token modal ─────────────────────────────────────────────
const revokingTask = ref<Task | null>(null)
const revokeFeedback = useFormFeedback()
const revoking = ref(false)

function openRevokeModal(task: Task) {
  revokingTask.value = task
  revokeFeedback.reset()
}

async function confirmRevoke() {
  if (!revokingTask.value) return
  revoking.value = true
  revokeFeedback.reset()
  try {
    await tasksApi.revokeToken(revokingTask.value.id)
    delete vendorLinks.value[revokingTask.value.id]
    revokeFeedback.setSuccess(t('common.saved'))
    setTimeout(() => { revokingTask.value = null; reload() }, SUCCESS_AUTO_CLEAR_MS)
  } catch (err) {
    revokeFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    revoking.value = false
  }
}

// ── Take charge modal ─────────────────────────────────────────────
const takingChargeTask = ref<Task | null>(null)
const takeChargeFeedback = useFormFeedback()
const takingCharge = ref(false)

function openTakeChargeModal(task: Task) {
  takingChargeTask.value = task
  takeChargeFeedback.reset()
}

async function confirmTakeCharge() {
  if (!takingChargeTask.value) return
  takingCharge.value = true
  takeChargeFeedback.reset()
  try {
    await tasksApi.update(takingChargeTask.value.id, { status: 'in_progress' })
    takeChargeFeedback.setSuccess(t('common.saved'))
    setTimeout(() => { takingChargeTask.value = null; reload() }, SUCCESS_AUTO_CLEAR_MS)
  } catch (err) {
    takeChargeFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    takingCharge.value = false
  }
}

// ── Reopen task modal ─────────────────────────────────────────────
const reopeningTask = ref<Task | null>(null)
const reopenFeedback = useFormFeedback()
const reopening = ref(false)

function openReopenModal(task: Task) {
  reopeningTask.value = task
  reopenFeedback.reset()
}

async function confirmReopen() {
  if (!reopeningTask.value) return
  reopening.value = true
  reopenFeedback.reset()
  try {
    await tasksApi.update(reopeningTask.value.id, { status: 'open' })
    reopenFeedback.setSuccess(t('common.saved'))
    setTimeout(() => { reopeningTask.value = null; reload() }, SUCCESS_AUTO_CLEAR_MS)
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
            padding="none"
            class="task-card"
          >
            <div class="task-row">
              <div
                class="prio-bar"
                :style="{ background: priorityColor(task.priority) }"
              ></div>
              <div class="task-body">
                <div class="task-head">
                  <div class="task-badges">
                    <CBadge :variant="priorityVariant(task.priority)">
                      {{ t(`tasks.priorities.${task.priority}`) }}
                    </CBadge>
                    <CBadge :variant="statusVariant(task.status)" dot>
                      {{ t(`tasks.statuses.${task.status}`) }}
                    </CBadge>
                  </div>
                  <div class="task-actions">
                    <button
                      class="icon-btn"
                      :title="t('tasks.editTask')"
                      @click="openEditModal(task)"
                    >
                      <CIconEdit
                        width="15"
                        height="15"
                      />
                    </button>
                    <button
                      class="icon-btn icon-btn--danger"
                      :title="t('tasks.deleteTask')"
                      @click="deleteTask(task)"
                    >
                      <CIconTrash
                        width="15"
                        height="15"
                      />
                    </button>
                    <!-- Take charge: open → in_progress (internal, no contractor) -->
                    <button
                      v-if="task.status === 'open'"
                      class="icon-btn icon-btn--valid"
                      :title="t('tasks.takeCharge')"
                      :disabled="updatingStatus === task.id"
                      @click="takeCharge(task)"
                    >
                      <CIconUser
                        width="15"
                        height="15"
                      />
                    </button>
                    <!-- Close manually: in_progress → closed (no vendor needed) -->
                    <button
                      v-if="task.status === 'in_progress'"
                      class="icon-btn icon-btn--valid"
                      :title="t('tasks.closeTask')"
                      :disabled="updatingStatus === task.id"
                      @click="closeTaskManually(task)"
                    >
                      <CIconCheckCircle
                        width="15"
                        height="15"
                      />
                    </button>
                    <!-- Reopen: closed → open -->
                    <button
                      v-if="task.status === 'closed'"
                      class="icon-btn"
                      :title="t('tasks.reopenTask')"
                      :disabled="updatingStatus === task.id"
                      @click="openReopenModal(task)"
                    >
                      <CIconRefresh
                        width="15"
                        height="15"
                      />
                    </button>
                    <!-- Generate link (only if no active token and task is active) -->
                    <button
                      v-if="!task.has_vendor_token && task.status !== 'closed' && task.status !== 'cancelled'"
                      class="icon-btn"
                      :title="t('tasks.generateToken')"
                      :disabled="generatingToken === task.id"
                      @click="generateToken(task)"
                    >
                      <CIconLink
                        width="15"
                        height="15"
                      />
                    </button>
                  </div>
                </div>

                <span class="title-sm task-title">{{ task.title }}</span>
                <p
                  v-if="task.description"
                  class="task-desc body-sm"
                >
                  {{ task.description }}
                </p>

                <!-- Vendor token section -->
                <div
                  v-if="task.has_vendor_token"
                  class="vendor-token"
                >
                  <div class="vendor-token-header">
                    <span
                      class="body-xs"
                      style="color:var(--color-on-surface-variant)"
                    >
                      {{ t('tasks.vendorTokenActive', { date: formatDate(task.token_expires_at!) }) }}
                    </span>
                    <div class="vendor-token-btns">
                      <button
                        class="icon-btn"
                        :title="t('tasks.showLink')"
                        @click="showVendorLink(task)"
                      >
                        <CIconEye
                          width="14"
                          height="14"
                        />
                      </button>
                      <button
                        class="icon-btn icon-btn--danger"
                        :title="t('tasks.revokeToken')"
                        @click="openRevokeModal(task)"
                      >
                        <CIconX
                          width="14"
                          height="14"
                        />
                      </button>
                    </div>
                  </div>
                  <div
                    v-if="vendorLinks[task.id]"
                    class="token-url-row"
                  >
                    <code class="token-url">{{ vendorLinks[task.id].url }}</code>
                    <button
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

                <div class="task-meta">
                  <span>{{ formatRelative(task.created_at) }}</span>
                  <span v-if="task.closed_at">· {{ t('tasks.closedAt', { date: formatDate(task.closed_at) }) }}</span>
                  <span v-if="task.contractor_id && contractorsById.get(task.contractor_id)">
                    · 🔧 {{ contractorsById.get(task.contractor_id)!.name }}
                  </span>
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
        <!-- Contractor assignment with autocomplete -->
        <div class="form-field">
          <label class="form-label">{{ t('tasks.contractorLabel') }}</label>
          <div style="position:relative">
            <input
              v-model="contractorSearch"
              type="text"
              class="form-input"
              :placeholder="t('tasks.contractorPlaceholder')"
              autocomplete="off"
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
              v-if="showSuggestions"
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
                  v-if="c.domain"
                  class="autocomplete-meta"
                >{{ t(`contractors.domains.${c.domain}`, c.domain) }}</span>
              </li>
            </ul>
          </div>
          <span
            v-if="selectedContractor"
            class="field-hint"
          >
            {{ selectedContractor.email || selectedContractor.phone || '' }}
          </span>
        </div>
      </form>
      <template #footer>
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
          @click="confirmRevoke"
        >
          {{ t('tasks.revokeToken') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Take charge modal -->
    <CModal
      :open="!!takingChargeTask"
      :title="t('tasks.confirmTakeChargeTitle')"
      size="sm"
      @update:open="v => !v && (takingChargeTask = null)"
    >
      <p class="body-sm">
        {{ t('tasks.confirmTakeChargeMessage', { title: takingChargeTask?.title ?? '' }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="takeChargeFeedback.status.value"
          :message="takeChargeFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="takingChargeTask = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="takingCharge"
          @click="confirmTakeCharge"
        >
          {{ t('tasks.takeCharge') }}
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
          @click="confirmReopen"
        >
          {{ t('tasks.reopenTask') }}
        </CBtn>
      </template>
    </CModal>
  </div>
</template>

<style scoped>
.task-list { display: flex; flex-direction: column; gap: 10px; }

.task-card { overflow: hidden; }
.task-row { display: flex; align-items: stretch; gap: 0; }

.prio-bar { width: 4px; flex-shrink: 0; }

.task-body { flex: 1; padding: 14px 16px; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
.task-head { display: flex; align-items: center; justify-content: space-between; gap: var(--space-2); }
.task-badges { display: flex; gap: 6px; flex-wrap: wrap; }
.task-title { display: block; margin-top: 4px; }
.task-desc { color: var(--color-on-surface-variant); }
.task-meta { font-size: var(--text-2xs); color: var(--color-on-surface-variant); margin-top: 4px; }

.task-actions { display: flex; gap: 2px; flex-shrink: 0; }

.vendor-token { display: flex; flex-direction: column; gap: 4px; margin: 4px 0; padding: 8px; background: var(--color-surface-container); border-radius: var(--radius-sm); }
.vendor-token-header { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.vendor-token-btns { display: flex; gap: 2px; }
.token-url-row { display: flex; align-items: center; gap: 6px; }
.token-url { font-size: var(--text-2xs); word-break: break-all; flex: 1; }

.empty-state { padding: var(--space-8); text-align: center; font-size: var(--text-md); color: var(--color-on-surface-variant); }

.form-grid { display: flex; flex-direction: column; gap: var(--space-4); }
.form-field { display: flex; flex-direction: column; gap: var(--space-1); }
.form-label { font-size: var(--text-sm); font-weight: var(--fw-medium); color: var(--color-on-surface-variant); }
.form-input { padding: var(--space-2) var(--space-3); border: 1px solid var(--color-outline); border-radius: var(--radius-md); font-size: var(--text-sm); background: var(--color-surface); color: var(--color-on-surface); }
.form-input:focus { outline: none; border-color: var(--color-primary); }

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
</style>
