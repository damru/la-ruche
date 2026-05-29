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
  alreadyCalledChargeAmount.value
  + (generateForm.value.include_works_fund ? alreadyCalledWorksFundAmount.value : 0)
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
  generateForm.value.include_works_fund
    ? Math.max(worksFundContributionTarget.value - alreadyCalledWorksFundAmount.value, 0)
    : 0
)
const generatePreviewTotal = computed(() =>
  generatePreviewChargeTotal.value + generatePreviewWorksFundTotal.value
)
const canGenerateLevies = computed(() =>
  selectedBudget.value?.status !== 'closed'
  && generatePreviewTotal.value > 0
  && !hasUnbalancedAllocationKeys.value
  && (!generateForm.value.include_works_fund || worksFundRateCompliant.value)
)
const generateBlockingReason = computed(() => {
  if (hasUnbalancedAllocationKeys.value) {
    return t('budget.allocationKeysBlocked', {
      keys: unbalancedAllocationKeys.value.join(', '),
    })
  }

  if (generateForm.value.include_works_fund && !worksFundRateCompliant.value) {
    return t('budget.worksFundRateBlocked', {
      minimum: formatSharePercentage(worksFundMinimumRate.value),
    })
  }

  return ''
})
const generateFeedbackStatus = computed(() =>
  generateFeedback.status.value !== 'idle'
    ? generateFeedback.status.value
    : (generateBlockingReason.value ? 'error' : 'idle')
)
const generateFeedbackMessage = computed(() =>
  generateFeedback.status.value !== 'idle'
    ? generateFeedback.message.value
    : generateBlockingReason.value
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

// ── Create line modal ───────────────────────────────────────

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

// ── Delete line ─────────────────────────────────────────────

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

// ── Generate levies modal ───────────────────────────────────

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

// ── Works fund settings modal ────────────────────────────────────

const showWorksFundModal = ref(false)
const savingWorksFund = ref(false)
const worksFundFeedback = useFormFeedback()
const worksFundForm = ref({ contribution_rate_pct: '' })

function openWorksFundModal() {
  worksFundFeedback.reset()
  worksFundForm.value = {
    contribution_rate_pct: worksFund.value
      ? worksFundContributionRate.value.toFixed(2)
      : worksFundMinimumRate.value.toFixed(2),
  }
  showWorksFundModal.value = true
}

async function saveWorksFund() {
  if (!selectedBudget.value) return

  const contributionRatePct = Number.parseFloat(worksFundForm.value.contribution_rate_pct)
  if (!Number.isFinite(contributionRatePct) || contributionRatePct < 0 || contributionRatePct > 100) {
    worksFundFeedback.setError(t('errors.budget.works_fund_rate_invalid'))
    return
  }

  savingWorksFund.value = true
  worksFundFeedback.reset()
  try {
    await budgetApi.updateWorksFund(selectedBudget.value.fiscal_year, {
      contribution_rate_pct: contributionRatePct,
    })
    if (selectedBudgetId.value) {
      await loadBudgetData(selectedBudgetId.value)
    }
    showWorksFundModal.value = false
  } catch (err) {
    worksFundFeedback.setError(toErrorMessage(err))
  } finally {
    savingWorksFund.value = false
  }
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

        <!-- KPI row -->
        <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
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
              <span class="text-end font-medium" style="color: var(--color-warning)">
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
              @click="openCreateLineModal"
            />
          </template>
        </CSectionHeader>

        <div
          v-if="budgetLines.length === 0"
          class="py-8 text-center text-md text-on-surface-variant"
        >
          <p>{{ t('common.noData') }}</p>
        </div>

        <CFlexGroup v-else>
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
              @click="openGenerateModal"
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
          :title="t('common.cancel')"
          @click="showCreateBudgetModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.create')"
          :loading="savingBudget"
          @click="saveBudget"
        />
      </template>
    </CModal>

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
          :title="t('common.cancel')"
          @click="showCreateLineModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="savingLine"
          @click="saveLine"
        />
      </template>
    </CModal>

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
          :title="t('common.cancel')"
          @click="lineToDelete = null"
        />
        <CBtn
          variant="danger"
          :title="t('common.delete')"
          :loading="deletingLine"
          @click="confirmDeleteLine"
        />
      </template>
    </CModal>

    <CModal
      size="lg"
      :open="showGenerateModal"
      :title="t('budget.generateLeviesTitle')"
      @update:open="showGenerateModal = $event"
    >
      <p class="body-sm  text-muted">
        {{ t('budget.generateLeviesSub') }}
      </p>
      <CFormInput
        v-model="generateForm.due_date"
        :label="t('common.dueDate')"
        type="date"
        required
      />
      <div class="flex flex-col gap-1">
        <CFormCheckbox
          v-model="generateForm.include_works_fund"
          :label="t('budget.includeWorksFund')"
        />
        <p class="text-xs text-on-surface-variant">
          {{ t('budget.worksFundLegal') }}
        </p>
      </div>
      <CFormInput
        v-model="generateForm.call_label"
        :label="t('budget.callLabel')"
        :placeholder="t('budget.callLabelPlaceholder')"
      />
      <CCardNote
        v-if="selectedTotalForecast > 0"
        class="flex-col"
      >
        <strong>{{ t('budget.generateSummaryLabel') }}</strong>
        <div class="grid gap-3 grid-cols-3">
          <div class="stat-cell">
            <span class="text-2xs opacity-80">{{ t('budget.generateSummaryLotsLabel') }}</span>
            <strong>{{ impactedLotsCount }}</strong>
          </div>
          <div class="stat-cell">
            <span class="text-2xs opacity-80">{{ t('budget.generateSummaryAlreadyCalledLabel') }}</span>
            <strong class="font-mono tabular-nums">{{ formatCurrency(alreadyCalledDisplayAmount) }}</strong>
          </div>
          <div class="stat-cell">
            <span class="text-2xs opacity-80">{{ t('budget.generateSummaryRemainingLabel') }}</span>
            <strong class="font-mono tabular-nums">{{ formatCurrency(generatePreviewTotal) }}</strong>
          </div>
        </div>
        <p
          v-if="generatePreviewTotal === 0"
        >
          {{ t('budget.generateSummaryNothingToGenerate') }}
        </p>
      </CCardNote>
      <template #footer>
        <CFormFeedback
          :status="generateFeedbackStatus"
          :message="generateFeedbackMessage"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showGenerateModal = false"
        />
        <CBtn
          variant="primary"
          :icon="CIconMegaphone"
          :title="t('budget.generate')"
          :disabled="!canGenerateLevies"
          :loading="generatingLevies"
          @click="generateLevies"
        />
      </template>
    </CModal>

    <CModal
      :open="showBreakdownModal"
      :title="t('budget.breakdownTitle')"
      size="lg"
      @update:open="showBreakdownModal = $event"
    >
      <p class="body-sm text-muted">
        {{ t('budget.breakdownIntro') }}
      </p>
      <div
        class="mt-4 grid items-center gap-6 sm:grid-cols-[1fr_auto]"
      >
        <div class="flex justify-center">
          <div
            class="w-[min(280px,56vw)] aspect-square rounded-full"
            :style="{ ...budgetBreakdownStyle, boxShadow: 'inset 0 0 0 1px rgba(25,28,29,0.05)' }"
            role="img"
            :aria-label="t('budget.breakdownAriaLabel')"
          ></div>
        </div>

        <div class="flex flex-col gap-3">
          <div
            v-for="item in budgetBreakdownItems"
            :key="item.id"
            class="flex items-start gap-3"
          >
            <span
              class="mt-1 size-3 shrink-0 rounded-full"
              :style="{ background: item.color }"
              aria-hidden="true"
            ></span>
            <div class="flex min-w-0 flex-col gap-1">
              <strong class="text-sm text-on-surface">{{ item.label }}</strong>
              <CFlexGroup
                direction="row"
                class="flex-wrap"
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
      :open="showWorksFundModal"
      :title="t('budget.worksFundConfigureTitle')"
      @update:open="showWorksFundModal = $event"
    >
      <p class="body-sm mb-4 leading-6 text-muted">
        {{ t('budget.worksFundConfigureHint') }}
      </p>
      <CFormInput
        v-model="worksFundForm.contribution_rate_pct"
        :label="t('budget.worksFundRate')"
        type="number"
        :min="0"
        :max="100"
        step="0.01"
        required
      />
      <p class="text-xs text-on-surface-variant mt-2">
        {{ t('budget.worksFundRateHint', { minimum: formatSharePercentage(worksFundMinimumRate) }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="worksFundFeedback.status.value"
          :message="worksFundFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showWorksFundModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="savingWorksFund"
          @click="saveWorksFund"
        />
      </template>
    </CModal>

    <CModal
      :open="showCloseExerciseModal"
      :title="t('budget.closeExerciseTitle')"
      @update:open="showCloseExerciseModal = $event"
    >
      <p class="body-sm mb-4 leading-6 text-muted">
        {{ t('budget.closeExerciseConfirmation', { year: selectedBudget?.fiscal_year }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="closeExerciseFeedback.status.value"
          :message="closeExerciseFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showCloseExerciseModal = false"
        />
        <CBtn
          variant="danger"
          :title="t('budget.closeExerciseConfirm')"
          :loading="closingExercise"
          @click="closeExercise"
        />
      </template>
    </CModal>
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

