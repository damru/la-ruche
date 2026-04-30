<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useConfirm } from '@/composables/useConfirm'
import { useAutoClose } from '@/composables/useAutoClose'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { buildingApi, maintenanceApi } from '@/api'
import type { MaintenanceEntry } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatCurrency } = useFormatters()
const { confirm } = useConfirm()

type PeriodFilter = 'month' | 'quarter' | 'semester' | 'year' | 'all'

// ── State ─────────────────────────────────────────────────────────
const buildingName = ref('')
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
  const [buildingRes, entriesRes] = await Promise.all([
    buildingApi.get(),
    maintenanceApi.list(),
  ])
  buildingName.value = buildingRes.data.name
  allEntries.value = entriesRes.data
}

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
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
    entryFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    savingEntry.value = false
  }
}

async function deleteEntry(entry: MaintenanceEntry) {
  const ok = await confirm({
    title: t('maintenance.confirmDeleteTitle'),
    message: t('maintenance.confirmDeleteMessage', { title: entry.title }),
    confirmLabel: t('common.delete'),
    variant: 'danger',
  })
  if (!ok) return
  try {
    await maintenanceApi.delete(entry.id)
    await reload()
  } catch (err) {
    alert(toErrorMessage(err, t('common.error')))
  }
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
              {{ t('maintenance.title') }}
            </h1>
            <p
              class="body-md"
              style="color:var(--color-on-surface-variant)"
            >
              {{ t('maintenance.periodSummary', { label: periodLabel, count: periodEntries.length, cost: formatCurrency(totalCost) }) }}
            </p>
          </div>
        </div>

        <div class="period-toolbar">
          <div class="tabs period-tabs">
            <button
              v-for="filter in ['month', 'quarter', 'semester', 'year', 'all'] as PeriodFilter[]"
              :key="filter"
              :class="['tab', { 'tab--active': periodFilter === filter }]"
              @click="selectPeriod(filter)"
            >
              {{ t(`maintenance.filters.${filter}`) }}
            </button>
          </div>
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

        <div class="search-bar">
          <CIconSearch class="search-icon" />
          <input
            v-model="search"
            type="search"
            :placeholder="t('maintenance.searchPlaceholder')"
            class="search-input"
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
                  <button
                    class="icon-btn icon-btn--danger"
                    :title="t('maintenance.deleteEntry')"
                    @click="deleteEntry(entry)"
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
                class="body-sm entry-desc"
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
            class="empty-state"
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
        class="form-grid"
        @submit.prevent="saveEntry"
      >
        <div class="form-field">
          <label class="form-label">{{ t('maintenance.fields.title') }} *</label>
          <input
            v-model="entryForm.title"
            type="text"
            class="form-input"
            required
          />
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('maintenance.fields.description') }}</label>
          <textarea
            v-model="entryForm.description"
            class="form-input"
            rows="2"
          ></textarea>
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('maintenance.fields.interventionDate') }} *</label>
          <input
            v-model="entryForm.intervention_date"
            type="date"
            class="form-input"
            required
          />
        </div>
        <div class="form-row">
          <div class="form-field">
            <label class="form-label">{{ t('maintenance.fields.cost') }}</label>
            <input
              v-model="entryForm.cost"
              type="number"
              step="0.01"
              min="0"
              class="form-input"
            />
          </div>
          <div class="form-field">
            <label class="form-label">{{ t('maintenance.fields.vendorName') }}</label>
            <input
              v-model="entryForm.vendor_name"
              type="text"
              class="form-input"
            />
          </div>
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
  </div>
</template>

<style scoped>
.timeline { display: flex; flex-direction: column; gap: 0; }
.timeline-item {
  display: flex; gap: 16px; align-items: stretch;
  padding-bottom: 16px;
  position: relative;
}

.timeline-dot {
  width: 14px; height: 14px; border-radius: 50%; flex-shrink: 0;
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

.empty-state { padding: var(--space-8); text-align: center; font-size: var(--text-md); color: var(--color-on-surface-variant); }

@media (max-width: 720px) {
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
