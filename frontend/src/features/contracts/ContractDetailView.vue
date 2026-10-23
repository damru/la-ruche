<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { toErrorMessage, usePageLoader } from '@/composables/usePageLoader'
import { useRole } from '@/composables/useRole'
import CBadge from '@/components/atoms/CBadge.vue'
import CIconTile from '@/components/atoms/CIconTile.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CCard from '@/components/molecules/CCard.vue'
import CCardActions from '@/components/molecules/CCardActions.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CSectionHeader from '@/components/molecules/CSectionHeader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CIconArrowLeft from '@/components/atoms/CIconArrowLeft.svg'
import CIconBolt from '@/components/atoms/CIconBolt.svg'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconDroplet from '@/components/atoms/CIconDroplet.svg'
import CIconFile from '@/components/atoms/CIconFile.svg'
import CIconLayers from '@/components/atoms/CIconLayers.svg'
import CIconLeaf from '@/components/atoms/CIconLeaf.svg'
import CIconSettings from '@/components/atoms/CIconSettings.svg'
import CIconShield from '@/components/atoms/CIconShield.svg'
import CIconSparkles from '@/components/atoms/CIconSparkles.svg'
import CIconWrench from '@/components/atoms/CIconWrench.svg'
import CIconCheck from '@/components/atoms/CIconCheck.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconArchive from '@/components/atoms/CIconArchive.svg'
import { contractsApi, documentsApi } from '@/api'
import { effectiveContractStatus } from '@/utils/contracts'
import { useBuildingStore } from '@/stores/building'
import ContractFormModal from '@/features/contracts/ContractFormModal.vue'
import AmendContractModal from '@/features/contracts/AmendContractModal.vue'
import TerminateContractModal from '@/features/contracts/TerminateContractModal.vue'
import type { ContractReminder, ContractStatus, ContractType, Document, VendorContract } from '@/types/domain'

const { t } = useI18n()
const { formatCurrency, formatDate, formatFileSize } = useFormatters()
const { hasAnyRole } = useRole()
const route = useRoute()
const router = useRouter()

const contractId = computed(() => route.params.id as string)
const buildingName = ref('')
const contract = ref<VendorContract | null>(null)
const amendmentChain = ref<VendorContract[]>([])
const documents = ref<Document[]>([])
const reminders = ref<ContractReminder[]>([])
const activating = ref(false)

const showFormModal = ref(false)
const showAmendModal = ref(false)
const showTerminateModal = ref(false)

const canManage = computed(() => hasAnyRole(['manager']))

async function loadData() {
  const id = contractId.value
  const [building, contractRes, chainRes, documentsRes, remindersRes] = await Promise.all([
    useBuildingStore().fetch(),
    contractsApi.get(id),
    contractsApi.amendmentChain(id),
    contractsApi.listDocuments(id),
    contractsApi.listReminders(id),
  ])
  buildingName.value = building.name
  contract.value = contractRes.data
  amendmentChain.value = chainRes.data
  documents.value = documentsRes.data
  reminders.value = remindersRes.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

const currentStatus = computed<ContractStatus>(() =>
  contract.value ? effectiveContractStatus(contract.value) : 'draft',
)

function chainLevel(item: VendorContract): 2 | 3 {
  return contract.value && item.id === contract.value.id ? 2 : 3
}

function statusVariant(status: ContractStatus): 'open' | 'pending' | 'closed' | 'default' {
  if (status === 'active') return 'open'
  if (status === 'draft') return 'pending'
  if (status === 'terminated' || status === 'expired') return 'closed'
  return 'default'
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

function freqLabel(freq?: string): string {
  return freq ? t(`contracts.frequencies.${freq}`) : '—'
}

function chainLabel(index: number): string {
  return index === 0
    ? t('contracts.chainOrigin')
    : t('contracts.chainAmendment', { index })
}

function reminderStatusLabel(status: string): string {
  if (status === 'pending') return t('common.pending')
  return t(`contracts.filters.reminderStatuses.${status}`)
}

async function downloadDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch (currentError: unknown) {
    error.value = toErrorMessage(currentError, t('contracts.documents.downloadError'))
  }
}

async function activateContract() {
  if (!contract.value) return
  activating.value = true
  try {
    await contractsApi.activate(contract.value.id)
    await reload()
  } catch (currentError: unknown) {
    error.value = toErrorMessage(currentError, t('contracts.activateError'))
  } finally {
    activating.value = false
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
          <RouterLink
            :to="{ name: 'contracts' }"
            class="topbar-breadcrumb-link"
          >
            {{ t('contracts.title') }}
          </RouterLink>
          <CIconChevronRight />
          <span>{{ contract?.vendor_name ?? '…' }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="ghost"
          size="sm"
          :icon="CIconArrowLeft"
          :title="t('common.back')"
          @click="router.push({ name: 'contracts' })"
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
      <template v-else-if="contract">
        <CAlert
          v-if="currentStatus === 'draft'"
          variant="warning"
          :title="t('contracts.draftBanner')"
        >
          <CBtn
            v-if="canManage"
            variant="primary"
            size="sm"
            :icon="CIconCheck"
            :title="t('contracts.activate')"
            :loading="activating"
            :disabled="activating"
            @click="activateContract"
          />
        </CAlert>

        <!-- Header card -->
        <CCard class="contract-detail-card">
          <div class="contract-header">
            <CIconTile
              size="lg"
              :class="contractTypeTone(contract.contract_type)"
            >
              <component
                :is="contractTypeIcon(contract.contract_type)"
                aria-hidden="true"
              />
            </CIconTile>
            <div class="contract-header-info">
              <CEyebrow>
                {{ t('page.eyebrows.contractDetail') }}
              </CEyebrow>
              <CHeading
                type="header"
                class="mb-2"
              >
                {{ contract.vendor_name }}
              </CHeading>
              <div class="contract-badges">
                <CBadge variant="default">
                  {{ t(`contracts.types.${contract.contract_type}`) }}
                </CBadge>
                <CBadge
                  :variant="statusVariant(currentStatus)"
                  dot
                >
                  {{ t(`contracts.statuses.${currentStatus}`) }}
                </CBadge>
              </div>
            </div>
            <CCardActions v-if="canManage">
              <CBtn
                variant="ghost"
                size="sm"
                :icon="CIconEdit"
                :title="t('common.edit')"
                @click="showFormModal = true"
              />
              <CBtn
                v-if="currentStatus === 'active'"
                variant="ghost"
                size="sm"
                :icon="CIconEdit"
                :title="t('contracts.amend')"
                @click="showAmendModal = true"
              />
              <CBtn
                v-if="currentStatus === 'active'"
                variant="danger"
                size="sm"
                :icon="CIconArchive"
                :title="t('contracts.terminate')"
                @click="showTerminateModal = true"
              />
            </CCardActions>
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
            <div
              v-if="contract.notice_days"
              class="detail-row"
            >
              <span class="detail-label">{{ t('contracts.noticePeriod') }}</span>
              <span>{{ contract.notice_days }}</span>
            </div>
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
        </CCard>

        <!-- Amendment chain -->
        <CSectionHeader>
          <CHeading type="section">
            {{ t('contracts.amendments') }}
          </CHeading>
        </CSectionHeader>
        <div
          v-if="amendmentChain.length === 0"
          class="py-4 text-center text-md text-on-surface-variant"
        >
          {{ t('contracts.historyEmpty') }}
        </div>
        <div
          v-else
          class="chain-list"
        >
          <CCard
            v-for="(item, index) in amendmentChain"
            :key="item.id"
            :level="chainLevel(item)"
            class="chain-item"
            :class="{ 'chain-item--current': contract && item.id === contract.id }"
          >
            <div class="chain-item-row">
              <div class="chain-item-badges">
                <CBadge variant="default">
                  {{ chainLabel(index) }}
                </CBadge>
                <CBadge
                  :variant="statusVariant(item.status)"
                  dot
                >
                  {{ t(`contracts.statuses.${item.status}`) }}
                </CBadge>
              </div>
              <span class="body-xs chain-item-meta">{{ formatDate(item.start_date) }} → {{ formatDate(item.end_date) }}</span>
            </div>
            <span class="title-sm">{{ item.vendor_name }} — {{ item.subject }}</span>
          </CCard>
        </div>

        <!-- Linked documents -->
        <CSectionHeader>
          <CHeading type="section">
            {{ t('contracts.documents.titleInline') }}
          </CHeading>
        </CSectionHeader>
        <CCard>
          <p class="body-xs text-on-surface-variant contracts-documents-hint">
            {{ t('contracts.documents.hint') }}
          </p>
          <div
            v-if="documents.length === 0"
            class="py-4 text-center text-md text-on-surface-variant"
          >
            {{ t('contracts.documents.empty') }}
          </div>
          <div
            v-else
            class="documents-list"
          >
            <div
              v-for="document in documents"
              :key="document.id"
              class="document-row"
            >
              <div class="document-meta">
                <div class="document-name">
                  <CIconFile class="icon-sm" />
                  <span>{{ document.original_name }}</span>
                </div>
                <div class="document-details">
                  <span>{{ formatDate(document.created_at) }}</span>
                  <span>·</span>
                  <span>{{ formatFileSize(document.size_bytes) }}</span>
                </div>
              </div>
              <CBtn
                variant="ghost"
                size="sm"
                :icon="CIconDownload"
                :title="t('common.download')"
                @click="downloadDocument(document.id)"
              />
            </div>
          </div>
        </CCard>

        <!-- Reminders -->
        <CSectionHeader>
          <CHeading type="section">
            {{ t('contracts.remindersTitle') }}
          </CHeading>
        </CSectionHeader>
        <div
          v-if="reminders.length === 0"
          class="py-4 text-center text-md text-on-surface-variant"
        >
          {{ t('contracts.remindersEmpty') }}
        </div>
        <div
          v-else
          class="reminders-list"
        >
          <CCard
            v-for="reminder in reminders"
            :key="reminder.id"
            :level="3"
            class="reminder-row"
          >
            <CBadge variant="default">
              {{ t(`contracts.reminderTypes.${reminder.reminder_type}`) }}
            </CBadge>
            <span class="body-sm">{{ formatDate(reminder.reminder_date) }}</span>
            <CBadge
              :variant="reminder.status === 'acknowledged' ? 'valid' : 'pending'"
              dot
            >
              {{ reminderStatusLabel(reminder.status) }}
            </CBadge>
          </CCard>
        </div>
      </template>
    </div>
  </div>

  <ContractFormModal
    v-model:open="showFormModal"
    :contract="contract"
    @saved="reload"
  />

  <AmendContractModal
    :open="showAmendModal"
    :contract="contract"
    @update:open="showAmendModal = $event"
    @amended="reload"
  />

  <TerminateContractModal
    :open="showTerminateModal"
    :contract="contract"
    @update:open="showTerminateModal = $event"
    @terminated="reload"
  />
</template>

<style scoped>
.contract-detail-card { margin-bottom: var(--space-6); }

.contract-header { display: flex; align-items: flex-start; gap: var(--space-4); margin-bottom: var(--space-4); }
.contract-header-info { flex: 1; min-width: 0; }
.contract-badges { display: flex; gap: var(--space-1); flex-wrap: wrap; }
.contract-subject { color: var(--color-on-surface-variant); margin: 0 0 var(--space-4); }

.contract-details { display: flex; flex-direction: column; gap: var(--space-1); margin-bottom: var(--space-3); }
.detail-row { display: flex; justify-content: space-between; font-size: var(--text-sm); }
.detail-label { color: var(--color-on-surface-variant); }

.contract-contacts { display: flex; gap: var(--space-4); flex-wrap: wrap; }
.contact-link { color: var(--color-primary); font-size: var(--text-sm); text-decoration: none; }
.contact-link:hover { text-decoration: underline; }

.chain-list { display: flex; flex-direction: column; gap: var(--space-2); margin-bottom: var(--space-6); }
.chain-item-row { display: flex; align-items: center; justify-content: space-between; gap: var(--space-2); margin-bottom: var(--space-1); }
.chain-item-badges { display: flex; gap: var(--space-1); flex-wrap: wrap; }
.chain-item-meta { color: var(--color-on-surface-variant); white-space: nowrap; }
.chain-item--current { outline: 1px solid var(--color-primary); }

.contracts-documents-hint { margin: 0 0 var(--space-3); }
.documents-list { display: flex; flex-direction: column; gap: var(--space-2); }
.document-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-xl);
  background: var(--color-surface-container-low);
}
.document-meta { min-width: 0; display: flex; flex-direction: column; gap: var(--space-1); }
.document-name { display: inline-flex; align-items: center; gap: var(--space-2); min-width: 0; font-weight: var(--fw-medium); }
.document-name span { overflow: clip; text-overflow: ellipsis; white-space: nowrap; }
.document-details { display: inline-flex; align-items: center; gap: var(--space-1); color: var(--color-on-surface-variant); font-size: var(--text-xs); }

.reminders-list { display: flex; flex-direction: column; gap: var(--space-2); }
.reminder-row { flex-direction: row; align-items: center; justify-content: space-between; gap: var(--space-3); }
</style>
