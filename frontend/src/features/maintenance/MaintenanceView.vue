<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CSeparator from '@/components/atoms/CSeparator.vue'
import CCard from '@/components/molecules/CCard.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CMetaLine from '@/components/molecules/CMetaLine.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import CIconArrowLeft from '@/components/atoms/CIconArrowLeft.svg'
import CIconArrowRight from '@/components/atoms/CIconArrowRight.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import MaintenanceEntryModal from '@/features/maintenance/MaintenanceEntryModal.vue'
import DeleteMaintenanceEntryModal from '@/features/maintenance/DeleteMaintenanceEntryModal.vue'
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
const entryToDelete = ref<MaintenanceEntry | null>(null)

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
  showEntryModal.value = true
}

function openDeleteModal(entry: MaintenanceEntry) {
  entryToDelete.value = entry
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
          :icon="CIconPlus"
          :title="t('maintenance.addEntry')"
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
            {{ t('page.eyebrows.maintenance') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('maintenance.title') }}
          </CHeading>
          <CMetaLine>
            <CMetaItem variant="sub">
              {{ periodLabel }}
            </CMetaItem>
            <CSeparator />
            <CMetaItem variant="sub">
              {{ t('maintenance.header.entriesCount', periodEntries.length) }}
            </CMetaItem>
            <CSeparator />
            <CMetaItem variant="sub">
              {{ t('maintenance.header.totalCost', { amount: formatCurrency(totalCost) }) }}
            </CMetaItem>
          </CMetaLine>
        </header>
        <div class="page-layout">
          <aside
            v-if="logbookSummary"
            class="page-sidebar"
          >
            <div class="regulatory-grid">
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
          </aside>

          <div class="page-main">
            <CTabs
              v-model="periodFilter"
              variant="filter"
              :tabs="(['month', 'quarter', 'semester', 'year', 'all'] as const).map(f => ({ value: f, label: t(`maintenance.filters.${f}`) }))"
            >
              <CFormInput
                v-model="search"
                type="search"
                :placeholder="t('maintenance.searchPlaceholder')"
              >
                <template #prefix>
                  <CIconSearch />
                </template>
              </CFormInput>
            </CTabs>
            <div class="flex flex-col gap-4">
              <div class="period-nav">
                <CBtn
                  variant="ghost"
                  size="sm"
                  :icon="CIconArrowLeft"
                  :title="t('maintenance.previousPeriod')"
                  :disabled="periodFilter === 'all'"
                  :aria-label="t('maintenance.previousPeriod')"
                  @click="goToPreviousPeriod"
                />
                <span class="period-label title-sm">{{ periodLabel }}</span>
                <CBtn
                  variant="ghost"
                  size="sm"
                  :icon="CIconArrowRight"
                  :title="t('maintenance.nextPeriod')"
                  :disabled="!canGoNext"
                  :aria-label="t('maintenance.nextPeriod')"
                  @click="goToNextPeriod"
                />
              </div>
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
                        <CBtn
                          variant="icon"
                          :icon="CIconTrash"
                          class="tone--err"
                          :title="t('maintenance.deleteEntry')"
                          :aria-label="t('maintenance.deleteEntry')"
                          @click="openDeleteModal(entry)"
                        />
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
                          class="icon-2xs"
                        />
                        {{ entry.vendor_name }}
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
            </div>
          </div>
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
      </template>
    </div>

    <MaintenanceEntryModal
      :open="showEntryModal"
      @update:open="showEntryModal = $event"
      @saved="reload"
    />

    <DeleteMaintenanceEntryModal
      :open="!!entryToDelete"
      :entry="entryToDelete"
      @update:open="v => !v && (entryToDelete = null)"
      @deleted="() => { entryToDelete = null; reload() }"
    />
  </div>
</template>

<style scoped>
.timeline { display: flex; flex-direction: column; gap: 0; }

/* ── Page layout ─────────────────────────────────────── */
.page-layout {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.page-main {
  display: flex;
  flex-direction: column;
  gap: var(--space-8);
  min-width: 0;
}

/* Regulatory cards: always a vertical stack */
.regulatory-grid {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
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
  border-radius: var(--radius-xl);
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

.period-tabs { flex: 1; min-width: 0; }

.period-nav {
  display: flex;
  align-self: center;
  align-items: center;
  gap: var(--space-2);
}

.period-label {
  min-width: 9rem;
  text-align: center;
}

.form-row { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-3); }

@media (min-width: 1024px) {
  .page-layout {
    display: grid;
    grid-template-columns: 1fr 300px;
    grid-template-areas: "main sidebar";
    align-items: start;
  }

  .page-main   { grid-area: main; }
  .page-sidebar { grid-area: sidebar; }
}

@media (max-width: 720px) {
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
