<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useRole } from '@/composables/useRole'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CModal from '@/components/molecules/CModal.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormCheckbox from '@/components/atoms/CFormCheckbox.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CProgressBar from '@/components/atoms/CProgressBar.vue'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CHeroCard from '@/components/molecules/CHeroCard.vue'
import CFlexGroup from '@/components/organisms/CFlexGroup.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconCheckCircle from '@/components/atoms/CIconCheckCircle.svg'
import CIconCurrency from '@/components/atoms/CIconCurrency.svg'
import CIconCheck from '@/components/atoms/CIconCheck.svg'
import CIconWarn from '@/components/atoms/CIconWarn.svg'
import CIconWrench from '@/components/atoms/CIconWrench.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconMegaphone from '@/components/atoms/CIconMegaphone.svg'
import CIconPieChart from '@/components/atoms/CIconPieChart.svg'
import CIconLock from '@/components/atoms/CIconLock.svg'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import { budgetApi, allocationKeysApi, buildingApi } from '@/api'
import type { Budget, BudgetLine, LevyCall, WorksFund, AllocationKey, LevyStatus, BuildingStats } from '@/types/domain'

const { t } = useI18n()
const { formatCurrency, formatDate } = useFormatters()
const { hasAnyRole } = useRole()

const canManage = computed(() => hasAnyRole(['manager']))
const canEditSelectedBudget = computed(() => canManage.value && selectedBudget.value?.status !== 'closed')
const DEFAULT_WORKS_FUND_RATE = 0.05
const PIE_CHART_COLORS = [
  'var(--color-primary)',
  'var(--color-on-secondary-container)',
  'var(--color-warning)',
  'var(--color-success)',
  'var(--color-error)',
  'var(--color-on-surface-variant)',
  'var(--color-secondary-container)',
  'var(--color-success-container)',
]

// ── State ─────────────────────────────────────────────────────────

const buildingName = ref('')
const buildingStats = ref<BuildingStats | null>(null)
const budgets = ref<Budget[]>([])
const selectedBudgetId = ref<string | null>(null)
const budgetLines = ref<BudgetLine[]>([])
const levies = ref<LevyCall[]>([])
const worksFund = ref<WorksFund | null>(null)
const allocationKeys = ref<AllocationKey[]>([])

// ── Derived ───────────────────────────────────────────────────────

const selectedBudget = computed(() =>
  budgets.value.find(b => b.id === selectedBudgetId.value) ?? null
)

const overdueLevies = computed(() => levies.value.filter(l => l.status === 'overdue'))
const overdueTotal = computed(() => overdueLevies.value.reduce((s, l) => s + l.amount, 0))

const worksEngagedPct = computed(() => {
  if (!worksFund.value?.levied_amount) return 0
  const engaged = worksFund.value.levied_amount - worksFund.value.available_amount
  return Math.min(100, Math.round((engaged / worksFund.value.levied_amount) * 100))
})

const keyOptions = computed(() =>
  allocationKeys.value.map(k => ({ value: k.id, label: k.name }))
)

const selectedTotalForecast = computed(() =>
  Number(selectedBudget.value?.total_forecast ?? 0)
)
const selectedTotalActual = computed(() =>
  Number(selectedBudget.value?.total_actual ?? 0)
)

const levyCallCount = computed(() => levies.value.length)
const leviedUnitsCount = computed(() =>
  new Set(levies.value.map(levy => levy.unit_id)).size
)
const levyPaidCount = computed(() =>
  levies.value.filter(l => l.status === 'paid').length
)
const totalLeviedAmount = computed(() =>
  levies.value.reduce((sum, levy) => sum + levy.amount, 0)
)
const levyCollectionRate = computed(() =>
  levyCallCount.value > 0 ? Math.round((levyPaidCount.value / levyCallCount.value) * 100) : 0
)
const worksFundContributionTarget = computed(() => selectedTotalForecast.value * DEFAULT_WORKS_FUND_RATE)
const generatePreviewLots = computed(() =>
  buildingStats.value?.nb_lots ?? leviedUnitsCount.value
)
const generatePreviewTotal = computed(() => {
  const totalTarget = selectedTotalForecast.value + (generateForm.value.include_works_fund ? worksFundContributionTarget.value : 0)
  return Math.max(totalTarget - totalLeviedAmount.value, 0)
})
const budgetBreakdownItems = computed(() => {
  const total = budgetLines.value.reduce((sum, line) => sum + Number(line.budgeted_amount ?? 0), 0)

  return budgetLines.value
    .filter(line => Number(line.budgeted_amount ?? 0) > 0)
    .map((line, index) => {
      const amount = Number(line.budgeted_amount ?? 0)
      const sharePct = total > 0 ? (amount / total) * 100 : 0

      return {
        id: line.id,
        label: line.label,
        amount,
        sharePct,
        color: PIE_CHART_COLORS[index % PIE_CHART_COLORS.length],
      }
    })
})
const budgetBreakdownStyle = computed(() => {
  if (budgetBreakdownItems.value.length === 0) {
    return {}
  }

  let start = 0
  const segments = budgetBreakdownItems.value.map((item) => {
    const end = start + (item.sharePct / 100) * 360
    const segment = `${item.color} ${start}deg ${end}deg`
    start = end
    return segment
  })

  return {
    background: `conic-gradient(${segments.join(', ')})`,
  }
})

const legalCardLinks = computed(() => [
  {
    label: t('budget.legalCard.article35LinkLabel'),
    href: t('budget.legalCard.article35LinkHref'),
  },
  {
    label: t('budget.legalCard.article37LinkLabel'),
    href: t('budget.legalCard.article37LinkHref'),
  },
  {
    label: t('budget.legalCard.article14_2_1LinkLabel'),
    href: t('budget.legalCard.article14_2_1LinkHref'),
  },
])

// ── Loaders ───────────────────────────────────────────────────────

async function loadMeta() {
  const [statsRes, keysRes] = await Promise.all([
    buildingApi.stats(),
    allocationKeysApi.list(),
  ])
  buildingStats.value = statsRes.data
  buildingName.value = statsRes.data.name
  allocationKeys.value = keysRes.data
}

async function loadBudgets() {
  const res = await budgetApi.list()
  budgets.value = res.data
  if (res.data.length > 0 && !selectedBudgetId.value) {
    selectedBudgetId.value = res.data[0].id
  }
}

async function loadBudgetData(budgetId: string) {
  const [budgetRes, linesRes, leviesRes] = await Promise.all([
    budgetApi.get(budgetId),
    budgetApi.listLines(budgetId),
    budgetApi.listLevies(budgetId),
  ])
  const refreshed = budgetRes.data
  budgets.value = budgets.value.map(b => (b.id === refreshed.id ? refreshed : b))
  // Enrich lines with key names
  budgetLines.value = linesRes.data.map(l => ({
    ...l,
    budgeted_amount: Number(l.budgeted_amount),
    actual_amount: l.actual_amount != null ? Number(l.actual_amount) : undefined,
    key_name: allocationKeys.value.find(k => k.id === l.key_id)?.name,
  }))
  levies.value = (leviesRes.data as LevyCall[]).map(levy => ({
    ...levy,
    amount: Number(levy.amount),
    call_label:
      !levy.call_label || levy.call_label === 'Levy call'
        ? t('budget.defaultLevyLabel')
        : levy.call_label,
  }))

  // Load works fund for selected year
  if (selectedBudget.value) {
    try {
      const wfRes = await budgetApi.getWorksFund(selectedBudget.value.fiscal_year)
      worksFund.value = {
        ...wfRes.data,
        levied_amount: Number(wfRes.data.levied_amount),
        available_amount: Number(wfRes.data.available_amount),
      }
    } catch {
      worksFund.value = null
    }
  }
}

const { loading, error, reload } = usePageLoader(async () => {
  await loadMeta()
  await loadBudgets()
  if (selectedBudgetId.value) {
    await loadBudgetData(selectedBudgetId.value)
  }
})

onMounted(reload)

watch(selectedBudgetId, async (id) => {
  if (id) await loadBudgetData(id)
})

// ── Create budget modal (S01) ─────────────────────────────────────

const showCreateBudgetModal = ref(false)
const createBudgetFeedback = useFormFeedback()
const savingBudget = ref(false)
const createBudgetForm = ref({ fiscal_year: new Date().getFullYear() + 1 })
const { forceExpire: forceCreateBudgetClose } = useAutoClose()
watch(showCreateBudgetModal, open => { if (!open) forceCreateBudgetClose() })

function openCreateBudgetModal() {
  createBudgetFeedback.reset()
  createBudgetForm.value = { fiscal_year: new Date().getFullYear() + 1 }
  showCreateBudgetModal.value = true
}

async function saveBudget() {
  savingBudget.value = true
  createBudgetFeedback.reset()
  try {
    const res = await budgetApi.create({ fiscal_year: Number(createBudgetForm.value.fiscal_year) })
    await loadBudgets()
    selectedBudgetId.value = res.data.id
    showCreateBudgetModal.value = false
  } catch (err) {
    createBudgetFeedback.setError(toErrorMessage(err))
  } finally {
    savingBudget.value = false
  }
}

// ── Create line modal (S01) ───────────────────────────────────────

const showCreateLineModal = ref(false)
const createLineFeedback = useFormFeedback()
const savingLine = ref(false)
const createLineForm = ref({ label: '', key_id: '', budgeted_amount: '' })
const { forceExpire: forceCreateLineClose } = useAutoClose()
watch(showCreateLineModal, open => { if (!open) forceCreateLineClose() })

function openCreateLineModal() {
  createLineFeedback.reset()
  createLineForm.value = { label: '', key_id: keyOptions.value[0]?.value ?? '', budgeted_amount: '' }
  showCreateLineModal.value = true
}

async function saveLine() {
  if (!selectedBudgetId.value) return
  const amount = parseFloat(createLineForm.value.budgeted_amount)
  if (!createLineForm.value.label.trim() || !createLineForm.value.key_id || isNaN(amount) || amount <= 0) {
    createLineFeedback.setError(t('common.required'))
    return
  }
  savingLine.value = true
  createLineFeedback.reset()
  try {
    await budgetApi.createLine(selectedBudgetId.value, {
      label: createLineForm.value.label.trim(),
      key_id: createLineForm.value.key_id,
      budgeted_amount: amount,
    })
    await loadBudgetData(selectedBudgetId.value)
    showCreateLineModal.value = false
  } catch (err) {
    createLineFeedback.setError(toErrorMessage(err))
  } finally {
    savingLine.value = false
  }
}

// ── Delete line (S01) ─────────────────────────────────────────────

const lineToDelete = ref<BudgetLine | null>(null)
const deletingLine = ref(false)
const deleteLineFeedback = useFormFeedback()

async function confirmDeleteLine() {
  if (!lineToDelete.value || !selectedBudgetId.value) return
  deletingLine.value = true
  deleteLineFeedback.reset()
  try {
    await budgetApi.deleteLine(selectedBudgetId.value, lineToDelete.value.id)
    await loadBudgetData(selectedBudgetId.value)
    lineToDelete.value = null
  } catch (err) {
    deleteLineFeedback.setError(toErrorMessage(err))
  } finally {
    deletingLine.value = false
  }
}

// ── Generate levies modal (S02) ───────────────────────────────────

const showGenerateModal = ref(false)
const showBreakdownModal = ref(false)
const generateFeedback = useFormFeedback()
const generatingLevies = ref(false)
const generateForm = ref({ due_date: '', include_works_fund: true, call_label: '' })
const { forceExpire: forceGenerateClose } = useAutoClose()
watch(showGenerateModal, open => { if (!open) forceGenerateClose() })

function openGenerateModal() {
  generateFeedback.reset()
  // Default due date = 1st day of next quarter
  const now = new Date()
  const nextQ = new Date(now.getFullYear(), Math.floor(now.getMonth() / 3) * 3 + 3, 1)
  generateForm.value = {
    due_date: nextQ.toISOString().slice(0, 10),
    include_works_fund: true,
    call_label: '',
  }
  showGenerateModal.value = true
}

async function generateLevies() {
  if (!selectedBudgetId.value || !generateForm.value.due_date) return
  generatingLevies.value = true
  generateFeedback.reset()
  try {
    await budgetApi.generateLevies(selectedBudgetId.value, {
      due_date: generateForm.value.due_date,
      include_works_fund: generateForm.value.include_works_fund,
      call_label: generateForm.value.call_label.trim() || undefined,
    })
    await loadBudgetData(selectedBudgetId.value)
    showGenerateModal.value = false
  } catch (err) {
    generateFeedback.setError(toErrorMessage(err))
  } finally {
    generatingLevies.value = false
  }
}

// ── Mark paid (S02) ───────────────────────────────────────────────

const markingPaidId = ref<string | null>(null)

async function markPaid(levy: LevyCall) {
  if (markingPaidId.value) return
  markingPaidId.value = levy.id
  try {
    await budgetApi.markPaid(levy.id)
    if (selectedBudgetId.value) await loadBudgetData(selectedBudgetId.value)
  } catch {
    // silently handle — row stays as-is
  } finally {
    markingPaidId.value = null
  }
}

// ── Close exercise modal ───────────────────────────────────────────

const showCloseExerciseModal = ref(false)
const closingExercise = ref(false)
const closeExerciseFeedback = useFormFeedback()

function openCloseExerciseModal() {
  closeExerciseFeedback.reset()
  showCloseExerciseModal.value = true
}

async function closeExercise() {
  if (!selectedBudgetId.value) return
  closingExercise.value = true
  closeExerciseFeedback.reset()
  try {
    await budgetApi.close(selectedBudgetId.value)
    await loadBudgetData(selectedBudgetId.value)
    showCloseExerciseModal.value = false
  } catch (err) {
    closeExerciseFeedback.setError(toErrorMessage(err))
  } finally {
    closingExercise.value = false
  }
}

// ── Helpers ───────────────────────────────────────────────────────

function linePct(line: BudgetLine): number {
  if (!line.actual_amount || !line.budgeted_amount) return 0
  return Math.min(100, Math.round((line.actual_amount / line.budgeted_amount) * 100))
}

function lineVariant(line: BudgetLine): 'primary' | 'warning' | 'success' {
  const pct = linePct(line)
  if (pct >= 100) return 'success'
  if (pct >= 85) return 'warning'
  return 'primary'
}

function levyStatusVariant(s: LevyStatus): string {
  if (s === 'paid') return 'tone--valid'
  if (s === 'issued') return 'tone--warn'
  if (s === 'overdue') return 'tone--err'
  return 'tone--disabled'
}

function initials(name: string): string {
  return name.split(' ').map(w => w[0]).join('').toUpperCase().slice(0, 2)
}

function formatSharePercentage(value: number): string {
  return new Intl.NumberFormat('fr-FR', {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  }).format(value)
}

// Works-fund legal threshold: alert if available < 5% of forecast (S03/S10)
const worksFundBelowThreshold = computed(() => {
  if (!worksFund.value || !selectedTotalForecast.value) return false
  const minRequired = selectedTotalForecast.value * 0.05
  return worksFund.value.available_amount < minRequired
})
</script>

<template>
  <div class="budget-page view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight aria-hidden="true" />
          <span>{{ t('budget.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canEditSelectedBudget && selectedBudgetId"
          variant="primary"
          size="sm"
          @click="openCloseExerciseModal"
        >
          <CIconLock aria-hidden="true" />
          <span class="btn-label">{{ t('budget.closeExercise') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container">
      <CPageLoader v-if="loading" />
      <CPageError
        v-else-if="error"
        :message="error"
        @retry="reload"
      />

      <template v-else>
        <!-- Page header -->
        <div class="page-header">
          <CEyebrow>{{ t('page.eyebrows.budget') }}</CEyebrow>
          <h1 class="page-title">
            {{ selectedBudget ? t('budget.fiscalYear', { year: selectedBudget.fiscal_year }) : t('budget.title') }}
          </h1>
          <CFlexGroup
            v-if="selectedBudget"
            direction="row"
            class="page-sub page-sub--metrics"
          >
            <CMetaItem variant="sub">
              {{ buildingName }}
            </CMetaItem>
            <CMetaItem variant="sub">
              {{ selectedBudget.status === 'approved' ? t('budget.statuses.approvedAG') : t(`budget.statuses.${selectedBudget.status}`) }}
            </CMetaItem>
          </CFlexGroup>
        </div>

        <!-- Year tabs -->
        <div class="tabs">
          <button
            v-for="b in budgets"
            :key="b.id"
            :class="['tab', { 'tab--active': selectedBudgetId === b.id }]"
            @click="selectedBudgetId = b.id"
          >
            {{ b.fiscal_year }}
          </button>
          <button
            v-if="canManage"
            class="tab tab--new"
            @click="openCreateBudgetModal"
          >
            + {{ t('budget.newFiscalYear') }}
          </button>
        </div>

        <!-- KPI row -->
        <div class="kpi-grid">
          <CHeroCard
            :label="t('budget.budgetForecastLabel')"
            :value="formatCurrency(selectedTotalForecast)"
            tone="default"
          >
            <template #icon>
              <CIconCurrency aria-hidden="true" />
            </template>
            <template #subtitle>
              {{ t('budget.budgetItems', budgetLines.length) }}
            </template>
          </CHeroCard>

          <CHeroCard
            :label="t('budget.realizedToDate')"
            :value="formatCurrency(selectedTotalActual)"
            tone="accent"
          >
            <template #icon>
              <div class="hero-card__icon-on-accent">
                <CIconCheck
                  stroke="white"
                  aria-hidden="true"
                />
              </div>
            </template>
            <template #subtitle>
              {{ t('budget.ofBudget', {
                pct: selectedTotalForecast ? Math.round((selectedTotalActual / selectedTotalForecast) * 100) : 0,
              }) }}
            </template>
          </CHeroCard>

          <CHeroCard
            :label="t('budget.overdueCallsLabel')"
            :value="overdueLevies.length"
            tone="err"
            :emphasize-value="true"
          >
            <template #icon>
              <CIconWarn aria-hidden="true" />
            </template>
            <template #subtitle>
              {{ t('budget.pendingAmount', { amount: formatCurrency(overdueTotal) }) }}
            </template>
          </CHeroCard>

          <CHeroCard
            :label="t('budget.worksFund')"
            :value="worksFund ? formatCurrency(worksFund.levied_amount) : '—'"
            tone="warn"
            :emphasize-value="!!worksFund"
          >
            <template #icon>
              <CIconWrench aria-hidden="true" />
            </template>
            <template
              v-if="worksFund"
              #subtitle
            >
              {{ t('budget.available', { amount: formatCurrency(worksFund.available_amount) }) }}
            </template>
          </CHeroCard>
        </div>

        <!-- Two-col layout -->
        <div class="two-col">
          <!-- LEFT column -->
          <div class="left-col">
            <!-- Budget lines consumption (S04 — arrêté de comptes) -->
            <div class="section-card">
              <div class="section-head">
                <div>
                  <h2 class="section-title">
                    {{ t('budget.consumptionByLine') }}
                  </h2>
                  <p class="section-sub">
                    {{ t('budget.forecastVsActual') }}
                  </p>
                </div>
                <div class="section-head__actions">
                  <CBtn
                    v-if="budgetBreakdownItems.length > 0"
                    variant="ghost"
                    size="sm"
                    @click="showBreakdownModal = true"
                  >
                    <CIconPieChart aria-hidden="true" />
                    {{ t('budget.breakdown') }}
                  </CBtn>
                  <CBtn
                    v-if="canEditSelectedBudget"
                    variant="ghost"
                    size="sm"
                    @click="openCreateLineModal"
                  >
                    <CIconPlus aria-hidden="true" />
                    {{ t('budget.newLine') }}
                  </CBtn>
                </div>
              </div>

              <div
                v-if="budgetLines.length === 0"
                class="empty-state"
              >
                <p>{{ t('common.noData') }}</p>
                <CBtn
                  v-if="canEditSelectedBudget"
                  variant="ghost"
                  size="sm"
                  @click="openCreateLineModal"
                >
                  <CIconPlus aria-hidden="true" />
                  {{ t('budget.addLine') }}
                </CBtn>
              </div>

              <div
                v-else
                class="prog-list"
              >
                <div
                  v-for="line in budgetLines"
                  :key="line.id"
                  class="prog-item"
                >
                  <div class="prog-item__header">
                    <span class="prog-item__label">{{ line.label }}</span>
                    <div class="prog-item__meta">
                      <span
                        v-if="line.key_name"
                        class="chip tone--default"
                      >{{ line.key_name }}</span>
                      <CBtn
                        v-if="canEditSelectedBudget"
                        variant="ghost"
                        size="sm"
                        class="prog-item__delete"
                        :aria-label="t('common.delete')"
                        @click="lineToDelete = line"
                      >
                        <CIconTrash aria-hidden="true" />
                      </CBtn>
                    </div>
                  </div>
                  <CProgressBar
                    :value="linePct(line)"
                    :variant="lineVariant(line)"
                    :show-value="true"
                  />
                  <div class="prog-item__foot">
                    <span>{{ t('budget.realizedLabel') }} : <b>{{ formatCurrency(line.actual_amount ?? 0) }}</b></span>
                    <span>{{ t('budget.plannedLabel') }} : <b>{{ formatCurrency(line.budgeted_amount) }}</b></span>
                  </div>
                </div>
              </div>

              <!-- Total row -->
              <div
                v-if="budgetLines.length > 0"
                class="prog-total"
              >
                <span>{{ t('common.total') }}</span>
                <span class="prog-total__values">
                  {{ formatCurrency(selectedTotalActual) }} / {{ formatCurrency(selectedTotalForecast) }}
                </span>
              </div>
            </div>

            <!-- Levy calls table (S02) -->
            <div class="section-card">
              <div class="section-head">
                <div>
                  <h2 class="section-title">
                    {{ t('budget.levies') }}
                  </h2>
                  <CFlexGroup
                    direction="row"
                    class="section-sub section-sub-metrics"
                  >
                    <CMetaItem variant="sub">
                      {{ t('budget.levyCallsCount', { count: levyCallCount }) }}
                    </CMetaItem>
                    <CMetaItem variant="sub">
                      {{ t('budget.collectionRatePct', { pct: levyCollectionRate }) }}
                    </CMetaItem>
                  </CFlexGroup>
                </div>
                <div class="section-head__actions">
                  <CBtn
                    v-if="canEditSelectedBudget && selectedBudgetId"
                    variant="ghost"
                    size="sm"
                    @click="openGenerateModal"
                  >
                    <CIconMegaphone aria-hidden="true" />
                    {{ t('budget.generateLevies') }}
                  </CBtn>
                </div>
              </div>

              <div
                v-if="levies.length === 0"
                class="empty-state"
              >
                <p>{{ t('common.noData') }}</p>
                <CBtn
                  v-if="canManage && selectedBudgetId"
                  variant="ghost"
                  size="sm"
                  @click="openGenerateModal"
                >
                  <CIconMegaphone aria-hidden="true" />
                  {{ t('budget.generateLevies') }}
                </CBtn>
              </div>

              <div
                v-else
                class="table-wrap"
              >
                <table class="data-table">
                  <thead>
                    <tr>
                      <th>{{ t('common.lot') }}</th>
                      <th>{{ t('common.coOwner') }}</th>
                      <th class="col-right">
                        {{ t('common.amount') }}
                      </th>
                      <th class="col-right">
                        {{ t('common.dueDate') }}
                      </th>
                      <th>{{ t('budget.levyCallLabel') }}</th>
                      <th>{{ t('common.status') }}</th>
                      <th
                        v-if="canEditSelectedBudget"
                        class="col-shrink"
                      ></th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr
                      v-for="levy in levies"
                      :key="levy.id"
                    >
                      <td class="cell-mono">
                        {{ levy.unit_number ?? '—' }}
                      </td>
                      <td>{{ levy.owner_name ?? '—' }}</td>
                      <td
                        class="cell-numeric"
                        :class="{ 'tone--err': levy.status === 'overdue' }"
                      >
                        {{ formatCurrency(levy.amount) }}
                      </td>
                      <td class="col-right cell-muted">
                        {{ levy.due_date ? formatDate(levy.due_date) : '—' }}
                      </td>
                      <td class="cell-muted">
                        {{ levy.call_label || t('budget.defaultLevyLabel') }}
                      </td>
                      <td>
                        <span :class="['chip', levyStatusVariant(levy.status as LevyStatus)]">
                          <span
                            class="chip-dot"
                            aria-hidden="true"
                          ></span>
                          {{ t(`budget.levyStatuses.${levy.status}`) }}
                        </span>
                      </td>
                      <td v-if="canEditSelectedBudget">
                        <CBtn
                          v-if="levy.status !== 'paid'"
                          variant="ghost"
                          size="sm"
                          :loading="markingPaidId === levy.id"
                          @click="markPaid(levy)"
                        >
                          <CIconCheckCircle aria-hidden="true" />
                          {{ t('budget.markPaid') }}
                        </CBtn>
                      </td>
                    </tr>
                  </tbody>
                  <tfoot>
                    <tr class="row-total">
                      <td colspan="2">
                        {{ t('common.total') }}
                      </td>
                      <td class="cell-numeric">
                        {{ formatCurrency(levies.reduce((s, l) => s + l.amount, 0)) }}
                      </td>
                      <td
                        :colspan="canEditSelectedBudget ? 4 : 3"
                      ></td>
                    </tr>
                  </tfoot>
                </table>
              </div>
            </div>
          </div>

          <!-- RIGHT sidebar -->
          <div class="right-col">
            <!-- Works fund card (S03) -->
            <div class="ft-card">
              <div class="ft-header">
                <h3 class="ft-title">
                  {{ t('budget.worksFund') }}
                </h3>
                <span class="ft-badge tone--default">Art. 14-2 ALUR</span>
              </div>

              <template v-if="worksFund">
                <div class="ft-amount">
                  {{ formatCurrency(worksFund.levied_amount) }}
                </div>
                <CFlexGroup
                  direction="row"
                  class="ft-sub ft-sub--metrics"
                >
                  <CMetaItem variant="sub">
                    {{ t('budget.separateBankAccountLabel') }}
                  </CMetaItem>
                  <CMetaItem variant="sub">
                    {{ t('budget.fiscalYearShort', { year: selectedBudget?.fiscal_year }) }}
                  </CMetaItem>
                </CFlexGroup>
                <div class="ft-prog">
                  <div class="ft-prog-head">
                    <span>{{ t('budget.engaged') }}</span>
                    <span class="ft-prog-pct tone--warn">{{ worksEngagedPct }}%</span>
                  </div>
                  <CProgressBar
                    :value="worksEngagedPct"
                    variant="warning"
                    :show-value="false"
                  />
                  <div class="ft-prog-foot">
                    <span>{{ t('budget.engagedAmount', { amount: formatCurrency(worksFund.levied_amount - worksFund.available_amount) }) }}</span>
                    <span
                      class="ft-prog-available"
                      :class="{ 'tone--err': worksFundBelowThreshold }"
                    >
                      {{ t('budget.remaining', { amount: formatCurrency(worksFund.available_amount) }) }}
                    </span>
                  </div>
                </div>
              </template>
              <template v-else>
                <p class="ft-sub">
                  {{ t('common.noData') }}
                </p>
              </template>
            </div>

            <!-- Overdue payments panel (S02) -->
            <div
              v-if="overdueLevies.length > 0"
              class="mini-card"
            >
              <h3 class="mini-title">
                {{ t('budget.latePayments') }}
              </h3>
              <CFlexGroup
                direction="row"
                class="mini-sub mini-sub--metrics"
              >
                <CMetaItem variant="sub">
                  {{ t('budget.latePaymentsCount', { count: overdueLevies.length }) }}
                </CMetaItem>
                <CMetaItem variant="sub">
                  {{ t('budget.latePaymentsTotal', { amount: formatCurrency(overdueTotal) }) }}
                </CMetaItem>
              </CFlexGroup>
              <div
                v-for="levy in overdueLevies"
                :key="levy.id"
                class="overdue-item"
              >
                <div class="overdue-avatar tone--err">
                  {{ initials(levy.owner_name ?? '?') }}
                </div>
                <div class="overdue-info">
                  <div class="overdue-name">
                    {{ levy.owner_name ?? '—' }}
                  </div>
                  <div class="overdue-lot">
                    {{ levy.unit_number ?? '—' }}
                  </div>
                </div>
                <div class="overdue-right">
                  <div class="overdue-amount">
                    {{ formatCurrency(levy.amount) }}
                  </div>
                  <div class="overdue-date">
                    {{ levy.due_date ? formatDate(levy.due_date) : '—' }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <CLegalCard>
          <CLegalNotice :links="legalCardLinks">
            <CLegalDescription
              :title="t('budget.legalCard.title')"
              :summary="t('budget.legalCard.summary')"
            />
            <CLegalDescription
              :title="t('budget.legalCard.worksFundTitle')"
              :summary="t('budget.legalCard.worksFundSummary')"
            />
          </CLegalNotice>
        </CLegalCard>
      </template>
    </div>

    <!-- ── Modal: Nouvel exercice (S01) ──────────────────────────── -->
    <CModal
      :open="showCreateBudgetModal"
      :title="t('budget.newFiscalYear')"
      @update:open="showCreateBudgetModal = $event"
    >
      <CFormInput
        v-model="createBudgetForm.fiscal_year"
        :label="t('budget.fiscalYearLabel')"
        type="number"
        :min="2020"
        :max="2099"
        required
      />
      <template #footer>
        <CFormFeedback
          :status="createBudgetFeedback.status.value"
          :message="createBudgetFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showCreateBudgetModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="savingBudget"
          @click="saveBudget"
        >
          {{ t('common.create') }}
        </CBtn>
      </template>
    </CModal>

    <!-- ── Modal: Nouvelle ligne (S01) ───────────────────────────── -->
    <CModal
      :open="showCreateLineModal"
      :title="t('budget.addLine')"
      @update:open="showCreateLineModal = $event"
    >
      <CFormInput
        v-model="createLineForm.label"
        :label="t('budget.lineLabel')"
        required
      />
      <CFormSelect
        v-model="createLineForm.key_id"
        :label="t('budget.lineKey')"
        :options="keyOptions"
        required
      />
      <CFormInput
        v-model="createLineForm.budgeted_amount"
        :label="t('budget.lineBudgetedAmount')"
        type="number"
        :min="0"
        step="0.01"
        required
      />
      <template #footer>
        <CFormFeedback
          :status="createLineFeedback.status.value"
          :message="createLineFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showCreateLineModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="savingLine"
          @click="saveLine"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>

    <!-- ── Modal: Supprimer ligne (S01) ──────────────────────────── -->
    <CModal
      :open="!!lineToDelete"
      :title="t('common.confirmDelete')"
      @update:open="v => { if (!v) lineToDelete = null }"
    >
      <p>{{ lineToDelete?.label }}</p>
      <template #footer>
        <CFormFeedback
          :status="deleteLineFeedback.status.value"
          :message="deleteLineFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="lineToDelete = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="deletingLine"
          @click="confirmDeleteLine"
        >
          {{ t('common.delete') }}
        </CBtn>
      </template>
    </CModal>

    <!-- ── Modal: Générer les appels (S02) ───────────────────────── -->
    <CModal
      :open="showGenerateModal"
      :title="t('budget.generateLeviesTitle')"
      @update:open="showGenerateModal = $event"
    >
      <p class="modal-sub">
        {{ t('budget.generateLeviesSub') }}
      </p>
      <CFormInput
        v-model="generateForm.due_date"
        :label="t('common.dueDate')"
        type="date"
        required
      />
      <div class="field-group">
        <CFormCheckbox
          v-model="generateForm.include_works_fund"
          :label="t('budget.includeWorksFund')"
        />
        <p class="field-hint">
          {{ t('budget.worksFundLegal') }}
        </p>
      </div>
      <CFormInput
        v-model="generateForm.call_label"
        :label="t('budget.callLabel')"
        :placeholder="t('budget.callLabelPlaceholder')"
      />
      <div
        v-if="selectedTotalForecast > 0"
        class="generate-summary"
      >
        <strong>{{ t('budget.generateSummaryLabel') }}</strong>
        <div class="generate-summary__grid">
          <div class="generate-summary__metric">
            <span class="generate-summary__metric-label">{{ t('budget.generateSummaryLotsLabel') }}</span>
            <strong>{{ generatePreviewLots }}</strong>
          </div>
          <div class="generate-summary__metric">
            <span class="generate-summary__metric-label">{{ t('budget.generateSummaryAlreadyCalledLabel') }}</span>
            <strong>{{ formatCurrency(totalLeviedAmount) }}</strong>
          </div>
          <div class="generate-summary__metric">
            <span class="generate-summary__metric-label">{{ t('budget.generateSummaryRemainingLabel') }}</span>
            <strong>{{ formatCurrency(generatePreviewTotal) }}</strong>
          </div>
        </div>
        <p
          v-if="generatePreviewTotal === 0"
          class="generate-summary__hint"
        >
          {{ t('budget.generateSummaryNothingToGenerate') }}
        </p>
      </div>
      <template #footer>
        <CFormFeedback
          :status="generateFeedback.status.value"
          :message="generateFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showGenerateModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="generatingLevies"
          @click="generateLevies"
        >
          <CIconMegaphone aria-hidden="true" />
          {{ t('budget.generate') }}
        </CBtn>
      </template>
    </CModal>

    <CModal
      :open="showBreakdownModal"
      :title="t('budget.breakdownTitle')"
      size="lg"
      @update:open="showBreakdownModal = $event"
    >
      <div class="breakdown-modal">
        <div class="breakdown-modal__chart-wrap">
          <div
            class="breakdown-modal__chart"
            :style="budgetBreakdownStyle"
            role="img"
            :aria-label="t('budget.breakdownAriaLabel')"
          ></div>
        </div>

        <div class="breakdown-modal__legend">
          <p class="breakdown-modal__intro">
            {{ t('budget.breakdownIntro') }}
          </p>
          <div
            v-for="item in budgetBreakdownItems"
            :key="item.id"
            class="breakdown-modal__legend-item"
          >
            <span
              class="breakdown-modal__swatch"
              :style="{ background: item.color }"
              aria-hidden="true"
            ></span>
            <div class="breakdown-modal__legend-copy">
              <strong>{{ item.label }}</strong>
              <CFlexGroup
                direction="row"
                class="breakdown-modal__legend-meta"
              >
                <CMetaItem variant="sub">
                  {{ formatCurrency(item.amount) }}
                </CMetaItem>
                <CMetaItem variant="sub">
                  {{ t('budget.breakdownShare', { pct: formatSharePercentage(item.sharePct) }) }}
                </CMetaItem>
              </CFlexGroup>
            </div>
          </div>
        </div>
      </div>
    </CModal>

    <CModal
      :open="showCloseExerciseModal"
      :title="t('budget.closeExerciseTitle')"
      @update:open="showCloseExerciseModal = $event"
    >
      <p class="modal-sub">
        {{ t('budget.closeExerciseConfirmation', { year: selectedBudget?.fiscal_year }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="closeExerciseFeedback.status.value"
          :message="closeExerciseFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showCloseExerciseModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="closingExercise"
          @click="closeExercise"
        >
          {{ t('budget.closeExerciseConfirm') }}
        </CBtn>
      </template>
    </CModal>
  </div>
</template>

<style scoped>
/* ── Layout ──────────────────────────────────────────────────────── */
.budget-page { display: flex; flex-direction: column; min-height: 100vh; }

.page-title {
  font-family: var(--font-display);
  font-size: 36px;
  font-weight: var(--fw-bold);
  font-variation-settings: 'opsz' 72;
  letter-spacing: -.03em;
  color: var(--color-on-surface);
  line-height: 1;
}
.page-sub {
  font-size: 14px;
  color: var(--color-on-surface-variant);
  margin-top: 6px;
  margin-bottom: 32px;
}
.page-sub--metrics {
  flex-wrap: wrap;
}

/* ── Tabs ────────────────────────────────────────────────────────── */
.tabs { margin-bottom: 32px; }
.tab--new { color: var(--color-primary); }

/* ── KPI ─────────────────────────────────────────────────────────── */
.kpi-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 32px;
}
.hero-card__icon-on-accent {
  color: var(--color-on-primary);
  display: inline-flex;
}

/* ── Two-col ─────────────────────────────────────────────────────── */
.two-col {
  display: grid;
  grid-template-columns: 1fr 340px;
  gap: 24px;
  align-items: start;
}
.left-col { display: flex; flex-direction: column; gap: 24px; }
.right-col { display: flex; flex-direction: column; gap: 20px; }

/* ── Section card ────────────────────────────────────────────────── */
.section-card {
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  overflow: hidden;
}
.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  gap: 12px;
}
.section-head__actions { display: inline-flex; align-items: center; gap: 8px; }
.section-title {
  font-family: var(--font-display);
  font-size: 17px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  letter-spacing: -.01em;
  color: var(--color-on-surface);
}
.section-sub { font-size: 12px; color: var(--color-on-surface-variant); margin-top: 2px; }
.section-sub-metrics {
  margin-top: 2px;
  color: var(--color-on-surface-variant);
  flex-wrap: wrap;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 24px;
  color: var(--color-on-surface-variant);
  font-size: 14px;
}

/* ── Progress bars ───────────────────────────────────────────────── */
.prog-list { padding: 0 24px 20px; display: flex; flex-direction: column; gap: 20px; }

.prog-item__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}
.prog-item__label { font-size: 13px; font-weight: var(--fw-medium); color: var(--color-on-surface); }
.prog-item__meta { display: flex; align-items: center; gap: 8px; }
.prog-item__delete { opacity: 0; transition: opacity var(--transition-fast); padding: 0 6px !important; }
.prog-item:hover .prog-item__delete { opacity: 1; }

.prog-item__foot {
  display: flex;
  justify-content: space-between;
  margin-top: 5px;
  font-size: 11px;
  color: var(--color-on-surface-variant);
}
.prog-item__foot b { color: var(--color-on-surface); font-weight: var(--fw-medium); font-family: var(--font-mono); }

.prog-total {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 24px;
  border-top: 1px solid var(--color-outline-variant);
  font-size: 13px;
  font-weight: var(--fw-semibold);
  color: var(--color-on-surface);
  background: var(--color-surface-container-lowest);
}
.prog-total__values { font-family: var(--font-mono); }

/* ── Chips ───────────────────────────────────────────────────────── */
.chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: var(--radius-full);
  font-size: 11px;
  font-weight: var(--fw-medium);
  background: var(--tone-bg, transparent);
  color: var(--tone-fg, inherit);
}
.chip-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: .65;
  flex-shrink: 0;
}

/* ── Table overrides for this view ───────────────────────────────── */
.table-wrap {
  overflow-x: auto;
  overflow-y: hidden;
  -webkit-overflow-scrolling: touch;
  scrollbar-gutter: stable both-edges;
}
.cell-mono { font-family: var(--font-mono); font-size: 12px; color: var(--color-primary); }
.cell-muted { color: var(--color-on-surface-variant); font-size: 12px; }

/* ── Works fund card ─────────────────────────────────────────────── */
.ft-card {
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  padding: 24px;
  box-shadow: var(--shadow-md);
}
.ft-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 16px; }
.ft-title {
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  color: var(--color-on-surface);
}
.ft-badge {
  font-size: 10px;
  font-weight: var(--fw-semibold);
  letter-spacing: .06em;
  text-transform: uppercase;
  padding: 4px 8px;
  border-radius: var(--radius-full);
  background: var(--tone-bg);
  color: var(--tone-fg);
}
.ft-amount {
  font-family: var(--font-display);
  font-size: 32px;
  font-weight: var(--fw-bold);
  font-variation-settings: 'opsz' 72;
  letter-spacing: -.03em;
  color: var(--color-on-surface);
  margin-bottom: 4px;
}
.ft-sub { font-size: 12px; color: var(--color-on-surface-variant); margin-bottom: 20px; }
.ft-sub--metrics { flex-wrap: wrap; }
.ft-prog { margin-bottom: 8px; }
.ft-prog-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
  font-size: 13px;
  font-weight: var(--fw-medium);
  color: var(--color-on-surface);
}
.ft-prog-pct { color: var(--color-warning, #e65100); font-weight: var(--fw-semibold); }
.ft-prog-foot { display: flex; justify-content: space-between; margin-top: 5px; font-size: 11px; color: var(--color-on-surface-variant); }
.ft-prog-available { color: var(--color-warning, #e65100); font-weight: var(--fw-medium); }
/* ── Mini card (overdue) ─────────────────────────────────────────── */
.mini-card {
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  padding: 20px;
  box-shadow: var(--shadow-md);
}
.mini-title {
  font-family: var(--font-display);
  font-size: 15px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  color: var(--color-on-surface);
  margin-bottom: 4px;
}
.mini-sub { font-size: 12px; color: var(--color-on-surface-variant); margin-bottom: 16px; }
.mini-sub--metrics { flex-wrap: wrap; }

.overdue-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 0;
  border-bottom: 1px solid rgba(194,199,207,.15);
}
.overdue-item:last-of-type { border-bottom: none; padding-bottom: 0; }
.overdue-avatar {
  width: 34px;
  height: 34px;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-display);
  font-size: 12px;
  font-weight: var(--fw-bold);
  flex-shrink: 0;
  background: var(--tone-bg);
  color: var(--tone-fg);
}
.overdue-info { flex: 1; min-width: 0; }
.overdue-name { font-size: 13px; font-weight: var(--fw-medium); color: var(--color-on-surface); }
.overdue-lot { font-size: 11px; color: var(--color-on-surface-variant); margin-top: 1px; }
.overdue-right { text-align: right; flex-shrink: 0; }
.overdue-amount { font-family: var(--font-mono); font-size: 13px; font-weight: var(--fw-medium); color: var(--color-error); }
.overdue-date { font-size: 11px; color: var(--color-error); margin-top: 1px; }

/* ── Modal ───────────────────────────────────────────────────────── */
.modal-sub { font-size: 13px; color: var(--color-on-surface-variant); margin-bottom: 16px; line-height: 1.5; }
.field-group { display: flex; flex-direction: column; gap: 4px; }
.field-hint { font-size: 11px; color: var(--color-on-surface-variant); }
.generate-summary {
  background: var(--color-secondary-container);
  border-radius: var(--radius-md);
  padding: 12px 14px;
  font-size: 13px;
  color: var(--color-on-secondary-container);
  margin-top: 8px;
}
.generate-summary__grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
  margin-top: 10px;
}
.generate-summary__metric {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.generate-summary__metric-label {
  font-size: 11px;
  color: color-mix(in srgb, var(--color-on-secondary-container) 78%, transparent);
}
.generate-summary__hint {
  margin-top: 10px;
  font-size: 12px;
}

.breakdown-modal {
  display: grid;
  grid-template-columns: minmax(220px, 280px) minmax(0, 1fr);
  gap: 24px;
  align-items: center;
}
.breakdown-modal__chart-wrap {
  display: flex;
  justify-content: center;
}
.breakdown-modal__chart {
  width: min(280px, var(--breakdown-chart-max-width, 56vw));
  aspect-ratio: 1;
  border-radius: 50%;
  box-shadow: inset 0 0 0 1px rgba(25, 28, 29, 0.05);
}
.breakdown-modal__legend {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.breakdown-modal__intro {
  font-size: 13px;
  color: var(--color-on-surface-variant);
  margin-bottom: 4px;
}
.breakdown-modal__legend-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}
.breakdown-modal__swatch {
  width: 12px;
  height: 12px;
  border-radius: 999px;
  margin-top: 4px;
  flex-shrink: 0;
}
.breakdown-modal__legend-copy {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.breakdown-modal__legend-copy strong {
  font-size: 13px;
  color: var(--color-on-surface);
}
.breakdown-modal__legend-meta {
  flex-wrap: wrap;
}
.breakdown-modal__legend-copy :deep(.c-meta-item) {
  font-size: 12px;
  color: var(--color-on-surface-variant);
}

@media (max-width: 1024px) {
  .two-col {
    grid-template-columns: 1fr;
  }

  .breakdown-modal {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .kpi-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .section-head {
    flex-wrap: wrap;
    align-items: flex-start;
  }

  .generate-summary__grid {
    grid-template-columns: 1fr;
  }
}
</style>
