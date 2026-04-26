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
import { buildingApi, tasksApi } from '@/api'
import type { Task, TaskStatus, TaskPriority } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()
const { confirm } = useConfirm()

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const allTasks = ref<Task[]>([])
const statusFilter = ref<'all' | TaskStatus>('all')

// ── Task form modal ───────────────────────────────────────────────
const showTaskModal = ref(false)
const taskFeedback = useFormFeedback()
const savingTask = ref(false)
const editingTask = ref<Task | null>(null)

const defaultForm = () => ({
  title: '',
  description: '',
  priority: 'normal' as string,
})
const taskForm = ref(defaultForm())

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [buildingRes, tasksRes] = await Promise.all([
    buildingApi.get(),
    tasksApi.list(),
  ])
  buildingName.value = buildingRes.data.name
  allTasks.value = tasksRes.data
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
function statusVariant(s: string): 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'open') return 'pending'
  if (s === 'in_progress') return 'open'
  if (s === 'awaiting_vendor') return 'default'
  if (s === 'closed') return 'closed'
  return 'default'
}

function priorityVariant(p: string): 'open' | 'pending' | 'closed' | 'default' {
  if (p === 'urgent') return 'closed'
  if (p === 'high') return 'open'
  if (p === 'normal') return 'pending'
  return 'default'
}

function priorityColor(p: string): string {
  if (p === 'urgent') return 'var(--color-error)'
  if (p === 'high') return 'var(--color-warning)'
  if (p === 'normal') return 'var(--color-primary)'
  return 'var(--color-outline-variant)'
}

// ── Task modal actions ────────────────────────────────────────────
function openCreateModal() {
  editingTask.value = null
  taskForm.value = defaultForm()
  taskFeedback.reset()
  showTaskModal.value = true
}

async function saveTask() {
  if (!taskForm.value.title.trim()) return
  savingTask.value = true
  taskFeedback.reset()
  try {
    if (editingTask.value) {
      await tasksApi.update(editingTask.value.id, { status: editingTask.value.status })
    } else {
      await tasksApi.create({
        title: taskForm.value.title,
        description: taskForm.value.description || undefined,
        priority: taskForm.value.priority as TaskPriority,
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

// ── Vendor token actions ───────────────────────────────────────────
const generatingToken = ref<string | null>(null)
const tokenData = ref<Record<string, { url: string; expires_at: string }>>({})

const pageError = ref('')

async function generateToken(task: Task) {
  generatingToken.value = task.id
  pageError.value = ''
  try {
    const res = await tasksApi.generateToken(task.id)
    tokenData.value[task.id] = { url: res.data.url, expires_at: res.data.expires_at }
    await reload()
  } catch (err) {
    pageError.value = toErrorMessage(err, t('common.error'))
  } finally {
    generatingToken.value = null
  }
}

async function revokeToken(task: Task) {
  const ok = await confirm({
    title: t('tasks.revokeToken'),
    message: t('tasks.confirmRevokeToken'),
    confirmLabel: t('tasks.revokeToken'),
    variant: 'danger',
  })
  if (!ok) return
  pageError.value = ''
  try {
    await tasksApi.revokeToken(task.id)
    delete tokenData.value[task.id]
    await reload()
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

        <CAlert v-if="pageError" variant="error">{{ pageError }}</CAlert>

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
                  <span class="title-sm">{{ task.title }}</span>
                  <div class="task-badges">
                    <CBadge :variant="priorityVariant(task.priority)">
                      {{ t(`tasks.priorities.${task.priority}`) }}
                    </CBadge>
                    <CBadge :variant="statusVariant(task.status)">
                      {{ t(`tasks.statuses.${task.status}`) }}
                    </CBadge>
                  </div>
                </div>
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
                  <span
                    class="body-xs"
                    style="color:var(--color-on-surface-variant)"
                  >
                    {{ t('tasks.vendorTokenActive', { date: formatDate(task.token_expires_at!) }) }}
                  </span>
                  <div
                    v-if="tokenData[task.id]"
                    class="token-url-row"
                  >
                    <code class="token-url">{{ tokenData[task.id].url }}</code>
                    <button
                      class="icon-btn"
                      :title="t('tasks.vendorTokenCopied')"
                      @click="copyToClipboard(tokenData[task.id].url)"
                    >
                      <CIconCheck
                        width="14"
                        height="14"
                      />
                    </button>
                  </div>
                  <CBtn
                    variant="ghost"
                    size="sm"
                    class="icon-btn--danger"
                    @click="revokeToken(task)"
                  >
                    {{ t('tasks.revokeToken') }}
                  </CBtn>
                </div>

                <div class="task-meta">
                  <span>{{ formatRelative(task.created_at) }}</span>
                  <span v-if="task.closed_at">· {{ t('tasks.closedAt', { date: formatDate(task.closed_at) }) }}</span>
                </div>
              </div>

              <!-- Actions -->
              <div class="task-actions">
                <button
                  v-if="!task.has_vendor_token && task.status !== 'closed' && task.status !== 'cancelled'"
                  class="icon-btn"
                  :title="t('tasks.generateToken')"
                  :disabled="generatingToken === task.id"
                  @click="generateToken(task)"
                >
                  <CIconWrench
                    width="16"
                    height="16"
                  />
                </button>
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

    <!-- Create task modal -->
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
        <CFormFeedback
          :status="taskFeedback.status.value"
          :message="taskFeedback.message.value"
        />
      </form>
      <template #footer>
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
  </div>
</template>

<style scoped>
.task-list { display: flex; flex-direction: column; gap: 10px; }

.task-card { overflow: hidden; }
.task-row { display: flex; align-items: stretch; gap: 0; }

.prio-bar { width: 4px; flex-shrink: 0; }

.task-body { flex: 1; padding: 14px 16px; min-width: 0; }
.task-head { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--space-3); margin-bottom: 6px; }
.task-badges { display: flex; gap: 6px; flex-shrink: 0; }
.task-desc { color: var(--color-on-surface-variant); margin-bottom: var(--space-2); }
.task-meta { font-size: var(--text-2xs); color: var(--color-on-surface-variant); }

.vendor-token { display: flex; flex-direction: column; gap: 4px; margin: 6px 0; padding: 8px; background: var(--color-surface-container); border-radius: var(--radius-sm); }
.token-url-row { display: flex; align-items: center; gap: 6px; }
.token-url { font-size: var(--text-2xs); word-break: break-all; flex: 1; }

.task-actions { display: flex; flex-direction: column; justify-content: center; padding: 0 8px; gap: 4px; }

.empty-state { padding: var(--space-8); text-align: center; font-size: var(--text-md); color: var(--color-on-surface-variant); }

.form-grid { display: flex; flex-direction: column; gap: var(--space-4); }
.form-field { display: flex; flex-direction: column; gap: var(--space-1); }
.form-label { font-size: var(--text-sm); font-weight: var(--fw-medium); color: var(--color-on-surface-variant); }
.form-input { padding: var(--space-2) var(--space-3); border: 1px solid var(--color-outline); border-radius: var(--radius-md); font-size: var(--text-sm); background: var(--color-surface); color: var(--color-on-surface); }
.form-input:focus { outline: none; border-color: var(--color-primary); }
</style>

