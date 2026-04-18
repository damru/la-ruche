<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useFormatters } from '@/composables/useFormatters'
import CKpiCard from '@/components/molecules/CKpiCard.vue'
import CCard from '@/components/molecules/CCard.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.vue'
import CIconWarn from '@/components/atoms/CIconWarn.vue'
import CIconCheck from '@/components/atoms/CIconCheck.vue'
import CIconCurrency from '@/components/atoms/CIconCurrency.vue'
import CIconCalendar from '@/components/atoms/CIconCalendar.vue'
import CIconWrench from '@/components/atoms/CIconWrench.vue'
import CIconPlus from '@/components/atoms/CIconPlus.vue'
import CIconFile from '@/components/atoms/CIconFile.vue'
import {
  mockBuildingStats, mockBudgets, mockBudgetLines, mockLevies, mockMeetings,
  mockTasks, mockReports, mockContracts, mockReminders, mockConsultations,
} from '@/mock/data'
import type { TaskPriority } from '@/types/domain'

const { t } = useI18n()
const auth = useAuthStore()
const { formatCurrency, formatDate, formatRelative, daysUntil } = useFormatters()

const stats = mockBuildingStats
const currentBudget = mockBudgets.find(b => b.status === 'approved') ?? mockBudgets[0]
const budgetLines = mockBudgetLines.filter(l => l.budget_id === currentBudget.id)
const levies = mockLevies
const meetings = mockMeetings
const tasks = mockTasks
const reports = mockReports
const consultations = mockConsultations.filter(c => c.status === 'open')

const reminders = mockReminders
  .map(r => ({ ...r, contract: mockContracts.find(c => c.id === r.contract_id) }))
  .filter(r => r.status === 'pending')
  .sort((a, b) => new Date(a.reminder_date).getTime() - new Date(b.reminder_date).getTime())
  .slice(0, 4)

const overdueLevies = computed(() => levies.filter(a => a.status === 'overdue'))
const nextMeeting = computed(() =>
  meetings
    .filter(m => m.status !== 'closed')
    .sort((a, b) => new Date(a.meeting_date).getTime() - new Date(b.meeting_date).getTime())[0]
)
const priorityTasks = computed(() =>
  tasks
    .filter(t => t.status !== 'closed' && t.status !== 'cancelled')
    .sort((a, b) => priorityOrder(b.priority) - priorityOrder(a.priority))
    .slice(0, 4)
)
const openReports = computed(() => reports.filter(s => s.status !== 'closed').slice(0, 3))
const overdueAmount = computed(() => overdueLevies.value.reduce((sum, a) => sum + a.amount, 0))
const collectionRate = computed(() => {
  if (!levies.length) return 100
  const paid = levies.filter(a => a.status === 'paid').length
  return Math.round((paid / levies.length) * 100)
})
const totalForecast = computed(() => budgetLines.reduce((s, l) => s + l.budgeted_amount, 0))
const totalActual = computed(() => budgetLines.reduce((s, l) => s + (l.actual_amount ?? 0), 0))
const budgetExecutionRate = computed(() =>
  totalForecast.value ? Math.round((totalActual.value / totalForecast.value) * 100) : 0
)

function priorityOrder(p: TaskPriority): number {
  return { urgent: 4, high: 3, normal: 2, low: 1 }[p] ?? 0
}
function statusBadgeVariant(s: string): 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'open') return 'pending'
  if (s === 'in_progress') return 'open'
  if (s === 'closed') return 'closed'
  return 'default'
}
function reminderUrgency(dateStr: string): 'high' | 'medium' | 'low' {
  const days = daysUntil(dateStr)
  if (days <= 14) return 'high'
  if (days <= 45) return 'medium'
  return 'low'
}
const todayStr = computed(() =>
  new Intl.DateTimeFormat('fr-FR', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' }).format(new Date())
)
</script>

<template>
  <div class="dashboard-page">

    <CTopbar>
      <template #left>
        <div>
          <p class="topbar-greeting-title">Bonjour, {{ auth.user?.first_name }} 👋</p>
          <p class="topbar-greeting-date">{{ todayStr }} · {{ stats.name }}</p>
        </div>
      </template>
      <template #right>
        <RouterLink to="/tasks">
          <CBtn variant="secondary" size="sm">
            <CIconPlus/>
            Nouvelle action
          </CBtn>
        </RouterLink>
      </template>
    </CTopbar>

    <div class="dashboard-content">
      <CMockBanner />

      <!-- Hero -->
      <div class="hero">
        <div class="hero-left">
          <p class="hero-eyebrow">Vue d'ensemble</p>
          <h1 class="hero-title">{{ stats.name }}</h1>
          <p class="hero-sub">{{ stats.nb_lots }} lots · Exercice {{ new Date().getFullYear() }} en cours</p>
        </div>
        <div class="hero-right">
          <div v-if="overdueLevies.length" class="hero-alert hero-alert--err">
            <CIconWarn aria-hidden="true"/>
            <div>
              <strong>{{ overdueLevies.length }} lot{{ overdueLevies.length > 1 ? 's' : '' }} en retard de paiement</strong>
              <span>Total : {{ formatCurrency(overdueAmount) }}</span>
            </div>
          </div>
          <div v-for="r in reminders.slice(0, 2)" :key="r.id" class="hero-alert hero-alert--warn">
            <CIconWarn aria-hidden="true"/>
            <div>
              <strong>{{ r.contract?.vendor_name }} — {{ t(`contracts.reminderTypes.${r.reminder_type}`) }}</strong>
              <span>{{ formatDate(r.reminder_date) }}</span>
            </div>
          </div>
          <div v-if="nextMeeting && nextMeeting.status === 'closed'" class="hero-alert hero-alert--ok">
            <CIconCheck aria-hidden="true"/>
            <div>
              <strong>Dernière AG clôturée</strong>
              <span>{{ formatDate(nextMeeting.meeting_date, 'long') }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- KPIs -->
      <div class="kpi-grid">
        <CKpiCard
          :label="t('dashboard.budgetLabel', { exercice: new Date().getFullYear() })"
          :value="formatCurrency(totalActual)"
          :delta="`sur ${formatCurrency(totalForecast)}`"
          delta-variant="neutral"
        >
          <template #icon>
            <CIconCurrency/>
          </template>
          <template #extra>
            <div class="kpi-prog"><div class="kpi-prog__fill kpi-prog__fill--primary" :style="`width:${budgetExecutionRate}%`"/></div>
          </template>
        </CKpiCard>

        <CKpiCard
          :label="t('dashboard.nextAg')"
          :value="nextMeeting ? formatDate(nextMeeting.meeting_date, 'long') : t('dashboard.nextAgNone')"
          :delta="nextMeeting ? formatRelative(nextMeeting.meeting_date) : ''"
          delta-variant="neutral"
          :icon-accent="true"
        >
          <template #icon>
            <CIconCalendar stroke="white"/>
          </template>
        </CKpiCard>

        <CKpiCard
          :label="t('dashboard.recoveryRate')"
          :value="String(overdueLevies.length)"
          :delta="overdueLevies.length ? `${formatCurrency(overdueAmount)} en attente` : t('dashboard.unitsUpToDate')"
          :delta-variant="overdueLevies.length ? 'down' : 'up'"
        >
          <template #icon>
            <CIconWarn/>
          </template>
          <template #extra>
            <div class="kpi-prog">
              <div class="kpi-prog__fill" :class="overdueLevies.length ? 'kpi-prog__fill--error' : 'kpi-prog__fill--success'" :style="`width:${collectionRate}%`"/>
            </div>
          </template>
        </CKpiCard>

        <CKpiCard
          :label="t('dashboard.worksFundLabel')"
          :value="formatCurrency(stats?.works_fund_available ?? 0)"
          :delta="stats?.works_fund_pct ? `${100 - stats.works_fund_pct}% disponible` : ''"
          delta-variant="neutral"
        >
          <template #icon>
            <CIconWrench/>
          </template>
          <template #extra>
            <div class="kpi-prog"><div class="kpi-prog__fill kpi-prog__fill--warning" :style="`width:${stats.works_fund_pct ?? 0}%`"/></div>
          </template>
        </CKpiCard>
      </div>

      <!-- Main 3-col grid -->
      <div class="grid3">

        <!-- Priority tasks -->
        <CCard padding="none">
          <div class="card-head">
            <div>
              <h2 class="card-title">{{ t('dashboard.priorityTasks') }}</h2>
              <p class="card-sub">{{ priorityTasks.length }} ouverte{{ priorityTasks.length !== 1 ? 's' : '' }}</p>
            </div>
            <RouterLink to="/tasks" class="card-link">
              Toutes <CIconChevronRight/>
            </RouterLink>
          </div>
          <div class="task-list">
            <div v-for="task in priorityTasks" :key="task.id" class="task-item">
              <div :class="['task-prio', `task-prio--${task.priority}`]" aria-hidden="true"/>
              <div class="task-body">
                <p class="task-title">{{ task.title }}</p>
                <div class="task-meta">
                  <span class="task-date">{{ formatRelative(task.created_at) }}</span>
                </div>
              </div>
              <CBadge :variant="statusBadgeVariant(task.status)">
                {{ t(`tasks.statuses.${task.status}`) }}
              </CBadge>
            </div>
            <div v-if="!priorityTasks.length" class="empty-state">{{ t('dashboard.noTasks') }}</div>
          </div>
          <div class="card-footer">
            <RouterLink to="/tasks">
              <CBtn variant="primary" size="sm" style="width:100%;justify-content:center">
                <CIconPlus/>
                Nouvelle tâche
              </CBtn>
            </RouterLink>
          </div>
        </CCard>

        <!-- Contract reminders + Reports -->
        <CCard padding="none">
          <div class="card-head">
            <div>
              <h2 class="card-title">{{ t('dashboard.contractReminders') }}</h2>
              <p class="card-sub">{{ t('dashboard.nextDeadlines') }}</p>
            </div>
            <RouterLink to="/contracts" class="card-link">
              Contrats <CIconChevronRight/>
            </RouterLink>
          </div>
          <div class="reminder-list">
            <div
              v-for="reminder in reminders"
              :key="reminder.id"
              class="reminder-item"
            >
              <div class="reminder-icon" :class="`reminder-icon--${reminderUrgency(reminder.reminder_date)}`">
                <CIconFile/>
              </div>
              <div class="reminder-body">
                <p class="reminder-title">{{ (reminder as any).contract?.vendor_name }}</p>
                <p class="reminder-sub">{{ t(`contracts.reminderTypes.${reminder.reminder_type}`) }}</p>
              </div>
              <span class="reminder-date" :class="`reminder-date--${reminderUrgency(reminder.reminder_date)}`">
                {{ formatDate(reminder.reminder_date) }}
              </span>
            </div>
            <div v-if="!reminders?.length" class="empty-state">{{ t('dashboard.noReminders') }}</div>
          </div>

          <div class="section-divider"/>
          <div class="card-head" style="padding-top:16px">
            <div>
              <h2 class="card-title">{{ t('dashboard.reportsTitle') }}</h2>
              <p class="card-sub">{{ openReports.length }} ouvert{{ openReports.length !== 1 ? 's' : '' }}</p>
            </div>
            <RouterLink to="/reports" class="card-link">
              Voir <CIconChevronRight/>
            </RouterLink>
          </div>
          <div class="signal-list">
            <div v-for="r in openReports" :key="r.id" class="signal-item">
              <span :class="['signal-dot', `signal-dot--${r.status}`]" aria-hidden="true"/>
              <div class="signal-body">
                <p class="signal-title">{{ r.title }}</p>
                <p class="signal-meta">{{ formatDate(r.created_at) }}</p>
              </div>
              <CBadge :variant="statusBadgeVariant(r.status)" dot>
                {{ t(`reports.statuses.${r.status}`) }}
              </CBadge>
            </div>
            <div v-if="!openReports.length" class="empty-state">{{ t('dashboard.noReports') }}</div>
          </div>
        </CCard>

        <!-- Right col: residents + consultation -->
        <div class="right-col">
          <CCard padding="none">
            <div class="card-head">
              <h2 class="card-title">Résidents</h2>
              <RouterLink to="/residents" class="card-link">
                Gérer <CIconChevronRight/>
              </RouterLink>
            </div>
            <div class="residents-grid">
              <div class="res-stat"><p class="res-stat-val">{{ stats.nb_lots }}</p><p class="res-stat-label">Lots</p></div>
              <div class="res-stat"><p class="res-stat-val">{{ levies.filter(l => l.status === 'paid').length }}</p><p class="res-stat-label">Payés</p></div>
              <div class="res-stat res-stat--ok"><p class="res-stat-val">{{ collectionRate }}%</p><p class="res-stat-label">Recouvrement</p></div>
              <div class="res-stat res-stat--err"><p class="res-stat-val">{{ overdueLevies.length }}</p><p class="res-stat-label">En retard</p></div>
            </div>
          </CCard>

          <CCard v-if="consultations.length" padding="none">
            <div class="card-head">
              <div>
                <h2 class="card-title">Consultation active</h2>
                <p v-if="consultations[0].closed_at" class="card-sub">Clôture {{ formatRelative(consultations[0].closed_at) }}</p>
              </div>
            </div>
            <div class="consultation-body">
              <p class="consultation-title">{{ consultations[0].title }}</p>
              <p class="consultation-legal">⚠️ {{ t('consultations.informalNote') }}</p>
            </div>
          </CCard>
        </div>

        <!-- Budget — spans 2 cols -->
        <CCard v-if="currentBudget" padding="none" class="budget-wide">
          <div class="card-head">
            <div>
              <h2 class="card-title">Budget {{ currentBudget.fiscal_year }}</h2>
              <p class="card-sub">{{ budgetExecutionRate }}% exécuté</p>
            </div>
            <RouterLink to="/budget" class="card-link">
              Détail <CIconChevronRight/>
            </RouterLink>
          </div>
          <div class="budget-prog-list">
            <div v-for="line in budgetLines.slice(0, 4)" :key="line.id" class="budget-prog-item">
              <div class="budget-prog-header">
                <span class="budget-prog-label">{{ line.label }}</span>
                <div class="budget-prog-meta">
                  <span class="budget-prog-amounts">{{ formatCurrency(line.actual_amount ?? 0) }} / {{ formatCurrency(line.budgeted_amount) }}</span>
                  <span class="budget-prog-pct" :class="(line.actual_amount ?? 0) > line.budgeted_amount ? 'pct--over' : 'pct--normal'">
                    {{ line.budgeted_amount ? Math.round(((line.actual_amount ?? 0) / line.budgeted_amount) * 100) : 0 }}%
                  </span>
                </div>
              </div>
              <div class="prog-track">
                <div
                  class="prog-fill"
                  :class="(line.actual_amount ?? 0) > line.budgeted_amount ? 'prog-fill--over' : 'prog-fill--primary'"
                  :style="`width:${Math.min(100, line.budgeted_amount ? Math.round(((line.actual_amount ?? 0) / line.budgeted_amount) * 100) : 0)}%`"
                />
              </div>
            </div>
            <div class="budget-total-row">
              <span>Total réalisé</span>
              <span class="budget-total-val">{{ formatCurrency(totalActual) }} / {{ formatCurrency(totalForecast) }}</span>
            </div>
          </div>
        </CCard>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard-page { display: flex; flex-direction: column; flex: 1; }

.dashboard-content {
  padding: var(--space-10);
  display: flex;
  flex-direction: column;
  gap: 28px;
  max-width: 1400px;
}

/* ── Hero ─────────────────────────────────────────────────────────── */
.hero { display: grid; grid-template-columns: 1fr auto; gap: var(--space-6); align-items: end; }

.hero-eyebrow {
  font-size: 11px; font-weight: var(--fw-semibold); letter-spacing: 0.1em;
  text-transform: uppercase; color: var(--color-primary); margin-bottom: var(--space-2);
}

.hero-title {
  font-family: var(--font-display); font-size: 42px; font-weight: var(--fw-bold);
  font-variation-settings: 'opsz' 144; letter-spacing: -0.04em; line-height: 0.95;
  color: var(--color-on-surface); margin-bottom: var(--space-2);
}

.hero-sub { font-size: 14px; color: var(--color-on-surface-variant); max-width: 480px; line-height: 1.5; }

.hero-right { display: flex; flex-direction: column; gap: var(--space-2); align-items: flex-end; }

.hero-alert {
  display: flex; align-items: center; gap: var(--space-2);
  padding: 12px var(--space-4); border-radius: var(--radius-md); min-width: 280px; font-size: 13px;
}
.hero-alert svg { width: 16px; height: 16px; flex-shrink: 0; }
.hero-alert div strong { font-weight: var(--fw-semibold); display: block; }
.hero-alert div span  { font-size: 11px; opacity: 0.85; }
.hero-alert--err  { background: var(--color-error-container);   color: var(--color-error); }
.hero-alert--warn { background: var(--color-warning-container); color: var(--color-warning); }
.hero-alert--ok   { background: var(--color-success-container); color: var(--color-success); }

/* ── KPI ──────────────────────────────────────────────────────────── */
.kpi-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: var(--space-4); }
@media (max-width: 1100px) { .kpi-grid { grid-template-columns: repeat(2, 1fr); } }

.kpi-prog { height: 3px; border-radius: var(--radius-full); background: var(--color-surface-container-high); overflow: hidden; margin-top: var(--space-3); }
.kpi-prog__fill { height: 100%; border-radius: var(--radius-full); transition: width 0.8s cubic-bezier(0.34, 1.3, 0.64, 1); }
.kpi-prog__fill--primary { background: var(--gradient-primary); }
.kpi-prog__fill--success { background: linear-gradient(135deg, #1b5e20, #43a047); }
.kpi-prog__fill--error   { background: linear-gradient(135deg, #ba1a1a, #e53935); }
.kpi-prog__fill--warning { background: linear-gradient(135deg, #e65100, #ff9800); }

/* ── Grid ─────────────────────────────────────────────────────────── */
.grid3 { display: grid; grid-template-columns: 1fr 1fr 320px; gap: var(--space-6); align-items: start; }
.budget-wide { grid-column: span 2; }
@media (max-width: 1200px) { .grid3 { grid-template-columns: 1fr 1fr; } }
@media (max-width: 768px)  { .grid3 { grid-template-columns: 1fr; } }
.right-col { display: flex; flex-direction: column; gap: var(--space-4); }

/* ── Card helpers ─────────────────────────────────────────────────── */
.card-head { display: flex; align-items: flex-start; justify-content: space-between; padding: var(--space-5) var(--space-5) 0; margin-bottom: var(--space-4); }
.card-title { font-family: var(--font-display); font-size: 16px; font-weight: var(--fw-semibold); font-variation-settings: 'opsz' 36; letter-spacing: -0.01em; color: var(--color-on-surface); }
.card-sub { font-size: 12px; color: var(--color-on-surface-variant); margin-top: 2px; }
.card-link { font-size: 12px; color: var(--color-primary); font-weight: var(--fw-medium); text-decoration: none; display: flex; align-items: center; gap: 3px; white-space: nowrap; flex-shrink: 0; }
.card-link svg { width: 12px; height: 12px; }
.card-link:hover { text-decoration: underline; }
.card-footer { padding: var(--space-2) var(--space-5) var(--space-5); }
.section-divider { height: 1px; background: rgba(194,199,207,.2); margin: var(--space-2) var(--space-5) 0; }
.empty-state { padding: var(--space-4) var(--space-5); font-size: 13px; color: var(--color-on-surface-variant); }

/* ── Tasks ────────────────────────────────────────────────────────── */
.task-list { padding: 0 var(--space-3) var(--space-2); display: flex; flex-direction: column; gap: var(--space-2); }
.task-item { display: flex; align-items: flex-start; gap: 10px; padding: 10px 12px; border-radius: var(--radius-md); background: var(--color-surface-container-low); cursor: pointer; transition: background var(--transition-fast); }
.task-item:hover { background: var(--color-surface-container); }
.task-prio { width: 4px; height: 32px; border-radius: var(--radius-full); flex-shrink: 0; margin-top: 2px; }
.task-prio--urgent { background: var(--color-error); }
.task-prio--high   { background: var(--color-warning); }
.task-prio--normal { background: var(--color-primary); }
.task-prio--low    { background: var(--color-outline-variant); }
.task-body { flex: 1; min-width: 0; }
.task-title { font-size: 13px; font-weight: var(--fw-medium); color: var(--color-on-surface); margin-bottom: 3px; }
.task-meta  { display: flex; align-items: center; gap: var(--space-2); }
.task-date  { font-size: 11px; color: var(--color-on-surface-variant); }

/* ── Reminders ────────────────────────────────────────────────────── */
.reminder-list { padding: 0 var(--space-5) var(--space-2); display: flex; flex-direction: column; gap: var(--space-1); }
.reminder-item { display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: var(--radius-md); transition: background var(--transition-fast); cursor: pointer; }
.reminder-item:hover { background: var(--color-surface-container-low); }
.reminder-icon { width: 32px; height: 32px; min-width: unset; min-height: unset; border-radius: var(--radius-sm); display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
.reminder-icon svg { width: 15px; height: 15px; }
.reminder-icon--high   { background: var(--color-error-container);   color: var(--color-error); }
.reminder-icon--medium { background: var(--color-warning-container); color: var(--color-warning); }
.reminder-icon--low    { background: var(--color-surface-container); color: var(--color-on-surface-variant); }
.reminder-body { flex: 1; min-width: 0; }
.reminder-title { font-size: 12px; font-weight: var(--fw-medium); color: var(--color-on-surface); }
.reminder-sub   { font-size: 11px; color: var(--color-on-surface-variant); margin-top: 1px; }
.reminder-date { font-family: var(--font-mono); font-size: 11px; font-weight: var(--fw-medium); flex-shrink: 0; }
.reminder-date--high   { color: var(--color-error); }
.reminder-date--medium { color: var(--color-warning); }
.reminder-date--low    { color: var(--color-on-surface-variant); }

/* ── Signals ──────────────────────────────────────────────────────── */
.signal-list { padding: 0 var(--space-5) var(--space-4); display: flex; flex-direction: column; gap: var(--space-1); }
.signal-item { display: flex; align-items: center; gap: 10px; padding: 10px 0; border-bottom: 1px solid rgba(194,199,207,.12); }
.signal-item:last-child { border-bottom: none; }
.signal-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.signal-dot--open        { background: var(--color-warning); }
.signal-dot--in_progress { background: var(--color-primary); }
.signal-dot--closed      { background: var(--color-success); }
.signal-body { flex: 1; min-width: 0; }
.signal-title { font-size: 12px; font-weight: var(--fw-medium); color: var(--color-on-surface); }
.signal-meta  { font-size: 11px; color: var(--color-on-surface-variant); margin-top: 1px; }

/* ── Budget progress ──────────────────────────────────────────────── */
.budget-prog-list { padding: 0 var(--space-6) var(--space-5); display: flex; flex-direction: column; gap: var(--space-4); }
.budget-prog-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-1); }
.budget-prog-label { font-size: 13px; font-weight: var(--fw-medium); color: var(--color-on-surface); }
.budget-prog-meta { display: flex; align-items: center; gap: 10px; flex-shrink: 0; }
.budget-prog-amounts { font-family: var(--font-mono); font-size: 11px; color: var(--color-on-surface-variant); }
.budget-prog-pct { font-size: 12px; font-weight: var(--fw-semibold); min-width: 36px; text-align: right; }
.pct--normal { color: var(--color-primary); }
.pct--over   { color: var(--color-error); }
.prog-track { height: 8px; border-radius: var(--radius-full); background: var(--color-surface-container-high); overflow: hidden; }
.prog-fill { height: 100%; border-radius: var(--radius-full); transition: width 0.8s cubic-bezier(0.34, 1.3, 0.64, 1); }
.prog-fill--primary { background: var(--gradient-primary); }
.prog-fill--over    { background: linear-gradient(135deg, #ba1a1a, #e53935); }
.budget-total-row { display: flex; justify-content: space-between; align-items: center; padding: 12px var(--space-4); background: var(--color-surface-container-low); border-radius: var(--radius-md); font-size: 13px; font-weight: var(--fw-medium); color: var(--color-on-surface); }
.budget-total-val { font-family: var(--font-mono); font-size: 13px; font-weight: var(--fw-medium); }

/* ── Residents ────────────────────────────────────────────────────── */
.residents-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; padding: 0 var(--space-5) var(--space-5); }
.res-stat { background: var(--color-surface-container-low); border-radius: var(--radius-md); padding: 14px; }
.res-stat-val { font-family: var(--font-display); font-size: 26px; font-weight: var(--fw-bold); font-variation-settings: 'opsz' 72; letter-spacing: -0.03em; color: var(--color-on-surface); margin-bottom: 2px; }
.res-stat--ok .res-stat-val  { color: var(--color-success); }
.res-stat--err .res-stat-val { color: var(--color-error); }
.res-stat-label { font-size: 11px; color: var(--color-on-surface-variant); }

/* ── Consultation ─────────────────────────────────────────────────── */
.consultation-body { padding: 0 var(--space-5) var(--space-5); display: flex; flex-direction: column; gap: var(--space-2); }
.consultation-title { font-size: 14px; font-weight: var(--fw-medium); color: var(--color-on-surface); }
.consultation-legal { font-size: 11px; color: var(--color-on-surface-variant); }
</style>
