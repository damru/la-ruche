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
import { buildingApi, reportsApi, unitsApi } from '@/api'
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

// ── Create report modal ───────────────────────────────────────────
const showCreateModal = ref(false)
const createFeedback = useFormFeedback()
const saving = ref(false)
const createForm = ref({ title: '', description: '', unit_id: '' })
const { schedule: scheduleCreateClose, forceExpire: forceCreateClose } = useAutoClose()
watch(showCreateModal, (open) => { if (!open) forceCreateClose() })

function openCreateModal() {
  createForm.value = { title: '', description: '', unit_id: '' }
  createFeedback.reset()
  showCreateModal.value = true
}

async function submitCreate() {
  if (createFeedback.status.value === 'success') { forceCreateClose(); return }
  if (!createForm.value.title.trim()) return
  saving.value = true
  createFeedback.reset()
  try {
    await reportsApi.create({
      title: createForm.value.title.trim(),
      description: createForm.value.description.trim() || undefined,
      unit_id: createForm.value.unit_id || undefined,
    })
    createFeedback.setSuccess(t('reports.createSuccess'))
    scheduleCreateClose(() => { showCreateModal.value = false; reload() })
  } catch (err) {
    createFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    saving.value = false
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
            </div>

            <!-- Card actions (manager / board only) -->
            <div
              v-if="canManage"
              class="report-card-actions"
            >
              <CBtn
                v-if="r.status !== 'closed'"
                variant="ghost"
                size="sm"
                @click="openStatusModal(r)"
              >
                <CIconEdit />
                {{ t('reports.editStatus') }}
              </CBtn>
              <CBtn
                v-if="r.status !== 'closed'"
                variant="ghost"
                size="sm"
                @click="openConvertModal(r)"
              >
                <CIconCheckCircle />
                {{ t('reports.convertToTask') }}
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
      :closable="!saving"
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
          :loading="saving"
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
            v-for="s in (['open', 'in_progress', 'closed'] as ReportStatus[])"
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
  </div>
</template>

<style scoped>
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
.report-card-actions { display: flex; gap: var(--space-2); flex-wrap: wrap; padding-top: var(--space-1); }
</style>
