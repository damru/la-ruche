<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useFormatters } from '@/composables/useFormatters'
import CBadge from '@/components/atoms/CBadge.vue'
import CIconTile from '@/components/atoms/CIconTile.vue'
import CIconBolt from '@/components/atoms/CIconBolt.svg'
import CIconDroplet from '@/components/atoms/CIconDroplet.svg'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CCardActions from '@/components/molecules/CCardActions.vue'
import CDataTable, { type Column } from '@/components/molecules/CDataTable.vue'
import CFormMultiSelect from '@/components/atoms/CFormMultiSelect.vue'
import CIconGrid from '@/components/atoms/CIconGrid.svg'
import CIconList from '@/components/atoms/CIconList.svg'
import CLinkedDocumentsModal from '@/components/organisms/CLinkedDocumentsModal.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CReminderCard from '@/components/organisms/CReminderCard.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CIconFile from '@/components/atoms/CIconFile.svg'
import CIconLeaf from '@/components/atoms/CIconLeaf.svg'
import CIconLayers from '@/components/atoms/CIconLayers.svg'
import CIconSettings from '@/components/atoms/CIconSettings.svg'
import CIconShield from '@/components/atoms/CIconShield.svg'
import CIconSparkles from '@/components/atoms/CIconSparkles.svg'
import CIconWrench from '@/components/atoms/CIconWrench.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconArchive from '@/components/atoms/CIconArchive.svg'
import CIconCurrency from '@/components/atoms/CIconCurrency.svg'
import CIconHistory from '@/components/atoms/CIconClock.svg'
import CIconCheck from '@/components/atoms/CIconCheck.svg'
import CIconX from '@/components/atoms/CIconX.svg'
import { contractsApi, documentsApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import ContractFormModal from '@/features/contracts/ContractFormModal.vue'
import AmendContractModal from '@/features/contracts/AmendContractModal.vue'
import AmendmentChainModal from '@/features/contracts/AmendmentChainModal.vue'
import CTabs, { type Tab } from '@/components/atoms/CTabs.vue'
import { toErrorMessage, usePageLoader } from '@/composables/usePageLoader'
import TerminateContractModal from '@/features/contracts/TerminateContractModal.vue'
import { effectiveContractStatus } from '@/utils/contracts'
import { useRole } from '@/composables/useRole'
import type { Role, ContractReminder, ContractStatus, ContractType, Document, VendorContract } from '@/types/domain'
import { ALL_FILTER, CONTRACT_STATUS_ACTIVE, CONTRACT_STATUS_DRAFT, CONTRACT_STATUS_EXPIRED, CONTRACT_STATUS_TERMINATED, REMINDER_URGENCY_HIGH_DAYS, REMINDER_URGENCY_MEDIUM_DAYS } from '@/utils/constants'

interface ContractReminderView extends ContractReminder {
  targetDate: string
  urgency: 'high' | 'medium' | 'low'
  daysLabel: string
}

const { t } = useI18n()
const { formatCurrency, formatDate, daysUntil } = useFormatters()
const { hasAnyRole } = useRole()
const router = useRouter()

type ContractFilter = typeof ALL_FILTER | 'active' | 'expired'
const filter = ref<ContractFilter>(ALL_FILTER)
const search = ref('')
const selectedTypes = ref<string[]>([])
const selectedVendors = ref<string[]>([])
const viewMode = ref<'cards' | 'table'>('table')
const buildingName = ref('')
const allContracts = ref<VendorContract[]>([])
const allReminders = ref<ContractReminder[]>([])
const editingContract = ref<VendorContract | null>(null)
const showFormModal = ref(false)
const busyKey = ref('')
const contractDocumentsTarget = ref<VendorContract | null>(null)
const contractDocuments = ref<Document[]>([])
const loadingContractDocuments = ref(false)

const canManage = computed(() => hasAnyRole(['manager' satisfies Role]))

async function loadData() {
  const [building, contractsResponse, remindersResponse] = await Promise.all([
    useBuildingStore().fetch(),
    contractsApi.list(),
    contractsApi.listAllReminders(),
  ])

  buildingName.value = building.name
  allContracts.value = contractsResponse.data
  allReminders.value = remindersResponse.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

const contractsById = computed(() =>
  new Map(allContracts.value.map(contract => [contract.id, contract])),
)

const reminderViews = computed<ContractReminderView[]>(() =>
  [...allReminders.value]
    .map((reminder) => {
      const targetDate = reminder.target_date ?? reminder.reminder_date
      const remainingDays = daysUntil(targetDate)

      return {
        ...reminder,
        targetDate,
        urgency: reminderUrgency(targetDate),
        daysLabel: remainingDays > 0
          ? t('format.daysLeft', { days: remainingDays })
          : t('contracts.reminders.dueNow'),
      }
    })
    .sort((left, right) => left.targetDate.localeCompare(right.targetDate)),
)

const pendingRemindersByContract = computed(() => {
  const remindersByContract = new Map<string, ContractReminderView[]>()

  for (const reminder of reminderViews.value) {
    const contractReminders = remindersByContract.get(reminder.contract_id)

    if (contractReminders) {
      contractReminders.push(reminder)
    } else {
      remindersByContract.set(reminder.contract_id, [reminder])
    }
  }

  return remindersByContract
})

const reminderViewsById = computed(() =>
  new Map(reminderViews.value.map(reminder => [reminder.id, reminder])),
)

const effectiveStatus = effectiveContractStatus

function reminderUrgency(dateStr: string): 'high' | 'medium' | 'low' {
  const d = daysUntil(dateStr)
  return d <= REMINDER_URGENCY_HIGH_DAYS ? 'high' : d <= REMINDER_URGENCY_MEDIUM_DAYS ? 'medium' : 'low'
}

function reminderTone(urgency: 'high' | 'medium' | 'low'): 'err' | 'warn' | 'muted' {
  if (urgency === 'high') return 'err'
  if (urgency === 'medium') return 'warn'
  return 'muted'
}

function statusVariant(status: ContractStatus): 'open' | 'pending' | 'closed' | 'default' {
  if (status === CONTRACT_STATUS_ACTIVE) return 'open'
  if (status === CONTRACT_STATUS_DRAFT) return 'pending'
  if (status === CONTRACT_STATUS_TERMINATED || status === CONTRACT_STATUS_EXPIRED) return 'closed'
  return 'default'
}

function freqLabel(freq?: string): string {
  return freq ? t(`contracts.frequencies.${freq}`) : '—'
}

function contractTypeIcon(type: ContractType) {
  const icons = {
    maintenance: CIconWrench,
    insurance: CIconShield,
    service: CIconSettings,
    energy: CIconBolt,
    water: CIconDroplet,
    cleaning: CIconSparkles,
    green_spaces: CIconLeaf,
    other: CIconLayers,
  }
  return icons[type] ?? CIconFile
}

function contractTypeTone(type: ContractType) {
  const tones = {
    maintenance: 'tone--warn',
    insurance: 'tone--err',
    service: 'tone--default',
    energy: 'tone--accent',
    water: 'tone--default',
    cleaning: 'tone--valid',
    green_spaces: 'tone--valid',
    other: 'tone--default',
  }

  return tones[type] ?? 'tone--default'
}

function contractIconTone(contract: VendorContract): string {
  if (effectiveStatus(contract) !== CONTRACT_STATUS_ACTIVE) return 'tone--disabled'
  return contractTypeTone(contract.contract_type)
}

const contracts = computed(() => {
  const query = search.value.trim().toLowerCase()

  return [...allContracts.value]
    .filter((contract) => {
      const currentStatus = effectiveStatus(contract)
      if (filter.value === 'active' && currentStatus !== CONTRACT_STATUS_ACTIVE) return false
      if (filter.value === 'expired' && currentStatus === CONTRACT_STATUS_ACTIVE) return false

      if (selectedTypes.value.length > 0 && !selectedTypes.value.includes(contract.contract_type)) {
        return false
      }

      if (selectedVendors.value.length > 0 && !selectedVendors.value.includes(contract.vendor_name)) {
        return false
      }

      if (!query) return true

      return [
        contract.vendor_name,
        contract.subject,
        contract.vendor_email ?? '',
        contract.vendor_phone ?? '',
      ]
        .join(' ')
        .toLowerCase()
        .includes(query)
    })
    .sort((left, right) => left.vendor_name.localeCompare(right.vendor_name))
})

const contractTypeOptions = computed(() =>
  [...new Set(allContracts.value.map(contract => contract.contract_type))]
    .map(value => ({ value, label: t(`contracts.types.${value}`) }))
    .sort((a, b) => a.label.localeCompare(b.label)),
)

const contractVendorOptions = computed(() =>
  [...new Set(allContracts.value.map(contract => contract.vendor_name))]
    .sort((a, b) => a.localeCompare(b))
    .map(vendor => ({ value: vendor, label: vendor })),
)

const hasActiveFilters = computed(
  () => selectedTypes.value.length > 0 || selectedVendors.value.length > 0,
)

function clearFilters() {
  selectedTypes.value = []
  selectedVendors.value = []
}

const contractTableColumns = computed<Column<VendorContract>[]>(() => [
  { key: 'vendor_name', label: t('contracts.form.vendorName') },
  { key: 'subject', label: t('contracts.form.subject') },
  { key: 'contract_type', label: t('common.type') },
  { key: 'start_date', label: t('common.startDate') },
  { key: 'end_date', label: t('common.endDate') },
  { key: 'amount', label: t('common.amount'), align: 'right' },
  { key: 'status', label: t('common.status') },
])

function openContractDetail(contract: VendorContract) {
  router.push({ name: 'contract-detail', params: { id: contract.id } })
}

const contractCounts = computed(() =>
  allContracts.value.reduce(
    (counts, contract) => {
      counts.all += 1
      if (effectiveStatus(contract) === CONTRACT_STATUS_ACTIVE) {
        counts.active += 1
      } else {
        counts.expired += 1
      }
      return counts
    },
    { all: 0, active: 0, expired: 0 },
  ),
)

const contractTabs = computed(() => [
  { key: ALL_FILTER, label: t('common.allMasc'), count: contractCounts.value.all },
  { key: 'active', label: t('contracts.filters.active'), count: contractCounts.value.active },
  { key: 'expired', label: t('contracts.filters.expired'), count: contractCounts.value.expired },
])

const contractTabsList = computed<Tab[]>(() =>
  contractTabs.value.map(t => ({ value: t.key, label: t.label, count: t.count }))
)

const contractReminderRows = computed(() =>
  reminderViews.value.map(reminder => ({
    id: reminder.id,
    title: contractsById.value.get(reminder.contract_id)?.vendor_name ?? '—',
    subtitle: t(`contracts.reminderTypes.${reminder.reminder_type}`),
    meta: formatDate(reminder.targetDate),
    emphasis: reminder.daysLabel,
    tone: reminderTone(reminder.urgency),
  })),
)

function openCreateModal() {
  editingContract.value = null
  showFormModal.value = true
}

function openEditModal(contract: VendorContract) {
  editingContract.value = contract
  showFormModal.value = true
}

async function openContractDocuments(contract: VendorContract) {
  contractDocumentsTarget.value = contract
  contractDocuments.value = []
  loadingContractDocuments.value = true

  try {
    const response = await contractsApi.listDocuments(contract.id)
    contractDocuments.value = response.data
  } catch (currentError: unknown) {
    error.value = toErrorMessage(currentError, t('contracts.documents.loadError'))
    contractDocumentsTarget.value = null
  } finally {
    loadingContractDocuments.value = false
  }
}

async function downloadContractDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch (currentError: unknown) {
    error.value = toErrorMessage(currentError, t('contracts.documents.downloadError'))
  }
}

async function acknowledgeReminder(reminder: ContractReminder) {
  busyKey.value = `reminder:${reminder.id}`

  try {
    await contractsApi.acknowledgeReminder(reminder.contract_id, reminder.id)
    await reload()
  } catch (currentError: unknown) {
    error.value = toErrorMessage(currentError, t('contracts.reminders.acknowledgeError'))
  } finally {
    busyKey.value = ''
  }
}

// ── Terminate contract modal ──────────────────────────────────────

const showTerminateModal = ref(false)
const contractToTerminate = ref<VendorContract | null>(null)

function openTerminateModal(contract: VendorContract) {
  contractToTerminate.value = contract
  showTerminateModal.value = true
}

// ── Amendment (Pattern B — ADR-009) ───────────────────────────────
// "Modifier" only edits descriptive fields on non-active contracts (draft).
// Once active, the 7 frozen fields (amount/start_date/end_date/notice_days/
// contract_type/payment_frequency/auto_renewal) require a new amendment
// contract instead — see docs/adr/ADR-009-accounting-immutability.md.

const showAmendModal = ref(false)
const contractToAmend = ref<VendorContract | null>(null)
const showChainModal = ref(false)
const contractForChain = ref<VendorContract | null>(null)
const activatingId = ref('')

function openAmendModal(contract: VendorContract) {
  contractToAmend.value = contract
  showAmendModal.value = true
}

function openChainModal(contract: VendorContract) {
  contractForChain.value = contract
  showChainModal.value = true
}

async function activateContract(contract: VendorContract) {
  activatingId.value = contract.id

  try {
    await contractsApi.activate(contract.id)
    await reload()
  } catch (currentError: unknown) {
    error.value = toErrorMessage(currentError, t('contracts.activateError'))
  } finally {
    activatingId.value = ''
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName || '...' }}
          <CIconChevronRight />
          <span>{{ t('contracts.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canManage"
          variant="primary"
          size="sm"
          shrink="sm"
          :icon="CIconPlus"
          :title="t('contracts.create')"
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
            {{ t('page.eyebrows.contracts') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('contracts.title') }}
          </CHeading>
          <CMetaItem variant="sub">
            {{ t('contracts.count', { count: allContracts.length }) }}
          </CMetaItem>
        </header>

        <CReminderCard
          v-if="contractReminderRows.length > 0"
          :title="t('contracts.upcomingReminders')"
          :rows="contractReminderRows"
        >
          <template
            v-if="canManage"
            #actions="{ row }"
          >
            <CBtn
              variant="negative"
              size="sm"
              :title="t('contracts.reminders.acknowledge')"
              :loading="busyKey === `reminder:${row.id}`"
              :disabled="busyKey === `reminder:${row.id}`"
              :icon="CIconCurrency"
              @click="acknowledgeReminder(reminderViewsById.get(row.id)!)"
            />
          </template>
        </CReminderCard>

        <CTabs
          v-model="filter"
          variant="filter"
          :tabs="contractTabsList"
        >
          <!-- Toolbar: search + view mode toggle -->
          <div class="list-toolbar">
            <CFormInput
              v-model="search"
              type="search"
              :placeholder="t('contracts.searchPlaceholder')"
              class="list-toolbar__search"
            >
              <template #prefix>
                <CIconSearch class="icon-sm" />
              </template>
            </CFormInput>
            <div
              class="view-toggle"
              role="group"
              :aria-label="t('common.viewMode', 'Vue')"
            >
              <button
                :class="['view-toggle__btn', { 'view-toggle__btn--active': viewMode === 'table' }]"
                :aria-pressed="viewMode === 'table'"
                :aria-label="t('contracts.filters.viewTable')"
                @click="viewMode = 'table'"
              >
                <CIconList class="icon-sm" />
              </button>
              <button
                :class="['view-toggle__btn', { 'view-toggle__btn--active': viewMode === 'cards' }]"
                :aria-pressed="viewMode === 'cards'"
                :aria-label="t('contracts.filters.viewCards')"
                @click="viewMode = 'cards'"
              >
                <CIconGrid class="icon-sm" />
              </button>
            </div>
          </div>
        </CTabs>

        <!-- Filters toolbar: multi-selects -->
        <div
          v-if="contractTypeOptions.length > 0 || contractVendorOptions.length > 0"
          class="contracts-toolbar"
        >
          <CFormMultiSelect
            v-if="contractTypeOptions.length > 0"
            v-model="selectedTypes"
            :options="contractTypeOptions"
            :placeholder="t('common.type')"
            :aria-label="t('common.type')"
          />
          <CFormMultiSelect
            v-if="contractVendorOptions.length > 0"
            v-model="selectedVendors"
            :options="contractVendorOptions"
            :placeholder="t('contracts.form.vendorName')"
            :aria-label="t('contracts.form.vendorName')"
          />
          <CBtn
            v-if="hasActiveFilters"
            variant="ghost"
            size="sm"
            :icon="CIconX"
            :title="t('common.clearFilters')"
            :aria-label="t('common.clearFilters')"
            @click="clearFilters"
          />
        </div>

        <template
          v-if="contracts.length === 0"
        >
          <CEmptyState
            :title="t('contracts.empty')"
            size="page"
          >
            <template #actions>
              <CBtn
                v-if="canManage && ['active', ALL_FILTER].includes(filter)"
                variant="primary"
                :icon="CIconPlus"
                :title="t('contracts.create')"
                @click="openCreateModal"
              />
            </template>
          </CEmptyState>
        </template>

        <CCard
          v-if="viewMode === 'table'"
          class="p-0!"
        >
          <CDataTable
            :columns="contractTableColumns"
            :rows="contracts"
            row-key="id"
            @row-click="openContractDetail($event)"
          >
            <template #cell-contract_type="{ row }">
              <CBadge variant="default">
                {{ t(`contracts.types.${row.contract_type}`) }}
              </CBadge>
            </template>
            <template #cell-start_date="{ row }">
              {{ formatDate(row.start_date) }}
            </template>
            <template #cell-end_date="{ row }">
              {{ formatDate(row.end_date) }}
            </template>
            <template #cell-amount="{ row }">
              <span class="whitespace-nowrap">{{ formatCurrency(row.amount) }}</span>
            </template>
            <template #cell-status="{ row }">
              <CBadge
                :variant="statusVariant(effectiveStatus(row))"
                dot
              >
                {{ t(`contracts.statuses.${effectiveStatus(row)}`) }}
              </CBadge>
            </template>
          </CDataTable>
        </CCard>

        <div
          v-else-if="viewMode === 'cards'"
          class="contracts-grid"
        >
          <CCard
            v-for="contract in contracts"
            :key="contract.id"
            :shadow="effectiveStatus(contract) === CONTRACT_STATUS_ACTIVE"
            gap="sm"
            interactive
            :class="[{ 'outline outline-1 outline-outline-variant': effectiveStatus(contract) !== CONTRACT_STATUS_ACTIVE }]"
            @click="openContractDetail(contract)"
          >
            <div class="flex items-center gap-3 min-w-0">
              <CIconTile :class="contractIconTone(contract)">
                <component
                  :is="contractTypeIcon(contract.contract_type)"
                  aria-hidden="true"
                />
              </CIconTile>
              <h3 class="text-lg font-semibold text-on-surface m-0 leading-snug min-w-0">
                {{ contract.vendor_name }}
              </h3>
              <div class="flex flex-wrap gap-x-2 gap-y-1">
                <CBadge variant="default">
                  {{ t(`contracts.types.${contract.contract_type}`) }}
                </CBadge>
                <CBadge
                  :variant="statusVariant(effectiveStatus(contract))"
                  dot
                >
                  {{ t(`contracts.statuses.${effectiveStatus(contract)}`) }}
                </CBadge>
              </div>
            </div>
            <div class="flex flex-col gap-3">
              <p class="body-xs">
                {{ contract.subject }}
              </p>

              <div class="contract-details">
                <div class="detail-row">
                  <span class="detail-label">{{ t('common.startDate') }}</span>
                  <span>{{ formatDate(contract.start_date) }}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">{{ t('common.endDate') }}</span>
                  <span>{{ formatDate(contract.end_date) }}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">{{ t('common.amount') }}</span>
                  <span>{{ formatCurrency(contract.amount) }}<template v-if="contract.payment_frequency"> / {{ freqLabel(contract.payment_frequency) }}</template></span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">{{ t('contracts.autoRenewal') }}</span>
                  <span>
                    {{
                      contract.auto_renewal
                        ? t('contracts.autoRenewalAuto', { days: contract.notice_days ?? '—' })
                        : t('contracts.autoRenewalManual')
                    }}
                  </span>
                </div>
              </div>

              <div
                v-if="pendingRemindersByContract.get(contract.id)?.length"
                class="contract-reminders"
              >
                <CBadge
                  v-for="reminder in pendingRemindersByContract.get(contract.id)"
                  :key="reminder.id"
                  variant="pending"
                >
                  🔔 {{ t(`contracts.reminderTypes.${reminder.reminder_type}`) }} — {{ formatDate(reminder.targetDate) }}
                </CBadge>
              </div>

              <div
                v-if="contract.vendor_email || contract.vendor_phone"
                class="flex flex-col gap-1"
              >
                <a
                  v-if="contract.vendor_email"
                  :href="`mailto:${contract.vendor_email}`"
                  class="inline-flex items-center gap-1 text-xs text-primary no-underline hover:underline"
                ><span aria-hidden="true">✉️</span>{{ contract.vendor_email }}</a>
                <a
                  v-if="contract.vendor_phone"
                  :href="`tel:${contract.vendor_phone}`"
                  class="inline-flex items-center gap-1 text-xs text-primary no-underline hover:underline"
                ><span aria-hidden="true">📞</span>{{ contract.vendor_phone }}</a>
              </div>

              <div
                v-if="(contract.document_count ?? 0) > 0"
                class="flex flex-wrap gap-x-2 gap-y-1 text-2xs leading-snug"
              >
                <button
                  class="inline-flex items-center gap-1 bg-transparent border-none p-0 cursor-pointer text-xs hover:text-primary"
                  @click="openContractDocuments(contract)"
                >
                  <CIconFile />
                  {{ t('contracts.documents.count', contract.document_count ?? 0) }}
                </button>
              </div>

              <CCardActions v-if="canManage">
                <CBtn
                  v-if="effectiveStatus(contract) === CONTRACT_STATUS_DRAFT"
                  variant="primary"
                  size="sm"
                  :icon="CIconCheck"
                  :title="t('contracts.activate')"
                  :loading="activatingId === contract.id"
                  :disabled="activatingId === contract.id"
                  @click="activateContract(contract)"
                />
                <CBtn
                  variant="ghost"
                  size="sm"
                  :icon="CIconEdit"
                  :title="t('common.edit')"
                  @click="openEditModal(contract)"
                />
                <CBtn
                  v-if="effectiveStatus(contract) === CONTRACT_STATUS_ACTIVE"
                  variant="ghost"
                  size="sm"
                  :icon="CIconEdit"
                  :title="t('contracts.amend')"
                  @click="openAmendModal(contract)"
                />
                <CBtn
                  v-if="contract.predecessor_id || effectiveStatus(contract) !== CONTRACT_STATUS_DRAFT"
                  variant="ghost"
                  size="sm"
                  :icon="CIconHistory"
                  :title="t('contracts.history')"
                  @click="openChainModal(contract)"
                />
                <CBtn
                  v-if="effectiveStatus(contract) === CONTRACT_STATUS_ACTIVE"
                  variant="danger"
                  size="sm"
                  :icon="CIconArchive"
                  :title="t('contracts.terminate')"
                  @click="openTerminateModal(contract)"
                />
              </CCardActions>
              <CCardActions v-else>
                <CBtn
                  v-if="contract.predecessor_id || effectiveStatus(contract) !== CONTRACT_STATUS_DRAFT"
                  variant="ghost"
                  size="sm"
                  :icon="CIconHistory"
                  :title="t('contracts.history')"
                  @click="openChainModal(contract)"
                />
              </CCardActions>
            </div>
          </CCard>
        </div>
      </template>
    </div>
  </div>

  <ContractFormModal
    v-model:open="showFormModal"
    :contract="editingContract"
    @saved="reload"
  />

  <CLinkedDocumentsModal
    :open="!!contractDocumentsTarget"
    :title="t('contracts.documents.title', { name: contractDocumentsTarget?.vendor_name ?? '' })"
    :documents="contractDocuments"
    :loading="loadingContractDocuments"
    :empty-message="t('contracts.documents.empty')"
    @update:open="value => !value && (contractDocumentsTarget = null)"
    @download="downloadContractDocument"
  />

  <TerminateContractModal
    :open="showTerminateModal"
    :contract="contractToTerminate"
    @update:open="showTerminateModal = $event"
    @terminated="reload"
  />

  <AmendContractModal
    :open="showAmendModal"
    :contract="contractToAmend"
    @update:open="showAmendModal = $event"
    @amended="reload"
  />

  <AmendmentChainModal
    :open="showChainModal"
    :contract="contractForChain"
    @update:open="showChainModal = $event"
  />
</template>

<style scoped>

.contracts-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: var(--space-4); }

.list-toolbar {
  display: flex;
  gap: var(--space-3);
  align-items: center;
  margin-top: var(--space-3);
}

.list-toolbar__search {
  flex: 1;
}

.view-toggle {
  display: flex;
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-xl);
  overflow: hidden;
  flex-shrink: 0;
}

.view-toggle__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: var(--color-surface);
  border: none;
  cursor: pointer;
  color: var(--color-on-surface-variant);
  transition: background 150ms ease-out, color 150ms ease-out;
}

.view-toggle__btn + .view-toggle__btn {
  border-left: 1px solid var(--color-outline-variant);
}

.view-toggle__btn:hover {
  background: var(--color-surface-container-low);
  color: var(--color-on-surface);
}

.view-toggle__btn--active {
  background: var(--color-surface-container);
  color: var(--color-on-surface);
}

.contracts-toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-2); margin-bottom: var(--space-4); }

.contract-details { display: flex; flex-direction: column; gap: var(--space-1); }
.detail-row { display: flex; justify-content: space-between; font-size: var(--text-sm); }
.detail-label { color: var(--color-on-surface-variant); }

.contract-reminders { display: flex; flex-wrap: wrap; gap: 6px; }

</style>
