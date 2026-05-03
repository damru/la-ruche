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
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconFile from '@/components/atoms/CIconFile.svg'
import CIconLeaf from '@/components/atoms/CIconLeaf.svg'
import CIconLayers from '@/components/atoms/CIconLayers.svg'
import CIconSettings from '@/components/atoms/CIconSettings.svg'
import CIconShield from '@/components/atoms/CIconShield.svg'
import CIconSparkles from '@/components/atoms/CIconSparkles.svg'
import CIconWrench from '@/components/atoms/CIconWrench.svg'
import { buildingApi, contractsApi } from '@/api'
import ContractFormModal from '@/features/contracts/ContractFormModal.vue'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { ContractReminder, ContractStatus, ContractType, VendorContract } from '@/types/domain'

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

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
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
          <CIconChevronRight
            width="14"
            height="14"
          />
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
        <div class="view-header">
          <div>
            <h1 class="display-md">
              {{ t('contracts.title') }}
            </h1>
            <p
              class="body-md"
              style="color:var(--color-on-surface-variant)"
            >
              {{ t('contracts.count', { count: allContracts.length }) }}
            </p>
          </div>
        </div>

        <CCard padding="none">
          <div class="section-head">
            <h2 class="section-title">
              {{ t('contracts.upcomingReminders') }}
            </h2>
          </div>
          <div
            v-if="reminderViews.length === 0"
            class="reminder-empty"
          >
            {{ t('contracts.reminders.empty') }}
          </div>
          <div
            v-else
            class="reminder-list"
          >
            <div
              v-for="reminder in reminderViews"
              :key="reminder.id"
              class="reminder-row"
              :class="`reminder-row--${reminder.urgency}`"
            >
              <div class="reminder-dot"></div>
              <div class="reminder-info">
                <span class="reminder-contract">{{ contractsById.get(reminder.contract_id)?.vendor_name }}</span>
                <span class="reminder-type">
                  {{ t(`contracts.reminderTypes.${reminder.reminder_type}`) }}
                </span>
              </div>
              <span class="reminder-date">{{ formatDate(reminder.targetDate) }}</span>
              <span
                class="reminder-days"
                :class="`urgency-${reminder.urgency}`"
              >
                {{ reminder.daysLabel }}
              </span>
              <CBtn
                variant="secondary"
                size="sm"
                :loading="busyKey === `reminder:${reminder.id}`"
                :disabled="busyKey === `reminder:${reminder.id}`"
                @click="acknowledgeReminder(reminder)"
              >
                {{ t('contracts.reminders.acknowledge') }}
              </CBtn>
            </div>
          </div>
        </CCard>

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

          <div class="search-bar">
            <CIconSearch class="search-icon" />
            <input
              v-model="search"
              type="search"
              :aria-label="t('contracts.searchPlaceholder')"
              :placeholder="t('contracts.searchPlaceholder')"
              class="search-input"
            />
          </div>
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
            :class="['contract-card', { 'card--terminated': effectiveStatus(contract) !== 'active' }]"
          >
            <div class="activity-card__head">
              <div class="activity-card__header-row activity-card__header-row--with-icon">
                <div :class="['activity-card__icon', 'contract-card__icon', contractIconTone(contract)]">
                  <component
                    :is="contractTypeIcon(contract.contract_type)"
                    class="contract-card__type-icon"
                    aria-hidden="true"
                  />
                </div>
                <h3 class="activity-card__title vendor-name">
                  {{ contract.vendor_name }}
                </h3>
                <CBadge
                  :variant="statusVariant(effectiveStatus(contract))"
                  dot
                  class="activity-card__status"
                >
                  {{ t(`contracts.statuses.${effectiveStatus(contract)}`) }}
                </CBadge>
              </div>

              <div class="activity-card__chips">
                <CBadge
                  variant="default"
                >
                  {{ t(`contracts.types.${contract.contract_type}`) }}
                </CBadge>
              </div>
            </div>

            <p class="body-sm contract-subject">
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
              class="contract-contacts"
            >
              <a
                v-if="contract.vendor_email"
                :href="`mailto:${contract.vendor_email}`"
                class="contact-link"
              >{{ contract.vendor_email }}</a>
              <a
                v-if="contract.vendor_phone"
                :href="`tel:${contract.vendor_phone}`"
                class="contact-link"
              >{{ contract.vendor_phone }}</a>
            </div>

            <div class="contract-card__bottom">
              <div class="contract-card__footer">
                <span class="contract-card__detail">
                  <CIconFile
                    width="14"
                    height="14"
                  />
                  {{ t('contracts.documents.count', contract.document_count ?? 0) }}
                </span>
              </div>

              <div class="contract-actions">
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
.section-head { padding: var(--space-4) var(--space-5) 0; }

.reminder-empty {
  padding: var(--space-4) var(--space-5) var(--space-5);
  color: var(--color-on-surface-variant);
}

.reminder-list { padding: var(--space-3) var(--space-5) var(--space-4); display: flex; flex-direction: column; gap: var(--space-2); }
.reminder-row { display: flex; align-items: center; gap: var(--space-3); padding: var(--space-2) var(--space-3); border-radius: var(--radius-sm); }
.reminder-row--high { background: var(--color-error-container); }
.reminder-row--medium { background: var(--color-warning-container); }
.reminder-dot { width: 8px; height: 8px; border-radius: 50%; background: currentColor; flex-shrink: 0; }
.reminder-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.reminder-contract { font-size: var(--text-sm); font-weight: var(--fw-medium); }
.reminder-type { font-size: var(--text-2xs); color: var(--color-on-surface-variant); }
.reminder-date { font-size: var(--text-sm); }
.reminder-days { font-size: var(--text-xs); font-weight: var(--fw-semibold); flex-shrink: 0; }
.urgency-high { color: var(--color-error); }
.urgency-medium { color: var(--color-warning); }
.urgency-low { color: var(--color-on-surface-variant); }

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

.contract-card { display: flex; flex-direction: column; gap: var(--space-3); }
.contract-card__icon {
  background: var(--tone-bg, var(--color-secondary-container));
  color: var(--tone-fg, var(--color-primary));
}
.contract-card__type-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}
.contract-card__bottom {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  margin-top: auto;
  padding-top: var(--space-3);
}
.contract-card__footer { display: flex; flex-wrap: wrap; gap: var(--space-3); }
.contract-card__detail {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: var(--text-sm);
  color: var(--color-on-surface-variant);
}
.contract-actions {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}
.vendor-name { margin: 0; }
.contract-subject {
  margin: 0;
  color: var(--color-on-surface-variant);
}

.contract-details { display: flex; flex-direction: column; gap: var(--space-1); }
.detail-row { display: flex; justify-content: space-between; font-size: var(--text-sm); }
.detail-label { color: var(--color-on-surface-variant); }

.contract-reminders { display: flex; flex-wrap: wrap; gap: 6px; }

.contract-contacts { display: flex; flex-direction: column; gap: var(--space-1); }
.contact-link { font-size: var(--text-xs); color: var(--color-primary); text-decoration: none; }
.contact-link:hover { text-decoration: underline; }

@media (max-width: 900px) {
  .reminder-row {
    flex-wrap: wrap;
  }

  .contract-actions {
    justify-content: flex-start;
  }
}

</style>
