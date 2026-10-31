<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import CAlert from '@/components/atoms/CAlert.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CCard from '@/components/molecules/CCard.vue'
import CCardActions from '@/components/molecules/CCardActions.vue'
import CMetaLine from '@/components/molecules/CMetaLine.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'
import CLinkedDocumentsModal from '@/components/organisms/CLinkedDocumentsModal.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconSearch from '@/components/atoms/CIconSearch.svg'
import CIconLink from '@/components/atoms/CIconLink.svg'
import CIconEye from '@/components/atoms/CIconEye.svg'
import CIconX from '@/components/atoms/CIconX.svg'
import CIconBan from '@/components/atoms/CIconBan.svg'
import CIconCopy from '@/components/atoms/CIconCopy.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconRefresh from '@/components/atoms/CIconRefresh.svg'
import CIconCheckCircle from '@/components/atoms/CIconCheckCircle.svg'
import SaveTaskModal from './SaveTaskModal.vue'
import CloseTaskModal from './CloseTaskModal.vue'
import DeleteTaskModal from './DeleteTaskModal.vue'
import CancelTaskModal from './CancelTaskModal.vue'
import RevokeTokenModal from './RevokeTokenModal.vue'
import ReopenTaskModal from './ReopenTaskModal.vue'
import { contractorsApi, documentsApi, tasksApi, usersApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import { usePermissions } from '@/composables/usePermissions'
import type {Contractor, Document, Task, TaskListFilters, TaskStatus, User} from '@/types/domain'
import { ALL_FILTER, TASK_PRIORITY_HIGH, TASK_PRIORITY_LOW, TASK_PRIORITY_NORMAL, TASK_PRIORITY_URGENT, TASK_STATUS_AWAITING_VALIDATION, TASK_STATUS_AWAITING_VENDOR, TASK_STATUS_CANCELLED, TASK_STATUS_CLOSED, TASK_STATUS_IN_PROGRESS, TASK_STATUS_OPEN } from '@/utils/constants'

const { t } = useI18n()
const { canAdministerTask } = usePermissions()
const { formatDate, formatRelative } = useFormatters()
const router = useRouter()

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const tasks = ref<Task[]>([])
const totalTasks = ref(0)
const allContractors = ref<Contractor[]>([])
const allUsers = ref<User[]>([])
const statusFilter = ref<typeof ALL_FILTER | TaskStatus>(ALL_FILTER)
const priorityFilter = ref<string>(ALL_FILTER)
const assigneeFilter = ref<string>(ALL_FILTER)
const contractorFilter = ref<string>(ALL_FILTER)
const searchQuery = ref('')
const loadingMore = ref(false)
const PAGE_SIZE = 50
const SEARCH_DEBOUNCE_MS = 300
const VENDOR_LINK_COPY_FEEDBACK_DELAY_MS = 1800

// ── Modal trigger state ───────────────────────────────────────────
const showTaskModal = ref(false)
const editingTask = ref<Task | null>(null)

const closingTask = ref<Task | null>(null)

const showDeleteModal = ref(false)
const taskToDelete = ref<Task | null>(null)

const cancellingTask = ref<Task | null>(null)

const revokingTask = ref<Task | null>(null)

const reopeningTask = ref<Task | null>(null)

// Map for looking up contractor name by id on task cards
const contractorsById = computed(() =>
  new Map(allContractors.value.map(c => [c.id, c])),
)

const usersById = computed(() =>
  new Map(allUsers.value.map(user => [user.id, user])),
)

// ── Load data ─────────────────────────────────────────────────────
function currentFilters(offset: number): TaskListFilters {
  const filters: TaskListFilters = { limit: PAGE_SIZE, offset }
  if (statusFilter.value !== ALL_FILTER) filters.status = statusFilter.value
  if (priorityFilter.value !== ALL_FILTER) filters.priority = priorityFilter.value
  if (assigneeFilter.value !== ALL_FILTER) filters.assigned_to = assigneeFilter.value
  if (contractorFilter.value !== ALL_FILTER) filters.contractor_id = contractorFilter.value
  const q = searchQuery.value.trim()
  if (q) filters.q = q
  return filters
}

async function fetchTasks(offset: number) {
  const res = await tasksApi.list(currentFilters(offset))
  if (offset === 0) {
    tasks.value = res.data.items
  } else {
    tasks.value.push(...res.data.items)
  }
  totalTasks.value = res.data.total
}

const counts = ref<Record<string, number>>({})

async function loadData() {
  const [usersRes, building, contractorsRes] = await Promise.all([
    usersApi.list(),
    useBuildingStore().fetch(),
    contractorsApi.list(),
  ])
  allUsers.value = usersRes.data
  buildingName.value = building.name
  allContractors.value = contractorsRes.data
  const [tasksPage, countersRes] = await Promise.all([
    fetchTasks(0),
    tasksApi.statusCounters(),
  ])
  void tasksPage
  const counters = new Map(countersRes.data.map(c => [c.status, c.count]))
  counts.value = {
    all: countersRes.data.reduce((sum, c) => sum + c.count, 0),
    open: counters.get(TASK_STATUS_OPEN) ?? 0,
    in_progress: counters.get(TASK_STATUS_IN_PROGRESS) ?? 0,
    awaiting_vendor: counters.get(TASK_STATUS_AWAITING_VENDOR) ?? 0,
    awaiting_validation: counters.get(TASK_STATUS_AWAITING_VALIDATION) ?? 0,
    closed: counters.get(TASK_STATUS_CLOSED) ?? 0,
  }
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

watch([statusFilter, priorityFilter, assigneeFilter, contractorFilter], () => reload())

let searchTimer: number | undefined
watch(searchQuery, () => {
  window.clearTimeout(searchTimer)
  searchTimer = window.setTimeout(() => reload(), SEARCH_DEBOUNCE_MS)
})

async function loadMore() {
  loadingMore.value = true
  try {
    await fetchTasks(tasks.value.length)
  } finally {
    loadingMore.value = false
  }
}

// ── Derived ───────────────────────────────────────────────────────
const hasActiveFilters = computed(() =>
  statusFilter.value !== ALL_FILTER ||
  priorityFilter.value !== ALL_FILTER ||
  assigneeFilter.value !== ALL_FILTER ||
  contractorFilter.value !== ALL_FILTER ||
  searchQuery.value.trim() !== '',
)

const hasMore = computed(() => tasks.value.length < totalTasks.value)

const activeCount = computed(() =>
  (counts.value.open ?? 0) +
  (counts.value.in_progress ?? 0) +
  (counts.value.awaiting_vendor ?? 0) +
  (counts.value.awaiting_validation ?? 0),
)

const priorityOptions = computed(() => [
  { value: ALL_FILTER, label: t('common.allFem') + ' ' + t('common.priority').toLowerCase() + 's' },
  { value: TASK_PRIORITY_LOW, label: t(`tasks.priorities.${TASK_PRIORITY_LOW}`) },
  { value: TASK_PRIORITY_NORMAL, label: t(`tasks.priorities.${TASK_PRIORITY_NORMAL}`) },
  { value: TASK_PRIORITY_HIGH, label: t(`tasks.priorities.${TASK_PRIORITY_HIGH}`) },
  { value: TASK_PRIORITY_URGENT, label: t(`tasks.priorities.${TASK_PRIORITY_URGENT}`) },
])

const assigneeOptions = computed(() => [
  { value: ALL_FILTER, label: t('common.allMasc') },
  ...allUsers.value.map(user => ({
    value: user.id,
    label: `${user.first_name} ${user.last_name}`,
  })),
])

const contractorOptions = computed(() => [
  { value: ALL_FILTER, label: t('common.allMasc') + ' ' + t('common.contractor').toLowerCase() + 's' },
  ...allContractors.value.map(contractor => ({
    value: contractor.id,
    label: contractor.name,
  })),
])

// ── Helpers ───────────────────────────────────────────────────────
function isTerminal(status: string): boolean {
  return status === TASK_STATUS_CLOSED || status === TASK_STATUS_CANCELLED
}

function statusVariant(s: string): 'valid' | 'open' | 'pending' | 'closed' | 'default' {
  if (s === TASK_STATUS_OPEN) return 'valid'
  if (s === TASK_STATUS_IN_PROGRESS) return 'open'
  if (s === TASK_STATUS_AWAITING_VENDOR) return 'default'
  if (s === TASK_STATUS_AWAITING_VALIDATION) return 'pending'
  if (s === TASK_STATUS_CLOSED) return 'closed'
  return 'default'
}

function closeActionLabel(task: Task): string {
  return task.status === TASK_STATUS_AWAITING_VALIDATION
    ? t('tasks.validateTask')
    : t('tasks.closeTask')
}

function priorityVariant(p: string): 'err' | 'pending' | 'primary' | 'valid' | 'default' {
  if (p === TASK_PRIORITY_URGENT) return 'err'
  if (p === TASK_PRIORITY_HIGH) return 'pending'
  if (p === TASK_PRIORITY_NORMAL) return 'primary'
  if (p === TASK_PRIORITY_LOW) return 'valid'
  return 'default'
}

function contractorDisplayName(task: Task): string | undefined {
  return contractorsById.value.get(task.contractor_id ?? '')?.name ?? task.contractor_name
}

function assignedUserName(task: Task): string | null {
  const assignedUser = task.assigned_to ? usersById.value.get(task.assigned_to) : undefined
  if (!assignedUser) return null
  return `${assignedUser.first_name} ${assignedUser.last_name}`
}

function hasAssignment(task: Task): boolean {
  return Boolean(contractorDisplayName(task) || assignedUserName(task))
}

function openTaskDetail(taskId: string) {
  void router.push({ name: 'task-detail', params: { id: taskId } })
}

// ── Modal openers ─────────────────────────────────────────────────
function openCreateModal() {
  editingTask.value = null
  showTaskModal.value = true
}

function openEditModal(task: Task) {
  editingTask.value = task
  showTaskModal.value = true
}

function openCloseModal(task: Task) {
  closingTask.value = task
}

function openReopenModal(task: Task) {
  reopeningTask.value = task
}

function openRevokeModal(task: Task) {
  revokingTask.value = task
}

function openDeleteModal(task: Task) {
  showTaskModal.value = false
  taskToDelete.value = task
  showDeleteModal.value = true
}

function openCancelModal(task: Task) {
  cancellingTask.value = task
}

// ── Modal event handlers ──────────────────────────────────────────
function onTaskSaved() {
  reload()
}

function onTaskDeleteRequest(task: Task) {
  openDeleteModal(task)
}

function onTaskDeleted() {
  showDeleteModal.value = false
  reload()
}

function onDeleteCancelRequested(task: Task) {
  showDeleteModal.value = false
  cancellingTask.value = task
}

function onTaskCancelled() {
  reload()
}

function onTaskClosed() {
  reload()
}

function onTaskReopened() {
  reload()
}

function onTokenRevoked() {
  if (revokingTask.value) delete vendorLinks.value[revokingTask.value.id]
  reload()
}

function onDocumentDetached(taskId: string, documentId: string) {
  updateTaskDocumentCount(taskId, -1)
  if (taskDocumentsTarget.value?.id === taskId) {
    taskDocuments.value = taskDocuments.value.filter(d => d.id !== documentId)
  }
}

function updateTaskDocumentCount(taskId: string, delta: number) {
  const task = tasks.value.find(currentTask => currentTask.id === taskId)
  if (task) {
    task.document_count = Math.max(0, task.document_count + delta)
  }
}

// ── Vendor token actions ───────────────────────────────────────────
const generatingToken = ref<string | null>(null)
const vendorLinks = ref<Record<string, { url: string; expires_at: string }>>({})
const pageError = ref('')
const copiedVendorLinkTaskId = ref<string | null>(null)

async function generateToken(task: Task) {
  generatingToken.value = task.id
  pageError.value = ''
  try {
    const res = await tasksApi.generateToken(task.id)
    const tokenStr = res.data.token
    const url = `${window.location.origin}/vendor/${tokenStr}`
    vendorLinks.value[task.id] = { url, expires_at: res.data.expires_at }
    await reload()
  } catch (err) {
    pageError.value = toErrorMessage(err)
  } finally {
    generatingToken.value = null
  }
}

async function showVendorLink(task: Task) {
  if (vendorLinks.value[task.id]) return
  pageError.value = ''
  try {
    const res = await tasksApi.getVendorTokenUrl(task.id)
    const parsedPath = new URL(res.data.url).pathname
    const tokenSegment = parsedPath.split('/vendor/')[1]
    const url = tokenSegment
      ? `${window.location.origin}/vendor/${tokenSegment}`
      : res.data.url
    vendorLinks.value[task.id] = { url, expires_at: res.data.expires_at }
  } catch (err) {
    pageError.value = toErrorMessage(err)
  }
}

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
    pageError.value = toErrorMessage(err)
  }
}

// ── Linked documents modal ─────────────────────────────────────────
const taskDocumentsTarget = ref<Task | null>(null)
const taskDocuments = ref<Document[]>([])
const loadingTaskDocuments = ref(false)

async function openTaskDocuments(task: Task) {
  taskDocumentsTarget.value = task
  taskDocuments.value = []
  loadingTaskDocuments.value = true
  try {
    const response = await tasksApi.listDocuments(task.id)
    taskDocuments.value = response.data
  } catch (err) {
    pageError.value = toErrorMessage(err)
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
    pageError.value = toErrorMessage(err)
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
          <span>{{ t('nav.tasks') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('tasks.create')"
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
            {{ t('page.eyebrows.tasks') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('nav.tasks') }}
          </CHeading>
          <CMetaItem variant="sub">
            {{ t('tasks.header.activeSummary', { count: activeCount }) }}
          </CMetaItem>
        </header>

        <CAlert
          v-if="pageError"
          variant="error"
        >
          {{ pageError }}
        </CAlert>

        <!-- Filtres -->
        <CTabs
          v-model="statusFilter"
          variant="filter"
          :tabs="[
            { value: ALL_FILTER, label: t('common.allMasc'), count: counts.all ?? 0 },
            { value: TASK_STATUS_OPEN, label: t('tasks.filters.open'), count: counts.open ?? 0 },
            { value: TASK_STATUS_IN_PROGRESS, label: t('tasks.filters.in_progress'), count: counts.in_progress ?? 0 },
            { value: TASK_STATUS_AWAITING_VENDOR, label: t('common.contractor'), count: counts.awaiting_vendor ?? 0 },
            { value: TASK_STATUS_AWAITING_VALIDATION, label: t('tasks.filters.awaiting_validation'), count: counts.awaiting_validation ?? 0 },
            { value: TASK_STATUS_CLOSED, label: t('tasks.filters.closed'), count: counts.closed ?? 0 },
          ]"
        />

        <div class="flex flex-wrap items-end gap-3">
          <CFormInput
            v-model="searchQuery"
            type="search"
            class="min-w-56 flex-1"
            :placeholder="`${t('common.search')} ${t('nav.tasks').toLowerCase()}…`"
          >
            <template #prefix>
              <CIconSearch />
            </template>
          </CFormInput>
          <CFormSelect
            v-model="priorityFilter"
            :label="t('common.priority')"
            :options="priorityOptions"
          />
          <CFormSelect
            v-model="assigneeFilter"
            :label="t('common.assignee')"
            :options="assigneeOptions"
          />
          <CFormSelect
            v-model="contractorFilter"
            :label="t('common.contractor')"
            :options="contractorOptions"
          />
        </div>

        <template v-if="tasks.length === 0">
          <CEmptyState
            :title="hasActiveFilters ? t('tasks.emptyFiltered') : t('tasks.emptyState')"
          />
        </template>
        <template v-else>
          <div class="flex flex-col gap-3">
            <CCard
              v-for="task in tasks"
              :key="task.id"
              :shadow="!isTerminal(task.status)"
              gap="sm"
              :class="['task-card', { 'outline outline-1 outline-outline-variant': isTerminal(task.status) }]"
              interactive
              tabindex="0"
              @click="openTaskDetail(task.id)"
              @keydown.enter.prevent="openTaskDetail(task.id)"
              @keydown.space.prevent="openTaskDetail(task.id)"
            >
              <div class="flex items-center gap-3 min-w-0">
                <h3 class="text-lg font-semibold text-on-surface m-0 leading-snug min-w-0">
                  {{ task.title }}
                </h3>
                <CMetaLine class="flex-1 justify-end text-xs">
                  <CBadge :variant="isTerminal(task.status) ? 'disabled' : priorityVariant(task.priority)">
                    {{ t(`tasks.priorities.${task.priority}`) }}
                  </CBadge>
                  <CBadge
                    :variant="isTerminal(task.status) ? 'disabled' : statusVariant(task.status)"
                    dot
                  >
                    {{ t(`tasks.statuses.${task.status}`) }}
                  </CBadge>
                </CMetaLine>
              </div>
  
              <div class="flex flex-col gap-3">
                <p
                  v-if="task.description"
                  class="body-xs"
                >
                  {{ task.description }}
                </p>
  
                <div
                  v-if="task.document_count > 0"
                  class="flex flex-wrap gap-x-2 gap-y-1"
                >
                  <button
                    class="inline-flex items-center gap-1 bg-transparent border-none p-0 cursor-pointer text-xs hover:text-primary"
                    @click.stop="openTaskDocuments(task)"
                  >
                    <CIconFile />
                    {{ t('common.documentsCount', task.document_count) }}
                  </button>
                </div>
  
                <CMetaLine>
                  <CMetaItem
                    v-if="hasAssignment(task)"
                  >
                    <template #icon>
                      <CIconWrench v-if="task.contractor_id || contractorDisplayName(task)" />
                      <CIconUser v-else />
                    </template>
                    <RouterLink
                      v-if="task.contractor_id && contractorDisplayName(task)"
                      :to="{ name: 'contractor-detail', params: { id: task.contractor_id } }"
                      class="task-card__meta-link"
                      @click.stop
                    >
                      {{ contractorDisplayName(task) }}
                    </RouterLink>
                    <span v-else>
                      {{ contractorDisplayName(task) ?? assignedUserName(task) }}
                    </span>
                  </CMetaItem>
                  <CMetaItem>
                    <template #icon>
                      <CIconCalendar />
                    </template>
                    {{ t('tasks.lifecycle.createdRelative', { relative: formatRelative(task.created_at) }) }}
                  </CMetaItem>
                </CMetaLine>
  
                <div
                  v-if="(contractorDisplayName(task) && !task.has_vendor_token && !isTerminal(task.status)) || task.has_vendor_token"
                  class="flex flex-wrap gap-x-2 gap-y-1"
                >
                  <CBtn
                    v-if="contractorDisplayName(task) && !task.has_vendor_token && !isTerminal(task.status)"
                    variant="ghost"
                    size="sm"
                    :icon="CIconLink"
                    :title="t('tasks.generateToken')"
                    :disabled="generatingToken === task.id"
                    :aria-label="t('tasks.generateToken')"
                    @click.stop="generateToken(task)"
                  />
                  <CBtn
                    v-if="task.has_vendor_token"
                    variant="ghost"
                    size="sm"
                    :icon="CIconEye"
                    :title="t('tasks.showLink')"
                    :aria-label="t('tasks.showLink')"
                    @click.stop="showVendorLink(task)"
                  />
                  <CBtn
                    v-if="task.has_vendor_token"
                    variant="ghost"
                    size="sm"
                    :icon="CIconX"
                    :title="t('tasks.revokeToken')"
                    :aria-label="t('tasks.revokeToken')"
                    @click.stop="openRevokeModal(task)"
                  />
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
                      @click.stop
                    >
                      {{ vendorLinks[task.id].url }}
                    </a>
                  </div>
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :icon="CIconCopy"
                    :title="copiedVendorLinkTaskId === task.id ? t('tasks.vendorTokenCopied') : t('common.copy')"
                    @click.stop="copyVendorLink(task.id)"
                  />
                </div>
  
                <CCardActions>
                  <CBtn
                    v-if="!isTerminal(task.status)"
                    variant="ghost"
                    size="sm"
                    :icon="CIconEdit"
                    :title="t('common.edit')"
                    :aria-label="t('tasks.editTask')"
                    @click.stop="openEditModal(task)"
                  />
                  <CBtn
                    v-if="task.status === TASK_STATUS_CLOSED"
                    variant="primary"
                    size="sm"
                    :icon="CIconRefresh"
                    :title="t('tasks.reopenTask')"
                    :aria-label="t('tasks.reopenTask')"
                    @click.stop="openReopenModal(task)"
                  />
                  <CBtn
                    v-else-if="!isTerminal(task.status)"
                    variant="primary"
                    size="sm"
                    :icon="CIconCheckCircle"
                    :title="closeActionLabel(task)"
                    :aria-label="closeActionLabel(task)"
                    @click.stop="openCloseModal(task)"
                  />
                  <CBtn
                    v-if="canAdministerTask && !isTerminal(task.status)"
                    variant="ghost"
                    size="sm"
                    :icon="CIconBan"
                    :title="t('tasks.cancelTask')"
                    :aria-label="t('tasks.cancelTask')"
                    @click.stop="openCancelModal(task)"
                  />
                </CCardActions>
              </div>
            </CCard>
          </div>
          <div
            v-if="hasMore"
            class="flex justify-center"
          >
            <CBtn
              variant="ghost"
              :loading="loadingMore"
              @click="loadMore"
            >
              {{ t('common.loadMore') }}
            </CBtn>
          </div>
        </template>
      </template>
    </div>

    <SaveTaskModal
      :open="showTaskModal"
      :task="editingTask"
      :contractors="allContractors"
      @update:open="showTaskModal = $event"
      @saved="onTaskSaved"
      @delete="onTaskDeleteRequest"
      @document-detached="onDocumentDetached"
    />

    <RevokeTokenModal
      :open="!!revokingTask"
      :task="revokingTask"
      @update:open="v => !v && (revokingTask = null)"
      @revoked="onTokenRevoked"
    />

    <CloseTaskModal
      :open="!!closingTask"
      :task="closingTask"
      @update:open="(v?: boolean) => !v && (closingTask = null)"
      @closed="onTaskClosed"
    />

    <ReopenTaskModal
      :open="!!reopeningTask"
      :task="reopeningTask"
      @update:open="v => !v && (reopeningTask = null)"
      @reopened="onTaskReopened"
    />

    <DeleteTaskModal
      :open="showDeleteModal"
      :task="taskToDelete"
      @update:open="showDeleteModal = $event"
      @deleted="onTaskDeleted"
      @cancel-requested="onDeleteCancelRequested"
    />

    <CancelTaskModal
      :open="!!cancellingTask"
      :task="cancellingTask"
      @update:open="v => !v && (cancellingTask = null)"
      @cancelled="onTaskCancelled"
    />

    <CLinkedDocumentsModal
      :open="!!taskDocumentsTarget"
      :title="taskDocumentsTarget?.title ?? ''"
      :documents="taskDocuments"
      :loading="loadingTaskDocuments"
      :empty-message="t('tasks.noLinkedDocuments')"
      @update:open="value => !value && (taskDocumentsTarget = null)"
      @download="downloadTaskDocument"
    />
  </div>
</template>

<style scoped>
.task-card__meta-link {
  color: inherit;
  text-decoration: none;
}
.task-card__meta-link:hover { color: var(--color-primary); text-decoration: underline; }
.task-card__vendor-link {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-xl);
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
</style>
