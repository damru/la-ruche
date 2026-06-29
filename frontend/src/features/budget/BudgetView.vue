<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
import { useRole } from '@/composables/useRole'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CProgressBar from '@/components/atoms/CProgressBar.vue'
import CDataTable from '@/components/molecules/CDataTable.vue'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CTabs, { type Tab } from '@/components/atoms/CTabs.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CCard from '@/components/molecules/CCard.vue'
import CKpiCard from '@/components/molecules/CKpiCard.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'
import CMetaLine from '@/components/molecules/CMetaLine.vue'
import CSeparator from '@/components/atoms/CSeparator.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CFlexGroup from '@/components/organisms/CFlexGroup.vue'
import CreateLineModal from './CreateLineModal.vue'
import DeleteLineModal from './DeleteLineModal.vue'
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
import CreateBudgetModal from './CreateBudgetModal.vue'
import CloseExerciseModal from './CloseExerciseModal.vue'
import WorksFundModal from './WorksFundModal.vue'
import { budgetApi, allocationKeysApi, buildingApi } from '@/api'
import GenerateLeviesModal from './GenerateLeviesModal.vue'
import BreakdownModal from './BreakdownModal.vue'
import type { Budget, BudgetLine, LevyCall, WorksFund, AllocationKey, LevyStatus, BuildingStats } from '@/types/domain'
import type { Column } from '@/components/molecules/CDataTable.vue'

interface ConsolidatedLevyRow {
  id: string
  owner_name: string
  unit_numbers: string[]
  amount: number
  due_date: string
  call_label: string
  issued_at: string
  paid_at: string | null
  status: LevyStatus
  levy_call_id: string
  levy_ids: string[]
  payable_unit_ids: string[]
}

type ConsolidatedLevyAccumulator = ConsolidatedLevyRow & {
  statuses: LevyStatus[]
  paid_dates: string[]
  unit_number_set: Set<string>
}

function asConsolidatedLevyRow(row: Record<string, unknown>): ConsolidatedLevyRow {
  return row as unknown as ConsolidatedLevyRow
}

const { t } = useI18n()
const { formatCurrency, formatDate } = useFormatters()
const { hasAnyRole } = useRole()

const canManage = computed(() => hasAnyRole(['manager']))
const canViewLevies = computed(() => hasAnyRole(['manager', 'board_member']))
const canEditSelectedBudget = computed(() => canManage.value && selectedBudget.value?.status !== 'closed')
const canMarkLeviesPaid = computed(() => hasAnyRole(['manager', 'board_member']))
const sharePercentageFormatter = new Intl.NumberFormat('fr-FR', {
  minimumFractionDigits: 1,
  maximumFractionDigits: 1,
})
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

const selectedBudgetTab = computed({
  get: () => selectedBudgetId.value ?? '',
  set: (v: string) => { selectedBudgetId.value = v || null },
})

const budgetYearTabs = computed<Tab[]>(() =>
  budgets.value.map(b => ({ value: b.id, label: String(b.fiscal_year) }))
)
const budgetLines = ref<BudgetLine[]>([])
const levies = ref<LevyCall[]>([])
const worksFund = ref<WorksFund | null>(null)
const allocationKeys = ref<AllocationKey[]>([])
const unbalancedAllocationKeys = ref<string[]>([])
// ── Derived ───────────────────────────────────────────────────────

const selectedBudget = computed(() =>
  budgets.value.find(b => b.id === selectedBudgetId.value) ?? null
)

function aggregateLevyStatus(statuses: LevyStatus[]): LevyStatus {
  if (statuses.includes('overdue')) return 'overdue'
  if (statuses.every(status => status === 'paid')) return 'paid'
  return 'issued'
}

const consolidatedLevies = computed<ConsolidatedLevyRow[]>(() => {
  const groups = new Map<string, ConsolidatedLevyAccumulator>()

  for (const levy of levies.value) {
    const ownerName = levy.owner_name?.trim() || `${t('common.lot')} ${levy.unit_number ?? '—'}`
    const ownerKey = levy.owner_id ?? `unit:${levy.unit_id}`
    const groupKey = [ownerKey, levy.call_label, levy.due_date, levy.issued_at].join('|')
    const existing = groups.get(groupKey)

    if (existing) {
      existing.amount += Number(levy.amount ?? 0)
      existing.statuses.push(levy.status)
      existing.levy_ids.push(levy.id)
      if (levy.status !== 'paid') {
        existing.payable_unit_ids.push(levy.unit_id)
      }
      if (levy.paid_at) {
        existing.paid_dates.push(levy.paid_at)
      }
      if (levy.unit_number) {
        existing.unit_number_set.add(levy.unit_number)
      }
      continue
    }

    const accumulator: ConsolidatedLevyAccumulator = {
      id: groupKey,
      owner_name: ownerName,
      unit_numbers: [],
      amount: Number(levy.amount ?? 0),
      due_date: levy.due_date,
      call_label: levy.call_label,
      issued_at: levy.issued_at,
      paid_at: levy.paid_at ?? null,
      status: levy.status,
      levy_call_id: levy.id,
      levy_ids: [levy.id],
      payable_unit_ids: levy.status === 'paid' ? [] : [levy.unit_id],
      statuses: [levy.status],
      paid_dates: levy.paid_at ? [levy.paid_at] : [],
      unit_number_set: new Set(levy.unit_number ? [levy.unit_number] : []),
    }
    groups.set(groupKey, accumulator)
  }

  return Array.from(groups.values())
    .map(group => ({
      id: group.id,
      owner_name: group.owner_name,
      unit_numbers: Array.from(group.unit_number_set).sort((a, b) =>
        a.localeCompare(b, undefined, { numeric: true }),
      ),
      amount: group.amount,
      due_date: group.due_date,
      call_label: group.call_label,
      issued_at: group.issued_at,
      paid_at: group.paid_dates.sort().at(-1) ?? null,
      status: aggregateLevyStatus(group.statuses),
      levy_call_id: group.levy_call_id,
      levy_ids: group.levy_ids,
      payable_unit_ids: group.payable_unit_ids,
    }))
    .sort((a, b) =>
      b.due_date.localeCompare(a.due_date)
      || a.owner_name.localeCompare(b.owner_name)
      || a.call_label.localeCompare(b.call_label),
    )
})

const overdueLevies = computed(() => consolidatedLevies.value.filter(levy => levy.status === 'overdue'))
const overdueTotal = computed(() => overdueLevies.value.reduce((sum, levy) => sum + levy.amount, 0))
const levyTableRows = computed<Record<string, unknown>[]>(() => consolidatedLevies.value as unknown as Record<string, unknown>[])

const keyOptions = computed(() =>
  allocationKeys.value.map(k => ({ value: k.id, label: k.name }))
)

const selectedTotalForecast = computed(() =>
  Number(selectedBudget.value?.total_forecast ?? 0)
)
const selectedTotalActual = computed(() =>
  Number(selectedBudget.value?.total_actual ?? 0)
)

const levyCallCount = computed(() => consolidatedLevies.value.length)
const leviedUnitsCount = computed(() =>
  new Set(levies.value.map(levy => levy.unit_id)).size
)
const levyPaidCount = computed(() =>
  consolidatedLevies.value.filter(levy => levy.status === 'paid').length
)
const totalLeviedAmount = computed(() =>
  consolidatedLevies.value.reduce((sum, levy) => sum + levy.amount, 0)
)
const alreadyCalledChargeAmount = computed(() =>
  // Keep a defensive fallback for pre-migration payloads in local/demo environments.
  levies.value.reduce((sum, levy) => sum + Number(levy.charge_amount ?? levy.amount ?? 0), 0)
)
const alreadyCalledWorksFundAmount = computed(() =>
  levies.value.reduce((sum, levy) => sum + Number(levy.works_fund_amount ?? 0), 0)
)
const alreadyCalledDisplayAmount = computed(() =>
  alreadyCalledChargeAmount.value + alreadyCalledWorksFundAmount.value
)
const levyCollectionRate = computed(() =>
  levyCallCount.value > 0 ? Math.round((levyPaidCount.value / levyCallCount.value) * 100) : 0
)
const worksFundContributionRate = computed(() =>
  Number(worksFund.value?.contribution_rate_pct ?? worksFund.value?.minimum_legal_rate_pct ?? 5)
)
const worksFundMinimumRate = computed(() =>
  Number(worksFund.value?.minimum_legal_rate_pct ?? 5)
)
const worksFundContributionTarget = computed(() =>
  selectedTotalForecast.value * (worksFundContributionRate.value / 100)
)
const worksFundCalledAmount = computed(() => Number(worksFund.value?.levied_amount ?? 0))
const worksFundAvailableAmount = computed(() => Number(worksFund.value?.available_amount ?? 0))
const worksFundRemainingToCall = computed(() =>
  Math.max(worksFundContributionTarget.value - worksFundCalledAmount.value, 0)
)
const worksFundProgressPct = computed(() => {
  if (worksFundContributionTarget.value <= 0) return 0
  return Math.min(100, Math.round((worksFundCalledAmount.value / worksFundContributionTarget.value) * 100))
})
const worksFundRateCompliant = computed(() =>
  Boolean(worksFund.value?.is_rate_compliant ?? worksFundContributionRate.value >= worksFundMinimumRate.value)
)
const worksFundTargetMet = computed(() => worksFundRemainingToCall.value < 0.01)
const hasUnbalancedAllocationKeys = computed(() => unbalancedAllocationKeys.value.length > 0)
const impactedLotsCount = computed(() =>
  buildingStats.value?.nb_lots ?? leviedUnitsCount.value
)
const generatePreviewChargeTotal = computed(() =>
  Math.max(selectedTotalForecast.value - alreadyCalledChargeAmount.value, 0)
)
const generatePreviewWorksFundTotal = computed(() =>
  Math.max(worksFundContributionTarget.value - alreadyCalledWorksFundAmount.value, 0)
)
const generatePreviewTotal = computed(() =>
  generatePreviewChargeTotal.value + generatePreviewWorksFundTotal.value
)
const canGenerateLevies = computed(() =>
  selectedBudget.value?.status !== 'closed'
  && generatePreviewTotal.value > 0
  && !hasUnbalancedAllocationKeys.value
  && worksFundRateCompliant.value
)

const levyTableColumns = computed<Column<Record<string, unknown>>[]>(() => {
  const columns: Column<Record<string, unknown>>[] = [
    { key: 'owner_name', label: t('common.coOwner') },
    { key: 'amount', label: t('common.amount'), align: 'right', numeric: true },
    { key: 'due_date', label: t('common.dueDate'), numeric: true },
    { key: 'call_label', label: t('budget.levyCallLabel') },
    { key: 'status', label: t('common.status') },
  ]

  if (canMarkLeviesPaid.value) {
    columns.push({ key: 'actions', label: '', align: 'right' })
  }

  return columns
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

const totalsPerKey = computed(() => {
  const map = new Map<string, { keyName: string; total: number }>()
  for (const line of budgetLines.value) {
    const keyName = line.key_name ?? t('common.noKey')
    const existing = map.get(line.key_id)
    if (existing) {
      existing.total += Number(line.budgeted_amount ?? 0)
    } else {
      map.set(line.key_id, { keyName, total: Number(line.budgeted_amount ?? 0) })
    }
  }
  return Array.from(map.values()).sort((a, b) => a.keyName.localeCompare(b.keyName))
})

const legalCardLinksLevies = computed(() => [
  {
    label: t('budget.legalCard.article35LinkLabel'),
    href: t('budget.legalCard.article35LinkHref'),
  },
  {
    label: t('budget.legalCard.article37LinkLabel'),
    href: t('budget.legalCard.article37LinkHref'),
  },
])

const legalCardLinksWorksFund = computed(() => [
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

  try {
    const balances = await Promise.all(
      keysRes.data.map(async (key) => {
        const res = await allocationKeysApi.balance(key.id)
        return res.data.balanced ? null : key.name
      }),
    )
    unbalancedAllocationKeys.value = balances.filter((value): value is string => Boolean(value))
  } catch {
    unbalancedAllocationKeys.value = []
  }
}

async function loadBudgets() {
  const res = await budgetApi.list()
  budgets.value = res.data
  if (res.data.length > 0 && !selectedBudgetId.value) {
    selectedBudgetId.value = res.data[0].id
  }
}

async function loadBudgetData(budgetId: string) {
  const leviesRequest = canViewLevies.value
    ? budgetApi.listLevies(budgetId).catch((error: unknown) => {
        if ((error as { response?: { status?: number } })?.response?.status === 403) {
          return { data: [] as LevyCall[] }
        }
        throw error
      })
    : Promise.resolve({ data: [] as LevyCall[] })

  const [budgetRes, linesRes, leviesRes] = await Promise.all([
    budgetApi.get(budgetId),
    budgetApi.listLines(budgetId),
    leviesRequest,
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
    charge_amount: levy.charge_amount != null ? Number(levy.charge_amount) : Number(levy.amount),
    works_fund_amount: levy.works_fund_amount != null ? Number(levy.works_fund_amount) : 0,
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
        contribution_rate_pct: Number(wfRes.data.contribution_rate_pct),
        minimum_legal_rate_pct: Number(wfRes.data.minimum_legal_rate_pct),
        is_rate_compliant: Boolean(wfRes.data.is_rate_compliant),
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
  if (!id || loading.value) return
  await loadBudgetData(id)
})

// ── Create budget modal ─────────────────────────────────────

const showCreateBudgetModal = ref(false)

function openCreateBudgetModal() {
  showCreateBudgetModal.value = true
}

async function handleBudgetCreated(id: string) {
  await loadBudgets()
  selectedBudgetId.value = id
}

// ── Create line modal ───────────────────────────────────────

const showCreateLineModal = ref(false)

// ── Delete line ─────────────────────────────────────────────

const lineToDelete = ref<BudgetLine | null>(null)

// ── Generate levies modal ───────────────────────────────────

const showGenerateModal = ref(false)
const showBreakdownModal = ref(false)

// ── Works fund settings modal ────────────────────────────────────

const showWorksFundModal = ref(false)

function openWorksFundModal() {
  showWorksFundModal.value = true
}

async function handleWorksFundSaved() {
  if (selectedBudgetId.value) await loadBudgetData(selectedBudgetId.value)
}

// ── Mark paid ───────────────────────────────────────────────

const markingPaidId = ref<string | null>(null)

async function markPaid(levy: ConsolidatedLevyRow) {
  if (markingPaidId.value || levy.payable_unit_ids.length === 0) return
  markingPaidId.value = levy.id
  try {
    await Promise.all(
      levy.payable_unit_ids.map(unitId => budgetApi.markPaid(levy.levy_call_id, unitId)),
    )
    if (selectedBudgetId.value) await loadBudgetData(selectedBudgetId.value)
  } catch {
    // silently handle — row stays as-is
  } finally {
    markingPaidId.value = null
  }
}

// ── Close exercise modal ───────────────────────────────────────────

const showCloseExerciseModal = ref(false)

function openCloseExerciseModal() {
  showCloseExerciseModal.value = true
}

async function handleExerciseClosed() {
  if (selectedBudgetId.value) await loadBudgetData(selectedBudgetId.value)
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

function levyStatusBadgeVariant(s: LevyStatus): 'valid' | 'pending' | 'err' | 'disabled' {
  if (s === 'paid') return 'valid'
  if (s === 'issued') return 'pending'
  if (s === 'overdue') return 'err'
  return 'disabled'
}

function formatSharePercentage(value: number): string {
  return sharePercentageFormatter.format(value)
}
</script>

<template>
  <div class="view-page">
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
          :icon="CIconLock"
          :title="t('budget.closeExercise')"
          shrink="sm"
          @click="openCloseExerciseModal"
        />
      </template>
    </CTopbar>

    <div class="view-container">
      <CPageLoader v-if="loading" />
      <CPageError
        v-else-if="error"
        :message="error"
        @retry="reload"
      />

      <template
        v-else-if="budgets.length === 0"
      >
        <div class="view-header">
          <CEyebrow>{{ t('page.eyebrows.budget') }}</CEyebrow>
          <CHeading type="header">
            {{ t('budget.title') }}
          </CHeading>
        </div>
        <CEmptyState
          :title="t('budget.noExercice')"
          :description="t('budget.noExerciceHint')"
          size="page"
        >
          <template #actions>
            <CBtn
              v-if="canManage"
              variant="primary"
              :icon="CIconPlus"
              :title="t('budget.newFiscalYear')"
              @click="openCreateBudgetModal"
            />
          </template>
        </CEmptyState>
      </template>

      <template v-else>
        <div class="view-header">
          <CEyebrow>
            <CBadge
              v-if="canEditSelectedBudget"
              :variant="worksFundRateCompliant && worksFundTargetMet && !hasUnbalancedAllocationKeys ? 'valid' : 'pending'"
              dot
              :pulse="false"
            >
              {{ worksFundRateCompliant && worksFundTargetMet && !hasUnbalancedAllocationKeys
                ? t('common.compliant')
                : t('common.actionRequired') }}
            </CBadge>
            {{ t('page.eyebrows.budget') }}
          </CEyebrow>
          <CHeading type="header">
            {{ selectedBudget ? t('budget.fiscalYear', { year: selectedBudget.fiscal_year }) : t('budget.title') }}
          </CHeading>
          <CMetaLine
            v-if="selectedBudget"
          >
            <CMetaItem variant="sub">
              {{ buildingName }}
            </CMetaItem>
            <CSeparator />
            <CMetaItem variant="sub">
              {{ selectedBudget.status === 'approved' ? t('budget.statuses.approvedAG') : t(`budget.statuses.${selectedBudget.status}`) }}
            </CMetaItem>
          </CMetaLine>
        </div>

        <div class="flex items-center gap-2">
          <CTabs
            v-model="selectedBudgetTab"
            variant="filter"
            :tabs="budgetYearTabs"
          />
          <CBtn
            v-if="canManage"
            variant="ghost"
            size="sm"
            class="mb-4"
            :icon="CIconPlus"
            :title="t('budget.newFiscalYear')"
            @click="openCreateBudgetModal"
          />
        </div>

        <template v-if="canEditSelectedBudget">
          <div class="flex flex-col gap-2">
            <CAlert
              v-if="!worksFundRateCompliant"
              variant="error"
              :title="t('budget.worksFundRateAlertTitle')"
            >
              <p>
                {{ t('budget.worksFundRateAlert', {
                  pct: formatSharePercentage(worksFundContributionRate),
                  minimum: formatSharePercentage(worksFundMinimumRate),
                }) }}
              </p>
            </CAlert>
            <CAlert
              v-if="worksFund && !worksFundTargetMet"
              variant="warning"
              :title="t('budget.worksFundTargetAlertTitle')"
            >
              <p>
                {{ t('budget.worksFundTargetAlert', {
                  amount: formatCurrency(worksFundRemainingToCall),
                }) }}
              </p>
            </CAlert>
            <CAlert
              v-if="hasUnbalancedAllocationKeys"
              variant="warning"
              :title="t('budget.allocationKeysAlertTitle')"
            >
              <p>
                {{ t('budget.allocationKeysAlert', {
                  keys: unbalancedAllocationKeys.join(', '),
                }) }}
              </p>
            </CAlert>
          </div>
        </template>

        <div class="grid gap-4 xs:grid-cols-2 xl:grid-cols-4">
          <CKpiCard
            :label="t('budget.budgetForecastLabel')"
            :value="formatCurrency(selectedTotalForecast)"
            :subtitle="t('budget.budgetItems', budgetLines.length)"
          >
            <template #icon>
              <CIconCurrency aria-hidden="true" />
            </template>
          </CKpiCard>

          <CKpiCard
            :label="t('budget.realizedToDate')"
            :value="formatCurrency(selectedTotalActual)"
            :subtitle="t('budget.ofBudget', {
              pct: selectedTotalForecast ? Math.round((selectedTotalActual / selectedTotalForecast) * 100) : 0,
            })"
            tone="accent"
          >
            <template #icon>
              <CIconCheck aria-hidden="true" />
            </template>
          </CKpiCard>

          <CKpiCard
            v-if="canViewLevies"
            :label="t('budget.overdueCallsLabel')"
            :value="overdueLevies.length > 0 ? String(overdueLevies.length) : t('common.none')"
            :subtitle="t('budget.pendingAmount', { amount: formatCurrency(overdueTotal) })"
            :emphasize-value="true"
            :tone="overdueLevies.length > 0 ? 'err' : 'valid'"
          >
            <template #icon>
              <CIconWarn
                v-if="overdueLevies.length > 0"
                aria-hidden="true"
              />
              <CIconCheck
                v-else
                aria-hidden="true"
              />
            </template>
          </CKpiCard>

          <CKpiCard
            v-if="canViewLevies"
            :label="t('budget.generateSummaryAlreadyCalledLabel')"
            :value="formatCurrency(totalLeviedAmount)"
            :subtitle="t('budget.levyCallsCount', { count: levyCallCount })"
            tone="accent"
          >
            <template #icon>
              <CIconMegaphone aria-hidden="true" />
            </template>
          </CKpiCard>
        </div>

        <CCard>
          <CSectionHeader>
            <CHeading type="card">
              {{ t('budget.worksFund') }}
            </CHeading>
            <CMetaLine v-if="selectedBudget">
              <CMetaItem variant="sub">
                {{ t('budget.worksFundRateSummary', {
                  pct: formatSharePercentage(worksFundContributionRate),
                  minimum: formatSharePercentage(worksFundMinimumRate),
                }) }}
              </CMetaItem>
            </CMetaLine>
            <template #actions>
              <CBtn
                v-if="canEditSelectedBudget && selectedBudget"
                variant="ghost"
                size="sm"
                :icon="CIconWrench"
                :title="t('budget.worksFundConfigure')"
                shrink="sm"
                @click="openWorksFundModal"
              />
            </template>
          </CSectionHeader>

          <p>
            {{ t('budget.worksFundLegal') }}
          </p>

          <div class="grid grid-cols-2 gap-3 md:grid-cols-4">
            <div class="wf-metric">
              <span class="text-xs">{{ t('budget.worksFundTarget') }}</span>
              <strong class="font-mono">{{ formatCurrency(worksFundContributionTarget) }}</strong>
            </div>
            <div class="wf-metric">
              <span class="text-xs">{{ t('budget.worksFundRate') }}</span>
              <strong>{{ formatSharePercentage(worksFundContributionRate) }}%</strong>
            </div>
            <div class="wf-metric">
              <span class="text-xs">{{ t('budget.worksFundCalled') }}</span>
              <strong class="font-mono">{{ formatCurrency(worksFundCalledAmount) }}</strong>
            </div>
            <div class="wf-metric">
              <span class="text-xs">{{ t('budget.worksFundAvailable') }}</span>
              <strong class="font-mono">{{ formatCurrency(worksFundAvailableAmount) }}</strong>
            </div>
          </div>

          <div>
            <CProgressBar
              :value="worksFundProgressPct"
              variant="warning"
              :show-value="true"
            />
            <div class="prog-foot">
              <span>{{ t('budget.worksFundProgressLabel', { pct: worksFundProgressPct }) }}</span>
              <span
                class="text-end font-medium"
                style="color: var(--color-warning)"
              >
                {{ t('budget.worksFundRemainingToCall', { amount: formatCurrency(worksFundRemainingToCall) }) }}
              </span>
            </div>
          </div>
        </CCard>

        <div
          v-if="canViewLevies && overdueLevies.length > 0"
          class="flex flex-col gap-4"
        >
          <CSectionHeader>
            <CHeading>
              {{ t('budget.latePayments') }}
            </CHeading>
            <CMetaLine>
              <CMetaItem variant="sub">
                {{ t('budget.latePaymentsCount', { count: overdueLevies.length }) }}
              </CMetaItem>
              <CSeparator />
              <CMetaItem variant="sub">
                {{ t('budget.latePaymentsTotal', { amount: formatCurrency(overdueTotal) }) }}
              </CMetaItem>
            </CMetaLine>
          </CSectionHeader>
          <CCard
            v-for="levy in overdueLevies"
            :key="levy.id"
            class="tone--err"
            :shadow="false"
            :level="4"
            :style="{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', gap: 'var(--space-4)' }"
          >
            <div class="flex gap-3">
              <CAvatar
                :name="levy.owner_name"
                size="lg"
              />
              <div>
                <div class="text-on-surface font-medium">
                  {{ levy.owner_name ?? '—' }}
                </div>
                <CMetaLine>
                  <CMetaItem variant="sub">
                    {{ t("common.lot") }} {{ levy.unit_numbers.join(', ') || '—' }}
                  </CMetaItem>
                  <CSeparator />
                  <CMetaItem variant="sub">
                    {{ levy.call_label }}
                  </CMetaItem>
                </CMetaLine>
              </div>
            </div>
            <div class="text-right">
              <div class="font-mono font-medium">
                {{ formatCurrency(levy.amount) }}
              </div>
              <CMetaItem variant="sub">
                {{ levy.due_date ? formatDate(levy.due_date) : '—' }}
              </CMetaItem>
            </div>
          </CCard>
        </div>
        <CCard gap="md">
          <CSectionHeader>
            <CHeading type="card">
              {{ t('budget.consumptionByLine') }}
            </CHeading>
            <CMetaItem variant="sub">
              {{ t('budget.forecastVsActual') }}
            </CMetaItem>
            <template #actions>
              <CBtn
                v-if="budgetBreakdownItems.length > 0"
                variant="ghost"
                size="sm"
                :icon="CIconPieChart"
                :title="t('budget.breakdown')"
                shrink="sm"
                @click="showBreakdownModal = true"
              />
              <CBtn
                v-if="canEditSelectedBudget"
                variant="ghost"
                size="sm"
                :icon="CIconPlus"
                :title="t('budget.newLine')"
                shrink="sm"
                @click="showCreateLineModal = true"
              />
            </template>
          </CSectionHeader>

          <div
            v-if="budgetLines.length === 0"
            class="py-8 text-center text-md text-on-surface-variant"
          >
            <p>{{ t('common.noData') }}</p>
          </div>

          <template v-else>
            <div
              v-if="totalsPerKey.length > 0"
              class="flex flex-wrap gap-3 pb-2 border-b border-outline-variant"
            >
              <div
                v-for="kt in totalsPerKey"
                :key="kt.keyName"
                class="flex flex-col gap-0.5"
              >
                <span class="text-xs text-on-surface-variant">{{ kt.keyName }}</span>
                <span class="font-mono font-medium text-sm">{{ formatCurrency(kt.total) }}</span>
              </div>
            </div>
          </template>

          <CFlexGroup v-if="budgetLines.length > 0">
            <div
              v-for="line in budgetLines"
              :key="line.id"
            >
              <div class="flex justify-between items-center mb-2">
                <span class="text-sm font-medium text-on-surface">{{ line.label }}</span>
                <CMetaLine class="text-xs">
                  <CBadge
                    v-if="line.key_name"
                    variant="primary"
                  >
                    {{ line.key_name }}
                  </CBadge>
                  <CBtn
                    v-if="canEditSelectedBudget"
                    variant="ghost"
                    size="sm"
                    :icon="CIconTrash"
                    :aria-label="t('common.delete')"
                    @click="lineToDelete = line"
                  />
                </CMetaLine>
              </div>
              <CProgressBar
                :value="linePct(line)"
                :variant="lineVariant(line)"
                :show-value="true"
              />
              <div class="prog-foot">
                <span>{{ t('budget.realizedLabel') }} : <b class="text-on-surface font-medium font-mono">{{ formatCurrency(line.actual_amount ?? 0) }}</b></span>
                <span>{{ t('budget.plannedLabel') }} : <b class="text-on-surface font-medium font-mono">{{ formatCurrency(line.budgeted_amount) }}</b></span>
              </div>
            </div>
          </CFlexGroup>
          <div
            v-if="budgetLines.length > 0"
            class="flex justify-between border-t border-outline-variant font-semibold text-sm text-on-surface pt-4"
          >
            <span>{{ t('common.total') }}</span>
            <span class="font-mono">
              {{ formatCurrency(selectedTotalActual) }} / {{ formatCurrency(selectedTotalForecast) }}
            </span>
          </div>
        </CCard>
      
        <CCard
          v-if="canViewLevies"
          gap="md"
        >
          <CSectionHeader>
            <CHeading type="card">
              {{ t('budget.levies') }}
            </CHeading>
            <CMetaLine>
              <CMetaItem variant="sub">
                {{ t('budget.levyCallsCount', { count: levyCallCount }) }}
              </CMetaItem>
              <CSeparator />
              <CMetaItem variant="sub">
                {{ t('budget.collectionRatePct', { pct: levyCollectionRate }) }}
              </CMetaItem>
            </CMetaLine>
            <template #actions>
              <CBtn
                v-if="canEditSelectedBudget && selectedBudgetId"
                variant="ghost"
                size="sm"
                :icon="CIconMegaphone"
                :title="t('budget.generateLevies')"
                shrink="sm"
                @click="showGenerateModal = true"
              />
            </template>
          </CSectionHeader>

      

          <div
            v-if="consolidatedLevies.length === 0"
            class="py-8 text-center text-md text-on-surface-variant"
          >
            <p>{{ t('common.noData') }}</p>
          </div>

          <CDataTable
            v-else
            class="-mx-5 border-b border-outline-variant"
            :columns="levyTableColumns"
            :rows="levyTableRows"
            :empty-text="t('common.noData')"
          >
            <template #cell-owner_name="{ row }">
              <div>
                <div>{{ row.owner_name ?? '—' }}</div>
                <div class="text-on-surface-variant text-xs">
                  {{ t('common.lot') }} {{ asConsolidatedLevyRow(row).unit_numbers.join(', ') || '—' }}
                </div>
              </div>
            </template>
            <template #cell-amount="{ row }">
              <span
                :class="{ 'text-error': row.status === 'overdue' }"
              >
                {{ formatCurrency(Number(row.amount ?? 0)) }}
              </span>
            </template>
            <template #cell-due_date="{ row }">
              <span class="text-on-surface-variant text-xs">
                {{ asConsolidatedLevyRow(row).due_date ? formatDate(asConsolidatedLevyRow(row).due_date) : '—' }}
              </span>
            </template>
            <template #cell-call_label="{ row }">
              <span class="text-on-surface-variant text-xs">
                {{ row.call_label || t('budget.defaultLevyLabel') }}
              </span>
            </template>
            <template #cell-status="{ row }">
              <CBadge
                :variant="levyStatusBadgeVariant(row.status as LevyStatus)"
                :pulse="row.status === 'overdue'"
                :ring="row.status === 'issued' || row.status === 'overdue'"
                dot
              >
                {{ t(`budget.levyStatuses.${row.status}`) }}
              </CBadge>
            </template>
            <template #cell-actions="{ row }">
              <CBtn
                v-if="canMarkLeviesPaid && row.status !== 'paid'"
                variant="icon"
                size="sm"
                :icon="CIconCheckCircle"
                :title="t('budget.markPaid')"
                :loading="markingPaidId === asConsolidatedLevyRow(row).id"
                class="tone--valid"
                @click="markPaid(asConsolidatedLevyRow(row))"
              />
            </template>
          </CDataTable>
          <div class="flex justify-between items-center text-sm font-semibold text-on-surface">
            <span>{{ t('common.total') }}</span>
            <span class="font-mono">
              {{ formatCurrency(totalLeviedAmount) }}
            </span>
          </div>
        </CCard>

        <CLegalCard>
          <CFlexGroup direction="column">
            <CLegalNotice :links="legalCardLinksLevies">
              <CLegalDescription
                :title="t('budget.legalCard.title')"
                :summary="t('budget.legalCard.summary')"
              />
            </CLegalNotice>
            <CLegalNotice :links="legalCardLinksWorksFund">
              <CLegalDescription
                :title="t('budget.legalCard.worksFundTitle')"
                :summary="t('budget.legalCard.worksFundSummary')"
              />
            </CLegalNotice>
          </CFlexGroup>
        </CLegalCard>
      </template>
    </div>

    <CreateBudgetModal
      :open="showCreateBudgetModal"
      @update:open="showCreateBudgetModal = $event"
      @created="handleBudgetCreated"
    />

    <CreateLineModal
      :open="showCreateLineModal"
      :budget-id="selectedBudgetId ?? ''"
      :key-options="keyOptions"
      @update:open="showCreateLineModal = $event"
      @created="loadBudgetData(selectedBudgetId!)"
    />

    <DeleteLineModal
      :open="!!lineToDelete"
      :budget-id="selectedBudgetId ?? ''"
      :line="lineToDelete"
      @update:open="v => { if (!v) lineToDelete = null }"
      @deleted="loadBudgetData(selectedBudgetId!)"
    />

    <GenerateLeviesModal
      :open="showGenerateModal"
      :budget-id="selectedBudgetId!"
      :impacted-lots-count="impactedLotsCount"
      :already-called-display-amount="alreadyCalledDisplayAmount"
      :generate-preview-total="generatePreviewTotal"
      :selected-total-forecast="selectedTotalForecast"
      :can-generate-levies="canGenerateLevies"
      @update:open="showGenerateModal = $event"
      @generated="loadBudgetData(selectedBudgetId!)"
    />

    <BreakdownModal
      :open="showBreakdownModal"
      :items="budgetBreakdownItems"
      @update:open="showBreakdownModal = $event"
    />

    <WorksFundModal
      :open="showWorksFundModal"
      :fiscal-year="selectedBudget?.fiscal_year ?? 0"
      :current-rate="worksFundContributionRate"
      :minimum-rate="worksFundMinimumRate"
      @update:open="showWorksFundModal = $event"
      @saved="handleWorksFundSaved"
    />

    <CloseExerciseModal
      :open="showCloseExerciseModal"
      :budget-id="selectedBudgetId ?? ''"
      :fiscal-year="selectedBudget?.fiscal_year ?? 0"
      @update:open="showCloseExerciseModal = $event"
      @closed="handleExerciseClosed"
    />
  </div>
</template>

<style scoped>
.wf-metric {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding: var(--space-3);
  border-radius: var(--radius-xl);
  background: color-mix(in srgb, var(--color-warning) var(--color-tint), transparent);
}

.prog-foot {
  display: flex;
  justify-content: space-between;
  margin-top: var(--space-1);
  font-size: var(--text-2xs);
  color: var(--color-on-surface-variant);
}

.stat-cell {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
</style>
