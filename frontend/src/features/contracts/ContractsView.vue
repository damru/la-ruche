<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CBadge from '@/components/atoms/CBadge.vue'
import CIconBolt from '@/components/atoms/CIconBolt.svg'
import CIconDroplet from '@/components/atoms/CIconDroplet.svg'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CLinkedDocumentsModal from '@/components/organisms/CLinkedDocumentsModal.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CReminderCard from '@/components/organisms/CReminderCard.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconFile from '@/components/atoms/CIconFile.svg'
import CIconLeaf from '@/components/atoms/CIconLeaf.svg'
import CIconLayers from '@/components/atoms/CIconLayers.svg'
import CIconSettings from '@/components/atoms/CIconSettings.svg'
import CIconShield from '@/components/atoms/CIconShield.svg'
import CIconSparkles from '@/components/atoms/CIconSparkles.svg'
import CIconWrench from '@/components/atoms/CIconWrench.svg'
import { buildingApi, contractsApi, documentsApi } from '@/api'
import ContractFormModal from '@/features/contracts/ContractFormModal.vue'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { ContractReminder, ContractStatus, ContractType, Document, VendorContract } from '@/types/domain'

interface ContractReminderView extends ContractReminder {
  targetDate: string
  urgency: 'high' | 'medium' | 'low'
  daysLabel: string
}

const { t } = useI18n()
const { formatCurrency, formatDate, daysUntil } = useFormatters()

const filter = ref<'all' | 'active' | 'expired'>('all')
const search = ref('')
const buildingName = ref('')
const allContracts = ref<VendorContract[]>([])
const allReminders = ref<ContractReminder[]>([])
const editingContract = ref<VendorContract | null>(null)
const showFormModal = ref(false)
const busyKey = ref('')
const contractDocumentsTarget = ref<VendorContract | null>(null)
const contractDocuments = ref<Document[]>([])
const loadingContractDocuments = ref(false)

async function loadData() {
  const [buildingResponse, contractsResponse, remindersResponse] = await Promise.all([
    buildingApi.get(),
    contractsApi.list(),
    contractsApi.listAllReminders(),
  ])

  buildingName.value = buildingResponse.data.name
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

function todayIso(): string {
  const today = new Date()
  const year = today.getFullYear()
  const month = String(today.getMonth() + 1).padStart(2, '0')
  const day = String(today.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function effectiveStatus(contract: VendorContract): ContractStatus {
  if (contract.status === 'terminated') return 'terminated'
  if (contract.status === 'expired') return 'expired'
  if (contract.end_date && contract.end_date < todayIso()) return 'expired'
  return 'active'
}

function reminderUrgency(dateStr: string): 'high' | 'medium' | 'low' {
  const d = daysUntil(dateStr)
  return d <= 15 ? 'high' : d <= 45 ? 'medium' : 'low'
}

function reminderTone(urgency: 'high' | 'medium' | 'low'): 'err' | 'warn' | 'muted' {
  if (urgency === 'high') return 'err'
  if (urgency === 'medium') return 'warn'
  return 'muted'
}

function statusVariant(status: ContractStatus): 'open' | 'pending' | 'closed' | 'default' {
  if (status === 'active') return 'open'
  if (status === 'terminated' || status === 'expired') return 'closed'
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
  if (effectiveStatus(contract) !== 'active') return 'tone--disabled'
  return contractTypeTone(contract.contract_type)
}

const contracts = computed(() => {
  const query = search.value.trim().toLowerCase()

  return [...allContracts.value]
    .filter((contract) => {
      const currentStatus = effectiveStatus(contract)
      if (filter.value === 'active' && currentStatus !== 'active') return false
      if (filter.value === 'expired' && currentStatus === 'active') return false

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

const contractCounts = computed(() =>
  allContracts.value.reduce(
    (counts, contract) => {
      counts.all += 1
      if (effectiveStatus(contract) === 'active') {
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
  { key: 'all', label: t('common.all'), count: contractCounts.value.all },
  { key: 'active', label: t('contracts.filters.active'), count: contractCounts.value.active },
  { key: 'expired', label: t('contracts.filters.expired'), count: contractCounts.value.expired },
])

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

function setFilter(nextFilter: 'all' | 'active' | 'expired') {
  filter.value = nextFilter
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
const terminatingContract = ref(false)
const terminateFeedback = useFormFeedback()
const { schedule: scheduleTerminateClose, forceExpire: forceTerminateClose } = useAutoClose()
watch(showTerminateModal, (open) => { if (!open) forceTerminateClose() })

function openTerminateModal(contract: VendorContract) {
  contractToTerminate.value = contract
  terminateFeedback.reset()
  showTerminateModal.value = true
}

async function confirmTerminateContract() {
  if (terminateFeedback.status.value === 'success') { forceTerminateClose(); return }
  if (!contractToTerminate.value) return
  terminatingContract.value = true
  terminateFeedback.reset()
  try {
    await contractsApi.terminate(contractToTerminate.value.id)
    terminateFeedback.setSuccess(t('contracts.terminateSuccess'))
    await reload()
    scheduleTerminateClose(() => { showTerminateModal.value = false })
  } catch (currentError: unknown) {
    terminateFeedback.setError(toErrorMessage(currentError, t('contracts.terminateError')))
  } finally {
    terminatingContract.value = false
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
          variant="primary"
          size="sm"
          @click="openCreateModal"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('contracts.create') }}</span>
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
        <header class="view-header">
          <CEyebrow>
            {{ t('page.eyebrows.contracts') }}
          </CEyebrow>
          <h1 class="display-md">
            {{ t('contracts.title') }}
          </h1>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('contracts.count', { count: allContracts.length }) }}
          </p>
        </header>

        <CReminderCard
          v-if="contractReminderRows.length > 0"
          :title="t('contracts.upcomingReminders')"
          :rows="contractReminderRows"
        >
          <template #actions="{ row }">
            <CBtn
              variant="secondary"
              size="sm"
              :loading="busyKey === `reminder:${row.id}`"
              :disabled="busyKey === `reminder:${row.id}`"
              @click="acknowledgeReminder(reminderViewsById.get(row.id)!)"
            >
              {{ t('contracts.reminders.acknowledge') }}
            </CBtn>
          </template>
        </CReminderCard>

        <div class="contracts-toolbar">
          <div class="tabs">
            <button
              v-for="opt in contractTabs"
              :key="opt.key"
              :class="['tab', { 'tab--active': filter === opt.key }]"
              @click="setFilter(opt.key as 'all' | 'active' | 'expired')"
            >
              {{ opt.label }}
              <span class="tab-count">{{ opt.count }}</span>
            </button>
          </div>

          <CFormInput
            v-model="search"
            type="search"
            :placeholder="t('contracts.searchPlaceholder')"
          >
            <template #prefix>
              <CIconSearch />
            </template>
          </CFormInput>
        </div>

        <CCard
          v-if="contracts.length === 0"
          class="contracts-empty"
        >
          {{ t('contracts.empty') }}
        </CCard>

        <div
          v-else
          class="contracts-grid"
        >
          <CCard
            v-for="contract in contracts"
            :key="contract.id"
            :shadow="effectiveStatus(contract) === 'active'"
            :class="['activity-card', { 'card--terminated': effectiveStatus(contract) !== 'active' }]"
          >
            <div class="activity-card__header-row activity-card__header-row--with-icon">
              <div :class="['activity-card__icon', contractIconTone(contract)]">
                <component
                  :is="contractTypeIcon(contract.contract_type)"
                  aria-hidden="true"
                />
              </div>
              <h3 class="activity-card__title">
                {{ contract.vendor_name }}
              </h3>
              <div class="activity-card__chips">
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
            <div class="activity-card__stack">
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
                class="activity-card__contacts"
              >
                <a
                  v-if="contract.vendor_email"
                  :href="`mailto:${contract.vendor_email}`"
                  class="activity-card__line activity-card__contact-link"
                ><span aria-hidden="true">✉️</span>{{ contract.vendor_email }}</a>
                <a
                  v-if="contract.vendor_phone"
                  :href="`tel:${contract.vendor_phone}`"
                  class="activity-card__line activity-card__contact-link"
                ><span aria-hidden="true">📞</span>{{ contract.vendor_phone }}</a>
              </div>

              <div
                v-if="(contract.document_count ?? 0) > 0"
                class="activity-card__documents"
              >
                <button
                  class="activity-card__line activity-card__documents-button"
                  @click="openContractDocuments(contract)"
                >
                  <CIconFile />
                  {{ t('contracts.documents.count', contract.document_count ?? 0) }}
                </button>
              </div>

              <div class="activity-card__actions">
                <CBtn
                  variant="ghost"
                  size="sm"
                  @click="openEditModal(contract)"
                >
                  <CIconEdit />
                  {{ t('common.edit') }}
                </CBtn>
                <CBtn
                  v-if="effectiveStatus(contract) === 'active'"
                  variant="danger"
                  size="sm"
                  @click="openTerminateModal(contract)"
                >
                  <CIconArchive />
                  {{ t('contracts.terminate') }}
                </CBtn>
              </div>
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

  <!-- Terminate contract modal -->
  <CModal
    :open="showTerminateModal"
    :title="t('contracts.terminateTitle')"
    size="sm"
    @update:open="showTerminateModal = $event"
  >
    <p class="body-md">
      {{ t('contracts.terminateMessage', { name: contractToTerminate?.vendor_name ?? '' }) }}
    </p>
    <template #footer>
      <CFormFeedback
        :status="terminateFeedback.status.value"
        :message="terminateFeedback.message.value"
      />
      <CBtn
        variant="ghost"
        @click="showTerminateModal = false"
      >
        {{ t('common.cancel') }}
      </CBtn>
      <CBtn
        variant="danger"
        :loading="terminatingContract"
        :closing="terminateFeedback.status.value === 'success'"
        @click="confirmTerminateContract"
      >
        {{ t('contracts.terminate') }}
      </CBtn>
    </template>
  </CModal>
</template>

<style scoped>

.contracts-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  flex-wrap: wrap;
}

.contracts-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: var(--space-4); }

.contracts-empty {
  color: var(--color-on-surface-variant);
}

.contract-details { display: flex; flex-direction: column; gap: var(--space-1); }
.detail-row { display: flex; justify-content: space-between; font-size: var(--text-sm); }
.detail-label { color: var(--color-on-surface-variant); }

.contract-reminders { display: flex; flex-wrap: wrap; gap: 6px; }

</style>
