<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import { maintenanceApi } from '@/api'
import type { LogbookContractSummary, LogbookSummary, MaintenanceEntry } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatCurrency } = useFormatters()

type PeriodFilter = 'month' | 'quarter' | 'semester' | 'year' | 'all'

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
const logbookSummary = ref<LogbookSummary | null>(null)
const allEntries = ref<MaintenanceEntry[]>([])
const search = ref('')
const periodFilter = ref<PeriodFilter>('quarter')
const periodAnchor = ref(startOfPeriod(new Date(), 'quarter'))

// ── Entry form modal ───────────────────────────────────────────────
const showEntryModal = ref(false)
const entryFeedback = useFormFeedback()
const savingEntry = ref(false)
const { schedule: scheduleEntryClose, forceExpire: forceEntryClose } = useAutoClose()
watch(showEntryModal, (open) => { if (!open) forceEntryClose() })

const entryToDelete = ref<MaintenanceEntry | null>(null)
const deleteFeedback = useFormFeedback()
const deletingEntry = ref(false)
const { schedule: scheduleDeleteClose, forceExpire: forceDeleteClose } = useAutoClose()
watch(entryToDelete, (entry) => { if (!entry) forceDeleteClose() })

const defaultForm = () => ({
  title: '',
  description: '',
  intervention_date: new Date().toISOString().slice(0, 10),
  cost: '',
  vendor_name: '',
})
const entryForm = ref(defaultForm())

function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate())
}

function startOfPeriod(date: Date, filter: Exclude<PeriodFilter, 'all'>): Date {
  const year = date.getFullYear()
  const month = date.getMonth()

  if (filter === 'month') return new Date(year, month, 1)
  if (filter === 'quarter') return new Date(year, Math.floor(month / 3) * 3, 1)
  if (filter === 'semester') return new Date(year, month < 6 ? 0 : 6, 1)
  return new Date(year, 0, 1)
}

function endOfPeriod(date: Date, filter: Exclude<PeriodFilter, 'all'>): Date {
  const year = date.getFullYear()
  const month = date.getMonth()

  if (filter === 'month') return new Date(year, month + 1, 0)
  if (filter === 'quarter') return new Date(year, Math.floor(month / 3) * 3 + 3, 0)
  if (filter === 'semester') return new Date(year, month < 6 ? 6 : 12, 0)
  return new Date(year, 12, 0)
}

function shiftPeriod(date: Date, filter: Exclude<PeriodFilter, 'all'>, delta: number): Date {
  if (filter === 'month') return new Date(date.getFullYear(), date.getMonth() + delta, 1)
  if (filter === 'quarter') return new Date(date.getFullYear(), date.getMonth() + delta * 3, 1)
  if (filter === 'semester') return new Date(date.getFullYear(), date.getMonth() + delta * 6, 1)
  return new Date(date.getFullYear() + delta, 0, 1)
}

function parseLocalDate(value: string): Date | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value)
  if (!match) return null

  const year = Number(match[1])
  const month = Number(match[2])
  const day = Number(match[3])
  if ([year, month, day].some(Number.isNaN)) return null

  return new Date(year, month - 1, day)
}

function resetPeriod(filter: PeriodFilter) {
  if (filter === 'all') return
  periodAnchor.value = startOfPeriod(new Date(), filter)
}

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [summaryRes, entriesRes] = await Promise.all([
    maintenanceApi.summary(),
    maintenanceApi.list(),
  ])
  logbookSummary.value = summaryRes.data
  buildingName.value = summaryRes.data.building.name
  allEntries.value = entriesRes.data
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

// ── Derived ───────────────────────────────────────────────────────
const periodRange = computed(() => {
  if (periodFilter.value === 'all') return null

  const start = startOfDay(periodAnchor.value)
  const today = startOfDay(new Date())
  const periodEnd = endOfPeriod(start, periodFilter.value)

  return {
    start,
    end: periodEnd > today ? today : periodEnd,
  }
})

const periodLabel = computed(() => {
  if (periodFilter.value === 'all') return t('maintenance.period.all')

  const start = periodAnchor.value
  const today = new Date()
  if (periodFilter.value === 'month') {
    return t('maintenance.period.month', {
      month: new Intl.DateTimeFormat('fr-FR', { month: 'long' }).format(start),
      year: start.getFullYear(),
    })
  }
  if (periodFilter.value === 'quarter') {
    return t('maintenance.period.quarter', {
      quarter: Math.floor(start.getMonth() / 3) + 1,
      year: start.getFullYear(),
    })
  }
  if (periodFilter.value === 'semester') {
    return t('maintenance.period.semester', {
      semester: start.getMonth() < 6 ? 1 : 2,
      year: start.getFullYear(),
    })
  }
  if (start.getFullYear() === today.getFullYear()) {
    return t('maintenance.period.year', { year: start.getFullYear() })
  }
  return String(start.getFullYear())
})

const canGoNext = computed(() => {
  if (periodFilter.value === 'all') return false
  const currentStart = startOfPeriod(new Date(), periodFilter.value)
  return periodAnchor.value.getTime() < currentStart.getTime()
})

const entries = computed(() => {
  const q = search.value.toLowerCase()
  if (!q) return allEntries.value
  return allEntries.value.filter(e =>
    e.title.toLowerCase().includes(q) ||
    (e.vendor_name ?? '').toLowerCase().includes(q),
  )
})

const periodEntries = computed(() => {
  const range = periodRange.value
  if (!range) return entries.value
  return entries.value.filter(entry => {
    const date = parseLocalDate(entry.intervention_date)
    return date ? date >= range.start && date <= range.end : false
  })
})

const totalCost = computed(() =>
  periodEntries.value.reduce((sum, entry) => sum + Number(entry.cost ?? 0), 0),
)

const buildingAddress = computed(() => {
  const building = logbookSummary.value?.building
  if (!building) return ''

  return [
    building.address_line1,
    building.address_line2,
    `${building.postal_code} ${building.city}`,
  ]
    .filter(Boolean)
    .join(', ')
})

const currentManagerLabel = computed(() => {
  const manager = logbookSummary.value?.current_manager
  if (!manager) return t('maintenance.regulatory.noManager')
  return `${manager.first_name} ${manager.last_name}`.trim()
})

const regulatoryLegalLinks = computed(() => [
  {
    label: t('maintenance.regulatory.legalCard.linkLabel'),
    href: t('maintenance.regulatory.legalCard.linkHref'),
  },
])

function contractStatusVariant(status: LogbookContractSummary['status']): 'open' | 'closed' | 'default' {
  if (status === 'active') return 'open'
  if (status === 'expired' || status === 'terminated') return 'closed'
  return 'default'
}

function selectPeriod(filter: PeriodFilter) {
  periodFilter.value = filter
  resetPeriod(filter)
}

function goToPreviousPeriod() {
  if (periodFilter.value === 'all') return
  periodAnchor.value = shiftPeriod(periodAnchor.value, periodFilter.value, -1)
}

function goToNextPeriod() {
  if (periodFilter.value === 'all' || !canGoNext.value) return
  periodAnchor.value = shiftPeriod(periodAnchor.value, periodFilter.value, 1)
}

// ── Entry modal actions ────────────────────────────────────────────
function openCreateModal() {
  entryForm.value = defaultForm()
  entryFeedback.reset()
  showEntryModal.value = true
}

async function saveEntry() {
  if (entryFeedback.status.value === 'success') { forceEntryClose(); return }
  if (!entryForm.value.title.trim()) return
  savingEntry.value = true
  entryFeedback.reset()
  try {
    await maintenanceApi.create({
      title: entryForm.value.title,
      description: entryForm.value.description || undefined,
      intervention_date: entryForm.value.intervention_date,
      cost: entryForm.value.cost ? Number(entryForm.value.cost) : undefined,
      vendor_name: entryForm.value.vendor_name || undefined,
    })
    entryFeedback.setSuccess(t('common.saved'))
    scheduleEntryClose(() => {
      showEntryModal.value = false
      reload()
    })
  } catch (err) {
    entryFeedback.setError(toErrorMessage(err))
  } finally {
    savingEntry.value = false
  }
}

function openDeleteModal(entry: MaintenanceEntry) {
  entryToDelete.value = entry
  deleteFeedback.reset()
}

async function deleteEntry() {
  if (deleteFeedback.status.value === 'success') { forceDeleteClose(); return }
  if (!entryToDelete.value) return
  deletingEntry.value = true
  deleteFeedback.reset()
  try {
    await maintenanceApi.delete(entryToDelete.value.id)
    deleteFeedback.setSuccess(t('maintenance.deleteSuccess'))
    scheduleDeleteClose(() => {
      entryToDelete.value = null
      reload()
    })
  } catch (err) {
    deleteFeedback.setError(toErrorMessage(err))
  } finally {
    deletingEntry.value = false
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
          <span>{{ t('maintenance.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          @click="openCreateModal"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('maintenance.addEntry') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="flex flex-col p-10 max-w-[1200px] gap-6 max-sm:p-5">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else>
        <header class="flex flex-col items-start gap-2">
          <CEyebrow>
            {{ t('page.eyebrows.maintenance') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('maintenance.title') }}
          </CHeading>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('maintenance.periodSummary', { label: periodLabel, count: periodEntries.length, cost: formatCurrency(totalCost) }) }}
          </p>
        </header>

        <div
          v-if="logbookSummary"
          class="regulatory-grid"
        >
          <CCard class="regulatory-card">
            <div class="regulatory-card__header">
              <span class="title-sm">{{ t('maintenance.regulatory.title') }}</span>
              <CBadge variant="default">
                {{ t('maintenance.regulatory.decree') }}
              </CBadge>
            </div>
            <div class="regulatory-card__body">
              <div>
                <span class="regulatory-card__label">{{ t('maintenance.regulatory.building') }}</span>
                <p class="body-xs regulatory-card__value">
                  {{ buildingAddress }}
                </p>
              </div>
              <div>
                <span class="regulatory-card__label">{{ t('maintenance.regulatory.manager') }}</span>
                <p class="body-xs regulatory-card__value">
                  {{ currentManagerLabel }}
                </p>
                <p
                  v-if="logbookSummary.current_manager?.email"
                  class="body-xs regulatory-card__hint"
                >
                  {{ logbookSummary.current_manager.email }}
                  <template v-if="logbookSummary.current_manager.phone">
                    · {{ logbookSummary.current_manager.phone }}
                  </template>
                </p>
              </div>
            </div>
          </CCard>

          <CCard class="regulatory-card">
            <div class="regulatory-card__header">
              <span class="title-sm">{{ t('maintenance.regulatory.insuranceContracts') }}</span>
              <CBadge
                variant="default"
                :dot="logbookSummary.insurance_contracts.length > 0"
              >
                {{ t('maintenance.regulatory.contractsCount', logbookSummary.insurance_contracts.length) }}
              </CBadge>
            </div>
            <div
              v-if="logbookSummary.insurance_contracts.length > 0"
              class="contract-list"
            >
              <div
                v-for="contract in logbookSummary.insurance_contracts"
                :key="contract.id"
                class="contract-list__item"
              >
                <div>
                  <p class="body-xs contract-list__title">
                    {{ contract.subject }}
                  </p>
                  <p class="body-xs contract-list__meta">
                    {{ contract.vendor_name }}
                  </p>
                </div>
                <div class="contract-list__aside">
                  <CBadge :variant="contractStatusVariant(contract.status)">
                    {{ t(`contracts.statuses.${contract.status}`) }}
                  </CBadge>
                  <span class="contract-list__deadline">
                    {{ t('maintenance.regulatory.endsOn', { date: formatDate(contract.end_date) }) }}
                  </span>
                </div>
              </div>
            </div>
            <p
              v-else
              class="body-xs regulatory-card__hint"
            >
              {{ t('maintenance.regulatory.noInsuranceContracts') }}
            </p>
          </CCard>

          <CCard class="regulatory-card">
            <div class="regulatory-card__header">
              <span class="title-sm">{{ t('maintenance.regulatory.maintenanceContracts') }}</span>
              <CBadge
                variant="default"
                :dot="logbookSummary.maintenance_contracts.length > 0"
              >
                {{ t('maintenance.regulatory.contractsCount', logbookSummary.maintenance_contracts.length) }}
              </CBadge>
            </div>
            <div
              v-if="logbookSummary.maintenance_contracts.length > 0"
              class="contract-list"
            >
              <div
                v-for="contract in logbookSummary.maintenance_contracts"
                :key="contract.id"
                class="contract-list__item"
              >
                <div>
                  <p class="body-xs contract-list__title">
                    {{ contract.subject }}
                  </p>
                  <p class="body-xs contract-list__meta">
                    {{ contract.vendor_name }} · {{ t(`contracts.types.${contract.contract_type}`) }}
                  </p>
                </div>
                <div class="contract-list__aside">
                  <CBadge :variant="contractStatusVariant(contract.status)">
                    {{ t(`contracts.statuses.${contract.status}`) }}
                  </CBadge>
                  <span class="contract-list__deadline">
                    {{ t('maintenance.regulatory.endsOn', { date: formatDate(contract.end_date) }) }}
                  </span>
                </div>
              </div>
            </div>
            <p
              v-else
              class="body-xs regulatory-card__hint"
            >
              {{ t('maintenance.regulatory.noMaintenanceContracts') }}
            </p>
          </CCard>
        </div>

        <CLegalCard
          v-if="logbookSummary"
          class="regulatory-legal-card"
        >
          <CLegalNotice :links="regulatoryLegalLinks">
            <CLegalDescription
              :title="t('maintenance.regulatory.legalCard.title')"
              :summary="t('maintenance.regulatory.legalCard.summary')"
            />
          </CLegalNotice>
        </CLegalCard>

        <div class="period-toolbar">
          <CTabs
            variant="filter"
            v-model="periodFilter"
            :tabs="(['month', 'quarter', 'semester', 'year', 'all'] as const).map(f => ({ value: f, label: t(`maintenance.filters.${f}`) }))"
          />
          <div class="period-nav">
            <CBtn
              variant="ghost"
              size="sm"
              :disabled="periodFilter === 'all'"
              :aria-label="t('maintenance.previousPeriod')"
              @click="goToPreviousPeriod"
            >
              ←
            </CBtn>
            <span class="period-label title-sm">{{ periodLabel }}</span>
            <CBtn
              variant="ghost"
              size="sm"
              :disabled="!canGoNext"
              :aria-label="t('maintenance.nextPeriod')"
              @click="goToNextPeriod"
            >
              →
            </CBtn>
          </div>
        </div>

        <CFormInput
          v-model="search"
          type="search"
          :placeholder="t('maintenance.searchPlaceholder')"
        >
          <template #prefix>
            <CIconSearch />
          </template>
        </CFormInput>

        <div class="timeline">
          <div
            v-for="entry in periodEntries"
            :key="entry.id"
            class="timeline-item"
          >
            <div
              class="timeline-dot"
              :class="entry.source === 'automatic' ? 'timeline-dot--auto' : ''"
            ></div>
            <CCard class="timeline-card">
              <div class="entry-header">
                <div>
                  <span class="title-sm">{{ entry.title }}</span>
                  <CBadge
                    :variant="entry.source === 'automatic' ? 'primary' : 'default'"
                    style="margin-left:8px"
                  >
                    {{ entry.source === 'automatic' ? t('maintenance.automatic') : t('maintenance.manual') }}
                  </CBadge>
                </div>
                <div class="entry-header-right">
                  <span class="entry-date">{{ formatDate(entry.intervention_date) }}</span>
                  <button
                    class="icon-btn icon-btn--danger"
                    :title="t('maintenance.deleteEntry')"
                    @click="openDeleteModal(entry)"
                  >
                    <CIconTrash
                      width="14"
                      height="14"
                    />
                  </button>
                </div>
              </div>
              <p
                v-if="entry.description"
                class="body-xs entry-desc"
              >
                {{ entry.description }}
              </p>
              <div class="entry-footer">
                <span
                  v-if="entry.vendor_name"
                  class="vendor"
                >
                  <CIconWrench
                    width="13"
                    height="13"
                    style="margin-right:4px"
                  />{{ entry.vendor_name }}
                </span>
                <span
                  v-if="entry.cost != null"
                  class="cost"
                >{{ formatCurrency(entry.cost) }}</span>
              </div>
            </CCard>
          </div>

          <div
            v-if="!periodEntries.length"
            class="py-8 text-center text-md text-on-surface-variant"
          >
            {{ t('maintenance.emptyPeriod') }}
          </div>
        </div>
      </template>
    </div>

    <!-- Create entry modal -->
    <CModal
      :open="showEntryModal"
      :title="t('maintenance.addEntry')"
      size="md"
      @update:open="showEntryModal = $event"
    >
      <form
        class="flex flex-col gap-4"
        @submit.prevent="saveEntry"
      >
        <CFormInput
          v-model="entryForm.title"
          :label="t('maintenance.fields.title')"
          required
        />
        <CFormTextarea
          v-model="entryForm.description"
          :label="t('maintenance.fields.description')"
          :rows="2"
        />
        <CFormInput
          v-model="entryForm.intervention_date"
          :label="t('maintenance.fields.interventionDate')"
          type="date"
          required
        />
        <div class="form-row">
          <CFormInput
            v-model="entryForm.cost"
            :label="t('maintenance.fields.cost')"
            type="number"
            :step="0.01"
            :min="0"
          />
          <CFormInput
            v-model="entryForm.vendor_name"
            :label="t('maintenance.fields.vendorName')"
          />
        </div>
      </form>
      <template #footer>
        <CFormFeedback
          class="modal-footer-feedback"
          :status="entryFeedback.status.value"
          :message="entryFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showEntryModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="savingEntry"
          :closing="entryFeedback.status.value === 'success'"
          @click="saveEntry"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>

    <CModal
      :open="!!entryToDelete"
      :title="t('maintenance.confirmDeleteTitle')"
      size="sm"
      @update:open="v => !v && (entryToDelete = null)"
    >
      <p class="body-xs">
        {{ t('maintenance.confirmDeleteMessage', { title: entryToDelete?.title ?? '' }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="deleteFeedback.status.value"
          :message="deleteFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="entryToDelete = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="deletingEntry"
          :closing="deleteFeedback.status.value === 'success'"
          @click="deleteEntry"
        >
          {{ t('common.delete') }}
        </CBtn>
      </template>
    </CModal>
  </div>
</template>

<style scoped>
.timeline { display: flex; flex-direction: column; gap: 0; }
.regulatory-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.regulatory-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.regulatory-card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
}

.regulatory-card__body {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.regulatory-card__label {
  display: block;
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
  margin-bottom: 4px;
}

.regulatory-card__value {
  margin: 0;
}

.regulatory-card__hint {
  margin: 4px 0 0;
  color: var(--color-on-surface-variant);
}

.regulatory-legal-card {
  margin-bottom: var(--space-4);
}

.contract-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.contract-list__item {
  display: flex;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
}

.contract-list__title,
.contract-list__meta {
  margin: 0;
}

.contract-list__meta,
.contract-list__deadline {
  color: var(--color-on-surface-variant);
}

.contract-list__aside {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
  flex-shrink: 0;
}

.timeline-item {
  display: flex; gap: 16px; align-items: stretch;
  padding-bottom: 16px;
  position: relative;
}

.timeline-dot {
  width: var(--icon-xs); height: var(--icon-xs); border-radius: 50%; flex-shrink: 0;
  background: var(--color-primary);
  border: 3px solid var(--color-surface);
  margin-top: 16px;
  z-index: 1;
  position: relative;
}
.timeline-dot--auto { background: var(--color-secondary); }

.timeline-item::before {
  content: '';
  position: absolute;
  left: 7px; top: 28px; bottom: 0;
  width: 2px;
  background: var(--color-outline-variant);
}
.timeline-item:last-child::before { display: none; }

.timeline-card { flex: 1; }

.entry-header { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--space-3); margin-bottom: 6px; }
.entry-header-right { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
.entry-date { font-size: var(--text-xs); color: var(--color-on-surface-variant); }
.entry-desc { color: var(--color-on-surface-variant); margin-bottom: var(--space-2); }
.entry-footer { display: flex; justify-content: flex-start; gap: var(--space-3); flex-wrap: wrap; font-size: var(--text-sm); align-items: center; }
.vendor { color: var(--color-on-surface-variant); display: flex; align-items: center; margin-right: auto; }
.cost {
  margin-left: auto;
  font-family: var(--font-mono);
  font-weight: var(--fw-semibold);
  color: var(--color-on-surface);
  font-variant-numeric: tabular-nums;
}
.modal-footer-feedback { margin-right: auto; min-width: 0; }

.period-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.period-tabs { flex: 1; min-width: 0; }

.period-nav {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.period-label {
  min-width: 9rem;
  text-align: center;
}

.form-row { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-3); }

@media (max-width: 720px) {
  .regulatory-grid {
    grid-template-columns: 1fr;
  }

  .contract-list__item {
    flex-direction: column;
  }

  .contract-list__aside {
    align-items: flex-start;
  }

  .period-nav {
    width: 100%;
    justify-content: space-between;
  }

  .period-label {
    flex: 1;
    min-width: 0;
  }

  .form-row {
    grid-template-columns: 1fr;
  }
}
</style>
