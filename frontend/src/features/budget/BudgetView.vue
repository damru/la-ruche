<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import { mockBudgets, mockBudgetLines, mockLevies, mockWorksFund, mockBuildingStats } from '@/mock/data'

const { t } = useI18n()
const { formatCurrency, formatDate } = useFormatters()

const selectedBudgetId = ref(mockBudgets[0].id)
const showGenerateModal = ref(false)

const selectedBudget = computed(() =>
  mockBudgets.find(b => b.id === selectedBudgetId.value) ?? mockBudgets[0]
)
const budgetLines = computed(() =>
  mockBudgetLines.filter(l => l.budget_id === selectedBudgetId.value)
)
const totalForecast = computed(() =>
  budgetLines.value.reduce((s, l) => s + l.budgeted_amount, 0)
)
const totalActual = computed(() =>
  budgetLines.value.reduce((s, l) => s + (l.actual_amount ?? 0), 0)
)
const executionRate = computed(() =>
  totalForecast.value ? Math.round((totalActual.value / totalForecast.value) * 100) : 0
)
const overdueLevies = computed(() => mockLevies.filter(l => l.status === 'overdue'))
const overdueTotal = computed(() => overdueLevies.value.reduce((s, l) => s + l.amount, 0))
const paidLevies = computed(() => mockLevies.filter(l => l.status === 'paid'))
const collectionRate = computed(() =>
  mockLevies.length ? Math.round((paidLevies.value.length / mockLevies.length) * 100) : 100
)
const worksAvailPct = computed(() =>
  mockWorksFund.levied_amount
    ? Math.round((mockWorksFund.available_amount / mockWorksFund.levied_amount) * 100)
    : 0
)
const worksEngagedPct = computed(() => 100 - worksAvailPct.value)

function linePct(line: typeof budgetLines.value[0]): number {
  if (!line.actual_amount || !line.budgeted_amount) return 0
  return Math.min(100, Math.round((line.actual_amount / line.budgeted_amount) * 100))
}
function lineVariant(line: typeof budgetLines.value[0]): string {
  const pct = linePct(line)
  if (pct >= 100) return 'over'
  if (pct >= 85) return 'warn'
  return 'primary'
}
function levyStatusLabel(s: string): string {
  return t(`budget.levyStatuses.${s}`)
}
function levyChipClass(s: string): string {
  if (s === 'paid') return 'tone--valid'
  if (s === 'issued') return 'tone--warn'
  if (s === 'overdue') return 'tone--err'
  return 'tone--disabled'
}
function initials(name: string): string {
  return name.split(' ').map(w => w[0]).join('').toUpperCase().slice(0, 2)
}
</script>

<template>
  <div class="budget-page view-page">
    <!-- Topbar -->
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ mockBuildingStats.name }}
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>Budget &amp; Charges</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="secondary"
          size="sm"
        >
          <CIconPlus />
          <span class="btn-label">{{ t('budget.newLine') }}</span>
        </CBtn>
        <CBtn
          variant="primary"
          size="sm"
          @click="showGenerateModal = true"
        >
          <CIconCheckCircle />
          <span class="btn-label">{{ t('budget.generateLevies') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <!-- Content -->
    <div class="content view-container">
      <CMockBanner />

      <!-- Page header -->
      <div class="page-header">
        <p class="page-eyebrow">
          {{ t('budget.title') }}
        </p>
        <h1 class="page-title">
          {{ t('budget.fiscalYear', { year: selectedBudget.fiscal_year }) }}
        </h1>
        <p class="page-sub">
          {{ mockBuildingStats.name }} ·
          {{ selectedBudget.status === 'approved' ? t('budget.statuses.approvedAG') : t('budget.statuses.draft') }}
        </p>
      </div>

      <!-- Year tabs -->
      <div class="tabs">
        <button
          v-for="b in mockBudgets"
          :key="b.id"
          :class="['tab', { 'tab--active': selectedBudgetId === b.id }]"
          @click="selectedBudgetId = b.id"
        >
          {{ b.fiscal_year }}
        </button>
        <button class="tab tab--new">
          {{ t('budget.newFiscalYear') }}
        </button>
      </div>

      <!-- KPI row -->
      <div class="kpi-grid">
        <!-- Budget prévisionnel -->
        <div class="kpi-card">
          <div class="kpi-head">
            <span class="kpi-label">{{ t('budget.budgetForecastLabel') }}</span>
            <div class="kpi-icon tone--default">
              <CIconCurrency />
            </div>
          </div>
          <div class="kpi-value">
            {{ formatCurrency(totalForecast) }}
          </div>
          <div class="kpi-delta kpi-delta--neutral">
            {{ t('budget.budgetItems', budgetLines.length) }}
          </div>
        </div>

        <!-- Réalisé -->
        <div class="kpi-card">
          <div class="kpi-head">
            <span class="kpi-label">{{ t('budget.realizedToDate') }}</span>
            <div class="kpi-icon tone--accent">
              <CIconCheck stroke="white" />
            </div>
          </div>
          <div class="kpi-value">
            {{ formatCurrency(totalActual) }}
          </div>
          <div class="kpi-delta kpi-delta--neutral">
            {{ t('budget.ofBudget', { pct: executionRate }) }}
          </div>
        </div>

        <!-- Appels en retard -->
        <div class="kpi-card">
          <div class="kpi-head">
            <span class="kpi-label">{{ t('budget.overdueCallsLabel') }}</span>
            <div class="kpi-icon tone--err">
              <CIconWarn />
            </div>
          </div>
          <div class="kpi-value kpi-value--err">
            {{ overdueLevies.length }}
          </div>
          <div class="kpi-delta kpi-delta--down">
            {{ t('budget.pendingAmount', { amount: formatCurrency(overdueTotal) }) }}
          </div>
        </div>

        <!-- Fonds travaux -->
        <div class="kpi-card">
          <div class="kpi-head">
            <span class="kpi-label">{{ t('budget.worksFund') }}</span>
            <div class="kpi-icon tone--warn">
              <CIconWrench />
            </div>
          </div>
          <div class="kpi-value kpi-value--warn">
            {{ formatCurrency(mockWorksFund.levied_amount) }}
          </div>
          <div class="kpi-delta kpi-delta--neutral">
            {{ t('budget.available', { amount: formatCurrency(mockWorksFund.available_amount) }) }}
          </div>
        </div>
      </div>

      <!-- Two-col layout -->
      <div class="two-col">
        <!-- LEFT: sections -->
        <div class="left-col">
          <!-- Consommation par poste -->
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
              <button class="btn-ghost-sm">
                <CIconDownload />
                {{ t('common.export') }}
              </button>
            </div>
            <div class="prog-list">
              <div
                v-for="line in budgetLines"
                :key="line.id"
                class="prog-item"
              >
                <div class="prog-header">
                  <span class="prog-label">{{ line.label }}</span>
                  <div class="prog-meta">
                    <span class="chip tone--default">{{ line.key_name }}</span>
                    <span :class="['prog-pct', 'prog-pct--' + lineVariant(line)]">{{ linePct(line) }}%</span>
                  </div>
                </div>
                <div class="prog-track">
                  <div
                    :class="['prog-fill', 'prog-fill--' + lineVariant(line)]"
                    :style="{ width: linePct(line) + '%' }"
                  ></div>
                </div>
                <div class="prog-foot">
                  <span>{{ t('budget.realizedLabel') }} : <b>{{ formatCurrency(line.actual_amount ?? 0) }}</b></span>
                  <span>{{ t('budget.plannedLabel') }} : <b>{{ formatCurrency(line.budgeted_amount) }}</b></span>
                </div>
              </div>
            </div>
          </div>

          <!-- Appels de fonds — tableau -->
          <div class="section-card">
            <div class="section-head">
              <div>
                <h2 class="section-title">
                  {{ t('budget.levies') }}
                </h2>
                <p class="section-sub">
                  {{ t('budget.collectionRate', { lots: mockLevies.length, pct: collectionRate }) }}
                </p>
              </div>
              <div class="section-actions">
                <button class="btn-ghost-sm">
                  {{ t('common.filter') }}
                </button>
                <CBtn
                  variant="secondary"
                  size="sm"
                >
                  <CIconPlus />
                  {{ t('budget.markPaid') }}
                </CBtn>
              </div>
            </div>
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>{{ t('common.lot') }}</th>
                    <th>{{ t('common.coOwner') }}</th>
                    <th class="td-right">
                      {{ t('common.amount') }}
                    </th>
                    <th class="td-right">
                      {{ t('common.dueDate') }}
                    </th>
                    <th>{{ t('common.status') }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="levy in mockLevies"
                    :key="levy.id"
                  >
                    <td class="td-mono">
                      {{ levy.unit_number }}
                    </td>
                    <td>{{ levy.owner_name }}</td>
                    <td :class="['td-amount', { 'td-amount--err': levy.status === 'overdue' }]">
                      {{ formatCurrency(levy.amount) }}
                    </td>
                    <td class="td-right td-muted">
                      {{ levy.due_date ? formatDate(levy.due_date) : '—' }}
                    </td>
                    <td>
                      <span :class="['chip', levyChipClass(levy.status)]">
                        <span class="chip-dot"></span>
                        {{ levyStatusLabel(levy.status) }}
                      </span>
                    </td>
                  </tr>
                </tbody>
                <tfoot>
                  <tr class="tfoot-row">
                    <td colspan="2">
                      {{ t('common.total') }}
                    </td>
                    <td class="td-right">
                      {{ formatCurrency(mockLevies.reduce((s, l) => s + l.amount, 0)) }}
                    </td>
                    <td colspan="2"></td>
                  </tr>
                </tfoot>
              </table>
            </div>
          </div>
        </div><!-- /left-col -->

        <!-- RIGHT: sidebar -->
        <div class="right-col">
          <!-- Fonds travaux -->
          <div class="ft-card">
            <div class="ft-header">
              <h3 class="ft-title">
                Fonds travaux
              </h3>
              <span class="ft-badge tone--default">Art. 14-2 ALUR</span>
            </div>
            <div class="ft-amount">
              {{ formatCurrency(mockWorksFund.levied_amount) }}
            </div>
            <p class="ft-sub">
              {{ t('budget.separateBankAccount', { year: selectedBudget.fiscal_year }) }}
            </p>
            <div class="ft-prog">
              <div class="ft-prog-head">
                <span>{{ t('budget.engaged') }}</span>
                <span class="ft-prog-pct">{{ worksEngagedPct }}%</span>
              </div>
              <div class="prog-track">
                <div
                  class="prog-fill prog-fill--warn"
                  :style="{ width: worksEngagedPct + '%' }"
                ></div>
              </div>
              <div class="ft-prog-foot">
                <span>{{ t('budget.engagedAmount', { amount: formatCurrency(mockWorksFund.levied_amount - mockWorksFund.available_amount) }) }}</span>
                <span class="ft-prog-available">{{ t('budget.remaining', { amount: formatCurrency(mockWorksFund.available_amount) }) }}</span>
              </div>
            </div>
            <div class="ft-legal">
              {{ t('budget.worksFundLegalFull') }}
            </div>
          </div>

          <!-- Retards de paiement -->
          <div
            v-if="overdueLevies.length"
            class="mini-card"
          >
            <h3 class="mini-title">
              {{ t('budget.latePayments') }}
            </h3>
            <p class="mini-sub">
              {{ t('budget.latePaymentsDesc', { count: overdueLevies.length, amount: formatCurrency(overdueTotal) }) }}
            </p>
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
            <CBtn
              variant="secondary"
              size="sm"
              style="width:100%;justify-content:center;margin-top:16px"
            >
              {{ t('budget.sendReminders') }}
            </CBtn>
          </div>
        </div><!-- /right-col -->
      </div><!-- /two-col -->
    </div><!-- /content -->

    <!-- Modal: Générer les appels -->
    <Teleport to="body">
      <div
        :class="['modal-overlay', { 'modal-overlay--show': showGenerateModal }]"
        @click.self="showGenerateModal = false"
      >
        <div
          class="modal"
          role="dialog"
          aria-modal="true"
        >
          <h2 class="modal-title">
            {{ t('budget.generateLeviesTitle') }}
          </h2>
          <p class="modal-sub">
            {{ t('budget.generateLeviesSub') }}
          </p>
          <div class="field">
            <label>{{ t('budget.quarterLabel') }}</label>
            <select>
              <option>T1 2025 — Échéance 1er janvier</option>
              <option>T2 2025 — Échéance 1er avril</option>
              <option>T3 2025 — Échéance 1er juillet</option>
              <option selected>
                T4 2025 — Échéance 1er octobre
              </option>
            </select>
          </div>
          <div class="field">
            <label>{{ t('budget.totalToShare') }}</label>
            <input
              type="number"
              :value="Math.round(totalForecast / 4)"
            />
            <p class="field-hint">
              {{ t('budget.totalToShareHint', { amount: formatCurrency(totalForecast) }) }}
            </p>
          </div>
          <div class="modal-actions">
            <CBtn
              variant="secondary"
              size="sm"
              @click="showGenerateModal = false"
            >
              {{ t('common.cancel') }}
            </CBtn>
            <CBtn
              variant="primary"
              size="sm"
              @click="showGenerateModal = false"
            >
              {{ t('budget.generate') }}
            </CBtn>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.budget-page {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: var(--color-surface);
}
.content {
  padding: 0;
  flex: 1;
}

/* Page header */
.page-eyebrow {
  font-size: 11px;
  font-weight: var(--fw-semibold);
  letter-spacing: .1em;
  text-transform: uppercase;
  color: var(--color-primary);
  margin-bottom: 8px;
}
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

/* Tabs — .tabs/.tab/.tab--active defined globally in tabs.css */
.tabs { margin-bottom: 32px; }
.tab--new { color: var(--color-primary); }

/* KPI grid */
.kpi-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 32px;
}
.kpi-card {
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  padding: 20px;
  box-shadow: var(--shadow-md);
}
.kpi-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}
.kpi-label {
  font-size: 12px;
  color: var(--color-on-surface-variant);
}
.kpi-value {
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: var(--fw-bold);
  font-variation-settings: 'opsz' 72;
  letter-spacing: -.03em;
  color: var(--color-on-surface);
  line-height: 1;
  margin-bottom: 4px;
}
.kpi-value--err { color: var(--color-error); }
.kpi-value--warn { color: var(--color-warning, #e65100); }
.kpi-delta { font-size: 12px; font-weight: var(--fw-medium); }
.kpi-delta--neutral { color: var(--color-on-surface-variant); }
.kpi-delta--down { color: var(--color-error); }

/* Two-col */
.two-col {
  display: grid;
  grid-template-columns: 1fr 340px;
  gap: 24px;
  align-items: start;
}
.left-col { display: flex; flex-direction: column; gap: 24px; }
.right-col { display: flex; flex-direction: column; gap: 20px; }

/* Section card */
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
}
.section-sub { font-size: 12px; color: var(--color-on-surface-variant); margin-top: 2px; }
.section-actions { display: flex; gap: 8px; align-items: center; }
.btn-ghost-sm {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: var(--fw-medium);
  border: none;
  cursor: pointer;
  background: transparent;
  color: var(--color-primary);
  font-family: var(--font-body);
  transition: background .2s ease;
}
.btn-ghost-sm:hover { background: var(--color-secondary-container); }
.btn-ghost-sm svg { width: 13px; height: 13px; }

/* Progress bars */
.prog-list {
  padding: 0 24px 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.prog-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px; }
.prog-label { font-size: 13px; font-weight: var(--fw-medium); color: var(--color-on-surface); }
.prog-meta { display: flex; align-items: center; gap: 10px; }
.prog-pct { font-size: 13px; font-weight: var(--fw-semibold); }
.prog-pct--primary { color: var(--color-primary); }
.prog-pct--warn { color: var(--color-warning, #e65100); }
.prog-pct--over { color: var(--color-error); }
.prog-track {
  height: 10px;
  border-radius: var(--radius-full);
  background: var(--color-surface-container-highest);
  overflow: hidden;
}
.prog-fill {
  height: 100%;
  border-radius: var(--radius-full);
  transition: width .8s cubic-bezier(.34,1.3,.64,1);
}
.prog-fill--primary { background: var(--gradient-primary); }
.prog-fill--warn { background: linear-gradient(135deg, #e65100, #ff9800); }
.prog-fill--over { background: linear-gradient(135deg, #ba1a1a, #e53935); }
.prog-fill--ok { background: linear-gradient(135deg, #1b5e20, #43a047); }
.prog-foot { display: flex; justify-content: space-between; margin-top: 5px; }
.prog-foot span { font-size: 11px; color: var(--color-on-surface-variant); }
.prog-foot span b { color: var(--color-on-surface); font-weight: var(--fw-medium); font-family: var(--font-mono); }

/* Chips */
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
.chip-dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; opacity: .65; flex-shrink: 0; }

/* Table */
.table-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: separate; border-spacing: 0; }
thead th {
  font-size: 11px;
  font-weight: var(--fw-semibold);
  letter-spacing: .06em;
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
  padding: 10px 16px;
  text-align: left;
  background: var(--color-surface-container-low);
}
tbody td {
  font-size: 13px;
  color: var(--color-on-surface);
  padding: 13px 16px;
  border-bottom: 1px solid rgba(194,199,207,.12);
}
tbody tr:last-child td { border-bottom: none; }
tbody tr { transition: background .2s ease; cursor: pointer; }
tbody tr:hover td { background: var(--color-surface-container-low); }
.td-mono { font-family: var(--font-mono); font-size: 12px; color: var(--color-primary); }
.td-muted { color: var(--color-on-surface-variant); font-size: 12px; }
.td-right { text-align: right; }
.td-amount { font-family: var(--font-mono); font-weight: var(--fw-medium); text-align: right; }
.td-amount--err { color: var(--color-error); }
.tfoot-row td { font-size: 13px; font-weight: var(--fw-semibold); padding: 14px 16px; background: var(--color-surface-container-low); }

/* Fonds travaux card */
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
.ft-legal {
  font-size: 11px;
  color: var(--color-on-surface-variant);
  background: var(--color-surface-container-low);
  border-radius: var(--radius-sm);
  padding: 10px 12px;
  line-height: 1.5;
  margin-top: 16px;
}
.ft-legal strong { color: var(--color-on-surface); font-weight: var(--fw-medium); }

/* Mini card */
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

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(25,28,29,.4);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  pointer-events: none;
  transition: opacity .2s;
}
.modal-overlay--show { opacity: 1; pointer-events: all; }
.modal {
  background: rgba(255,255,255,.92);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(194,199,207,.3);
  box-shadow: 0 20px 60px rgba(25,28,29,.14);
  border-radius: var(--radius-lg);
  padding: 28px;
  width: 520px;
  max-width: 90vw;
  transform: translateY(8px) scale(.98);
  transition: transform .25s cubic-bezier(.34,1.3,.64,1);
}
.modal-overlay--show .modal { transform: translateY(0) scale(1); }
.modal-title {
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  letter-spacing: -.01em;
  color: var(--color-on-surface);
  margin-bottom: 6px;
}
.modal-sub { font-size: 13px; color: var(--color-on-surface-variant); margin-bottom: 24px; line-height: 1.5; }
.modal-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 24px; }
.field label { font-size: 13px; font-weight: var(--fw-medium); color: var(--color-on-surface); display: block; margin-bottom: 6px; }
.field input,
.field select { width: 100%; padding: 0 14px; height: 42px; background: var(--color-surface-container); border: none; border-radius: var(--radius-md); font-family: var(--font-body); font-size: 14px; color: var(--color-on-surface); outline: none; transition: background .2s, box-shadow .2s; }
.field input:focus,
.field select:focus { background: var(--color-surface-container-low); box-shadow: 0 0 0 2px rgba(100,63,213,.3); }
</style>
