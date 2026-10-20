<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
import CKpiCard from '@/components/molecules/CKpiCard.vue'
import CCard from '@/components/molecules/CCard.vue'
import CProgressBar from '@/components/atoms/CProgressBar.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import DashboardAgendaCard from './DashboardAgendaCard.vue'
import {
  buildingApi, budgetApi, calendarApi, consultationsApi, contractsApi,
  governanceApi, meetingsApi, reportsApi, tasksApi,
} from '@/api'
import type {
  Budget, BudgetLine, BuildingStats, CalendarEvent, Consultation, ContractReminder,
  GovernanceMandate, LevyCall, Meeting, Report, Task, TaskPriority, VendorContract,
} from '@/types/domain'

const { t } = useI18n()
const auth = useAuthStore()
const { formatCurrency, formatDate, formatRelative, daysUntil } = useFormatters()

// ── State ─────────────────────────────────────────────────────────
const stats = ref<BuildingStats | null>(null)
const currentBudget = ref<Budget | null>(null)
const budgetLines = ref<BudgetLine[]>([])
const levies = ref<LevyCall[]>([])
const meetings = ref<Meeting[]>([])
const tasks = ref<Task[]>([])
const reports = ref<Report[]>([])
const contracts = ref<VendorContract[]>([])
const rawReminders = ref<ContractReminder[]>([])
const rawConsultations = ref<Consultation[]>([])
const governanceMandates = ref<GovernanceMandate[]>([])
const agendaEvents = ref<CalendarEvent[]>([])

// ── Load data ─────────────────────────────────────────────────────
function valueOr<T>(result: PromiseSettledResult<{ data: T }>, fallback: T): T {
  return result.status === 'fulfilled' ? result.value.data : fallback
}

function todayIso(): string {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}
function isoInDays(days: number): string {
  const d = new Date()
  d.setDate(d.getDate() + days)
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

async function loadData() {
  const [statsRes, budgetsRes, meetingsRes, tasksRes, reportsRes, contractsRes, remindersRes, consultationsRes, governanceRes, agendaRes] =
    await Promise.allSettled([
      buildingApi.stats(),
      budgetApi.list(),
      meetingsApi.list(),
      tasksApi.list(),
      reportsApi.list(),
      contractsApi.list(),
      contractsApi.listAllReminders(),
      consultationsApi.list(),
      governanceApi.list(),
      calendarApi.events(todayIso(), isoInDays(30)),
    ])

  // Building stats back the page shell — a full-page error only when they fail.
  if (statsRes.status === 'rejected') throw statsRes.reason
  stats.value = statsRes.value.data

  meetings.value = valueOr(meetingsRes, [])
  tasks.value = valueOr(tasksRes, [])
  reports.value = valueOr(reportsRes, [])
  contracts.value = valueOr(contractsRes, [])
  rawReminders.value = valueOr(remindersRes, [])
  rawConsultations.value = valueOr(consultationsRes, { consultations: [] }).consultations
  governanceMandates.value = valueOr(governanceRes, [])
  agendaEvents.value = valueOr(agendaRes, [])

  const budgets = valueOr(budgetsRes, [])
  const budget = budgets.find(b => b.status === 'approved') ?? budgets[0] ?? null
  currentBudget.value = budget
  if (budget) {
    const [linesRes, leviesRes] = await Promise.allSettled([
      budgetApi.listLines(budget.id),
      budgetApi.listLevies(budget.id),
    ])
    budgetLines.value = valueOr(linesRes, [])
    levies.value = valueOr(leviesRes, [])
  } else {
    budgetLines.value = []
    levies.value = []
  }
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

// ── Derived ───────────────────────────────────────────────────────
const contractsById = computed(() =>
  new Map(contracts.value.map(c => [c.id, c])),
)

const reminders = computed(() =>
  rawReminders.value
    .filter(r => r.status === 'pending')
    .map(r => ({ ...r, contract: contractsById.value.get(r.contract_id) }))
    .sort((a, b) => new Date(a.reminder_date).getTime() - new Date(b.reminder_date).getTime())
    .slice(0, 4),
)

const consultations = computed(() =>
  rawConsultations.value.filter(c => c.status === 'open'),
)

// Active governance mandates — syndic first, then council president, then members.
const governanceContacts = computed(() => {
  const today = todayIso()
  const roleOrder: Record<string, number> = { syndic: 0, council_president: 1, council_member: 2 }
  return governanceMandates.value
    .filter(m => !m.end_date || m.end_date >= today)
    .sort((a, b) => (roleOrder[a.role] ?? 9) - (roleOrder[b.role] ?? 9))
})

function contactInitials(name: string): string {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map(part => part[0]?.toUpperCase() ?? '')
    .join('')
}

const overdueLevies = computed(() => levies.value.filter(l => l.status === 'overdue'))
const nextMeeting = computed(() =>
  meetings.value
    .filter(m => m.status !== 'closed')
    .sort((a, b) => new Date(a.meeting_date).getTime() - new Date(b.meeting_date).getTime())[0]
)
const priorityTasks = computed(() =>
  tasks.value
    .filter(task => task.status !== 'closed' && task.status !== 'cancelled')
    .sort((a, b) => priorityOrder(b.priority) - priorityOrder(a.priority))
    .slice(0, 4)
)
const openReports = computed(() => reports.value.filter(s => s.status !== 'closed').slice(0, 3))
const overdueAmount = computed(() => overdueLevies.value.reduce((sum, l) => sum + Number(l.amount), 0))
const collectionRate = computed(() => {
  if (!levies.value.length) return 100
  const paid = levies.value.filter(l => l.status === 'paid').length
  return Math.round((paid / levies.value.length) * 100)
})
const totalForecast = computed(() => budgetLines.value.reduce((s, l) => s + Number(l.budgeted_amount), 0))
const totalActual = computed(() => budgetLines.value.reduce((s, l) => s + Number(l.actual_amount ?? 0), 0))
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
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div>
          <p class="topbar-greeting-title">
            {{ t('dashboard.hello', { name: auth.user?.first_name }) }} 👋
          </p>
          <p class="topbar-greeting-date">
            {{ todayStr }}{{ stats ? ` · ${stats.name}` : '' }}
          </p>
        </div>
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
        <!-- Hero -->
        <div class="dashboard-hero">
          <header class="view-header">
            <CEyebrow>
              {{ t('page.eyebrows.dashboard') }}
            </CEyebrow>
            <CHeading type="header">
              {{ stats?.name }}
            </CHeading>
            <CMetaLine>
              <CMetaItem variant="sub">
                {{ t('dashboard.lotsMeta', stats?.nb_lots ?? 0) }}
              </CMetaItem>
              <CSeparator />
              <CMetaItem variant="sub">
                {{ t('dashboard.fiscalYearMeta', { year: new Date().getFullYear() }) }}
              </CMetaItem>
            </CMetaLine>
          </header>
          <div class="hero-alerts">
            <div
              v-if="overdueLevies.length"
              class="hero-alert tone--err"
            >
              <CIconWarn aria-hidden="true" />
              <div>
                <strong>{{ t('dashboard.overdueLots', { count: overdueLevies.length }) }}</strong>
                <span>{{ t('dashboard.overdueTotal', { amount: formatCurrency(overdueAmount) }) }}</span>
              </div>
            </div>
            <div
              v-for="r in reminders.slice(0, 2)"
              :key="r.id"
              class="hero-alert tone--warn"
            >
              <CIconWarn aria-hidden="true" />
              <div>
                <strong>{{ r.contract?.vendor_name }} — {{ t(`contracts.reminderTypes.${r.reminder_type}`) }}</strong>
                <span>{{ formatDate(r.reminder_date) }}</span>
              </div>
            </div>
            <div
              v-if="nextMeeting && nextMeeting.status === 'closed'"
              class="hero-alert tone--valid"
            >
              <CIconCheck aria-hidden="true" />
              <div>
                <strong>{{ t('dashboard.lastClosedAg') }}</strong>
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
              <CIconCurrency />
            </template>
            <template #extra>
              <div class="kpi-prog">
                <div
                  class="kpi-prog__fill kpi-prog__fill--primary"
                  :style="`width:${budgetExecutionRate}%`"
                ></div>
              </div>
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
              <CIconCalendar stroke="white" />
            </template>
          </CKpiCard>

          <CKpiCard
            :label="t('dashboard.recoveryRate')"
            :value="String(overdueLevies.length)"
            :delta="overdueLevies.length ? t('dashboard.overdueAmount', { amount: formatCurrency(overdueAmount) }) : t('dashboard.unitsUpToDate')"
            :delta-variant="overdueLevies.length ? 'down' : 'up'"
            :tone="overdueLevies.length ? 'err' : 'valid'"
          >
            <template #icon>
              <CIconWarn />
            </template>
            <template #extra>
              <div class="kpi-prog">
                <div
                  class="kpi-prog__fill"
                  :class="overdueLevies.length ? 'kpi-prog__fill--error' : 'kpi-prog__fill--success'"
                  :style="`width:${collectionRate}%`"
                ></div>
              </div>
            </template>
          </CKpiCard>

          <CKpiCard
            :label="t('dashboard.worksFundLabel')"
            :value="formatCurrency(stats?.works_fund_available ?? 0)"
            :delta="stats?.works_fund_pct ? t('dashboard.worksFundPct', { pct: 100 - stats.works_fund_pct }) : ''"
            delta-variant="neutral"
            tone="warn"
          >
            <template #icon>
              <CIconWrench />
            </template>
            <template #extra>
              <div class="kpi-prog">
                <div
                  class="kpi-prog__fill kpi-prog__fill--warning"
                  :style="`width:${stats?.works_fund_pct ?? 0}%`"
                ></div>
              </div>
            </template>
          </CKpiCard>
        </div>

        <!-- Summary: two columns (2/3 · 1/3). Each column is a flex stack so
             cards take all available vertical space without leaving gaps. -->
        <div class="summary-grid">
          <!-- Colonne principale (2/3) -->
          <div class="summary-col summary-col--main">
            <DashboardAgendaCard :events="agendaEvents" />

            <CCard v-if="currentBudget">
              <CSectionHeader>
                <CHeading type="card">
                  {{ t('dashboard.budgetLabel', { exercice: currentBudget.fiscal_year }) }}
                </CHeading>
                <p class="body-sm text-on-surface-variant">
                  {{ t('dashboard.budgetRealized', { pct: budgetExecutionRate }) }}
                </p>
                <template #actions>
                  <CSectionLink to="/budget">
                    {{ t('dashboard.detailLink') }}
                  </CSectionLink>
                </template>
              </CSectionHeader>
              <div class="budget-prog-list">
                <div
                  v-for="line in budgetLines.slice(0, 4)"
                  :key="line.id"
                  class="budget-prog-item"
                >
                  <div class="budget-prog-header">
                    <span class="budget-prog-label">{{ line.label }}</span>
                    <div class="budget-prog-meta">
                      <span class="budget-prog-amounts">{{ formatCurrency(line.actual_amount ?? 0) }} / {{ formatCurrency(line.budgeted_amount) }}</span>
                      <span
                        class="budget-prog-pct"
                        :class="Number(line.actual_amount ?? 0) > Number(line.budgeted_amount) ? 'pct--over' : 'pct--normal'"
                      >
                        {{ line.budgeted_amount ? Math.round(((line.actual_amount ?? 0) / line.budgeted_amount) * 100) : 0 }}%
                      </span>
                    </div>
                  </div>
                  <CProgressBar
                    :value="line.budgeted_amount ? Math.round(((line.actual_amount ?? 0) / line.budgeted_amount) * 100) : 0"
                    :variant="Number(line.actual_amount ?? 0) > Number(line.budgeted_amount) ? 'error' : 'primary'"
                    :show-value="false"
                  />
                </div>
                <div class="budget-total-row">
                  <span>{{ t('dashboard.totalRealized') }}</span>
                  <span class="budget-total-val">{{ formatCurrency(totalActual) }} / {{ formatCurrency(totalForecast) }}</span>
                </div>
              </div>
            </CCard>

            <!-- Signalements + Consultation côte à côte -->
            <div class="summary-duo">
              <CCard>
                <CSectionHeader>
                  <CHeading type="card">
                    {{ t('dashboard.reportsTitle') }}
                  </CHeading>
                  <p class="body-sm text-on-surface-variant">
                    {{ t('dashboard.openCount', { count: openReports.length }) }}
                  </p>
                  <template #actions>
                    <CSectionLink to="/reports">
                      {{ t('dashboard.seeReports') }}
                    </CSectionLink>
                  </template>
                </CSectionHeader>
                <div class="signal-list">
                  <div
                    v-for="r in openReports"
                    :key="r.id"
                    class="signal-item"
                  >
                    <div class="signal-body">
                      <p class="signal-title">
                        {{ r.title }}
                      </p>
                      <p class="signal-meta">
                        {{ formatDate(r.created_at) }}
                      </p>
                    </div>
                    <CBadge
                      :variant="statusBadgeVariant(r.status)"
                      class="badge-xs"
                      dot
                    >
                      {{ t(`reports.statuses.${r.status}`) }}
                    </CBadge>
                  </div>
                  <div
                    v-if="!openReports.length"
                    class="py-8 text-center text-md text-on-surface-variant"
                  >
                    {{ t('dashboard.noReports') }}
                  </div>
                </div>
                <div class="card-footer">
                  <RouterLink to="/reports">
                    <CBtn
                      variant="primary"
                      size="sm"
                      style="width:100%;justify-content:center"
                      :icon="CIconPlus"
                      :title="t('reports.create')"
                    />
                  </RouterLink>
                </div>
              </CCard>

              <CCard v-if="consultations.length">
                <CSectionHeader>
                  <CHeading type="card">
                    {{ t('dashboard.activeConsultation') }}
                  </CHeading>
                  <p
                    v-if="consultations[0].closed_at"
                    class="body-sm text-on-surface-variant"
                  >
                    {{ t('dashboard.closesPrefix') }} {{ formatRelative(consultations[0].closed_at) }}
                  </p>
                </CSectionHeader>
                <div class="consultation-body">
                  <p class="consultation-title">
                    {{ consultations[0].title }}
                  </p>
                  <p class="consultation-legal">
                    ⚠️ {{ t('consultations.informalNote') }}
                  </p>
                </div>
              </CCard>
            </div>
          </div>

          <!-- Colonne latérale (1/3) -->
          <div class="summary-col">
            <!-- Informations résidence : contacts syndic + conseil syndical -->
            <CCard>
              <CSectionHeader>
                <CHeading type="card">
                  {{ t('dashboard.residenceInfo') }}
                </CHeading>
                <p class="body-sm text-on-surface-variant">
                  {{ t('dashboard.contacts') }}
                </p>
              </CSectionHeader>
              <div class="contact-list">
                <div
                  v-for="m in governanceContacts"
                  :key="m.id"
                  class="contact-item"
                >
                  <div
                    class="contact-avatar"
                    aria-hidden="true"
                  >
                    {{ contactInitials(m.holder_name) }}
                  </div>
                  <div class="contact-body">
                    <p class="contact-name">
                      {{ m.holder_name }}
                    </p>
                    <p class="contact-role">
                      {{ t(`registers.governance.roles.${m.role}`) }}
                    </p>
                  </div>
                </div>
                <div
                  v-if="!governanceContacts.length"
                  class="py-8 text-center text-md text-on-surface-variant"
                >
                  {{ t('dashboard.noContacts') }}
                </div>
              </div>
            </CCard>

            <!-- Rappels contrats -->
            <CCard>
              <CSectionHeader>
                <CHeading type="card">
                  {{ t('dashboard.contractReminders') }}
                </CHeading>
                <p class="body-sm text-on-surface-variant">
                  {{ t('dashboard.nextDeadlines') }}
                </p>
                <template #actions>
                  <CSectionLink to="/contracts">
                    {{ t('dashboard.contracts') }}
                  </CSectionLink>
                </template>
              </CSectionHeader>
              <div class="reminder-list">
                <div
                  v-for="reminder in reminders"
                  :key="reminder.id"
                  class="reminder-item"
                >
                  <div
                    :class="[
                      'reminder-icon',
                      {
                        'tone--err': reminderUrgency(reminder.reminder_date) === 'high',
                        'tone--warn': reminderUrgency(reminder.reminder_date) === 'medium',
                        'tone--muted': reminderUrgency(reminder.reminder_date) === 'low',
                      },
                    ]"
                  >
                    <CIconFile />
                  </div>
                  <div class="reminder-body">
                    <p class="reminder-title">
                      {{ reminder.contract?.vendor_name }}
                    </p>
                    <p class="reminder-sub">
                      {{ t(`contracts.reminderTypes.${reminder.reminder_type}`) }}
                    </p>
                  </div>
                  <span
                    class="reminder-date"
                    :class="`reminder-date--${reminderUrgency(reminder.reminder_date)}`"
                  >
                    {{ formatDate(reminder.reminder_date) }}
                  </span>
                </div>
                <div
                  v-if="!reminders?.length"
                  class="py-8 text-center text-md text-on-surface-variant"
                >
                  {{ t('dashboard.noReminders') }}
                </div>
              </div>
            </CCard>

            <!-- Tâches prioritaires -->
            <CCard>
              <CSectionHeader>
                <CHeading type="card">
                  {{ t('dashboard.priorityTasks') }}
                </CHeading>
                <p class="body-sm text-on-surface-variant">
                  {{ t('dashboard.taskCount', { count: priorityTasks.length }) }}
                </p>
                <template #actions>
                  <CSectionLink to="/tasks">
                    {{ t('dashboard.allTasks') }}
                  </CSectionLink>
                </template>
              </CSectionHeader>
              <div class="task-list">
                <div
                  v-for="task in priorityTasks"
                  :key="task.id"
                  class="task-item"
                >
                  <div
                    :class="['task-prio', `task-prio--${task.priority}`]"
                    aria-hidden="true"
                  ></div>
                  <div class="task-body">
                    <p class="task-title">
                      {{ task.title }}
                    </p>
                    <div class="task-meta">
                      <span class="task-date">{{ formatRelative(task.created_at) }}</span>
                    </div>
                  </div>
                  <CBadge
                    :variant="statusBadgeVariant(task.status)"
                    class="badge-xs"
                  >
                    {{ t(`tasks.statuses.${task.status}`) }}
                  </CBadge>
                </div>
                <div
                  v-if="!priorityTasks.length"
                  class="py-8 text-center text-md text-on-surface-variant"
                >
                  {{ t('dashboard.noTasks') }}
                </div>
              </div>
              <div class="card-footer">
                <RouterLink to="/tasks">
                  <CBtn
                    variant="primary"
                    size="sm"
                    style="width:100%;justify-content:center"
                    :icon="CIconPlus"
                    :title="t('dashboard.newTask')"
                  />
                </RouterLink>
              </div>
            </CCard>

            <!-- Résidents -->
            <CCard>
              <CSectionHeader>
                <CHeading type="card">
                  {{ t('dashboard.residentsTitle') }}
                </CHeading>
                <template #actions>
                  <CSectionLink to="/residents">
                    {{ t('dashboard.seeResidents') }}
                  </CSectionLink>
                </template>
              </CSectionHeader>
              <div class="residents-grid">
                <div class="res-stat">
                  <p class="res-stat-val">
                    {{ stats?.nb_lots }}
                  </p><p class="res-stat-label">
                    {{ t('dashboard.lotsLabel') }}
                  </p>
                </div>
                <div class="res-stat">
                  <p class="res-stat-val">
                    {{ levies.filter(l => l.status === 'paid').length }}
                  </p><p class="res-stat-label">
                    {{ t('dashboard.paidLabel') }}
                  </p>
                </div>
                <div class="res-stat res-stat--ok">
                  <p class="res-stat-val">
                    {{ collectionRate }}%
                  </p><p class="res-stat-label">
                    {{ t('dashboard.recoveryLabel') }}
                  </p>
                </div>
                <div class="res-stat res-stat--err">
                  <p class="res-stat-val">
                    {{ overdueLevies.length }}
                  </p><p class="res-stat-label">
                    {{ t('dashboard.lateLabel') }}
                  </p>
                </div>
              </div>
            </CCard>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
/* ── Hero ─────────────────────────────────────────────────────────── */
.dashboard-hero { display: grid; grid-template-columns: 1fr auto; gap: var(--space-6); align-items: end; }
@media (max-width: 900px) { .dashboard-hero { grid-template-columns: 1fr; } }

.hero-alerts { display: flex; flex-direction: column; gap: var(--space-2); align-items: flex-end; }
@media (max-width: 900px) { .hero-alerts { align-items: stretch; } .hero-alert { min-width: 0; } }

.hero-alert {
  display: flex; align-items: center; gap: var(--space-2);
  padding: var(--space-3) var(--space-4); border-radius: var(--radius-xl); min-width: 280px; font-size: var(--text-sm);
  background: var(--tone-bg, var(--color-surface-container-low));
  color: var(--tone-fg, var(--color-on-surface));
}
.hero-alert svg { width: var(--icon-sm); height: var(--icon-sm); flex-shrink: 0; }
.hero-alert > div { min-width: 0; }
.hero-alert div strong { font-weight: var(--fw-semibold); display: block; }
.hero-alert div span  { font-size: var(--text-2xs); opacity: 0.85; }

/* ── KPI ──────────────────────────────────────────────────────────── */
.kpi-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: var(--space-4); }
@media (max-width: 1100px) { .kpi-grid { grid-template-columns: repeat(2, 1fr); } }

.kpi-prog { height: 3px; border-radius: var(--radius-full); background: var(--color-surface-container-high); overflow: hidden; margin-top: var(--space-3); }
.kpi-prog__fill { height: 100%; border-radius: var(--radius-full); transition: width 0.8s cubic-bezier(0.34, 1.3, 0.64, 1); }
.kpi-prog__fill--primary { background: var(--gradient-primary); }
.kpi-prog__fill--success { background: var(--gradient-success); }
.kpi-prog__fill--error   { background: var(--gradient-error); }
.kpi-prog__fill--warning { background: var(--gradient-warning); }

/* ── Summary layout ──────────────────────────────────────────────────
   Two columns (2/3 · 1/3). Each column is a flex stack: cards flow
   vertically without leaving empty grid cells (no row-based placement).
   Tracks use minmax(0, 1fr) to keep shrinkable minimums (no overflow). */
.summary-grid { display: grid; grid-template-columns: minmax(0, 2fr) minmax(0, 1fr); gap: var(--space-6); align-items: start; }
.summary-col { display: flex; flex-direction: column; gap: var(--space-6); min-width: 0; }
.summary-duo { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-6); }
@media (max-width: 1100px) { .summary-grid { grid-template-columns: minmax(0, 1fr); } }
@media (max-width: 768px) { .summary-duo { grid-template-columns: minmax(0, 1fr); } }

/* ── Badges ───────────────────────────────────────────────────────── */
.badge-xs { font-size: var(--text-xs); }

/* ── Contacts (résidence) ─────────────────────────────────────────── */
.contact-list { display: flex; flex-direction: column; gap: var(--space-1); }
.contact-item { display: flex; align-items: center; gap: 10px; padding: 8px var(--space-3); border-radius: var(--radius-xl); transition: background var(--transition-fast); }
.contact-item:hover { background: var(--color-surface-container-low); }
.contact-avatar { width: 32px; height: 32px; min-width: 32px; border-radius: var(--radius-full); display: flex; align-items: center; justify-content: center; flex-shrink: 0; background: var(--gradient-primary); color: white; font-size: var(--text-2xs); font-weight: var(--fw-semibold); letter-spacing: 0.02em; }
.contact-body { flex: 1; min-width: 0; }
.contact-name { font-size: var(--text-xs); font-weight: var(--fw-medium); color: var(--color-on-surface); }
.contact-role  { font-size: var(--text-2xs); color: var(--color-on-surface-variant); margin-top: 1px; }

/* ── Tasks ────────────────────────────────────────────────────────── */
.task-list { display: flex; flex-direction: column; gap: var(--space-2); }
.task-item { display: flex; align-items: flex-start; gap: 10px; padding: 10px var(--space-3); border-radius: var(--radius-xl); background: var(--color-surface-container-low); cursor: pointer; transition: background var(--transition-fast); }
.task-item:hover { background: var(--color-surface-container); }
.task-prio { width: 4px; height: 32px; border-radius: var(--radius-full); flex-shrink: 0; margin-top: 2px; }
.task-prio--urgent { background: var(--color-error); }
.task-prio--high   { background: var(--color-warning); }
.task-prio--normal { background: var(--color-primary); }
.task-prio--low    { background: var(--color-outline-variant); }
.task-body { flex: 1; min-width: 0; }
.task-title { font-size: var(--text-sm); font-weight: var(--fw-medium); color: var(--color-on-surface); margin-bottom: 3px; }
.task-meta  { display: flex; align-items: center; gap: var(--space-2); }
.task-date  { font-size: var(--text-2xs); color: var(--color-on-surface-variant); }

/* ── Reminders ────────────────────────────────────────────────────── */
.reminder-list { display: flex; flex-direction: column; gap: var(--space-1); }
.reminder-item { display: flex; align-items: center; gap: 10px; padding: 10px var(--space-3); border-radius: var(--radius-xl); transition: background var(--transition-fast); cursor: pointer; }
.reminder-item:hover { background: var(--color-surface-container-low); }
.reminder-icon { width: 32px; height: 32px; min-width: unset; min-height: unset; border-radius: var(--radius-lg); display: flex; align-items: center; justify-content: center; flex-shrink: 0; background: var(--tone-bg, transparent); color: var(--tone-fg, inherit); }
.reminder-icon svg { width: 15px; height: 15px; }
.reminder-body { flex: 1; min-width: 0; }
.reminder-title { font-size: var(--text-xs); font-weight: var(--fw-medium); color: var(--color-on-surface); }
.reminder-sub   { font-size: var(--text-2xs); color: var(--color-on-surface-variant); margin-top: 1px; }
.reminder-date { font-family: var(--font-mono); font-size: var(--text-2xs); font-weight: var(--fw-medium); flex-shrink: 0; }
.reminder-date--high   { color: var(--color-error); }
.reminder-date--medium { color: var(--color-warning); }
.reminder-date--low    { color: var(--color-on-surface-variant); }

/* ── Signals ──────────────────────────────────────────────────────── */
.signal-list { display: flex; flex-direction: column; gap: var(--space-1); }
.signal-item { display: flex; align-items: center; gap: 10px; padding: 10px 0; }
.signal-body { flex: 1; min-width: 0; }
.signal-title { font-size: var(--text-xs); font-weight: var(--fw-medium); color: var(--color-on-surface); }
.signal-meta  { font-size: var(--text-2xs); color: var(--color-on-surface-variant); margin-top: 1px; }

/* ── Budget progress ──────────────────────────────────────────────── */
.budget-prog-list { display: flex; flex-direction: column; gap: var(--space-4); }
.budget-prog-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-1); }
.budget-prog-label { font-size: var(--text-sm); font-weight: var(--fw-medium); color: var(--color-on-surface); }
.budget-prog-meta { display: flex; align-items: center; gap: 10px; flex-shrink: 0; }
.budget-prog-amounts { font-family: var(--font-mono); font-size: var(--text-2xs); color: var(--color-on-surface-variant); }
.budget-prog-pct { font-size: var(--text-xs); font-weight: var(--fw-semibold); min-width: 36px; text-align: right; }
.pct--normal { color: var(--color-primary); }
.pct--over   { color: var(--color-error); }
.budget-total-row { display: flex; justify-content: space-between; align-items: center; padding: var(--space-3) var(--space-4); background: var(--color-surface-container-low); border-radius: var(--radius-xl); font-size: var(--text-sm); font-weight: var(--fw-medium); color: var(--color-on-surface); }
.budget-total-val { font-family: var(--font-mono); font-size: var(--text-sm); font-weight: var(--fw-medium); }

/* ── Residents ────────────────────────────────────────────────────── */
.residents-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-3); }
.res-stat { background: var(--color-surface-container-low); border-radius: var(--radius-xl); padding: 14px; }
.res-stat-val { font-family: var(--font-display); font-size: 26px; font-weight: var(--fw-bold); font-variation-settings: 'opsz' 72; letter-spacing: -0.03em; color: var(--color-on-surface); margin-bottom: 2px; }
.res-stat--ok .res-stat-val  { color: var(--color-success); }
.res-stat--err .res-stat-val { color: var(--color-error); }
.res-stat-label { font-size: var(--text-2xs); color: var(--color-on-surface-variant); }

/* ── Consultation ─────────────────────────────────────────────────── */
.consultation-body { display: flex; flex-direction: column; gap: var(--space-2); }
.consultation-title { font-size: var(--text-md); font-weight: var(--fw-medium); color: var(--color-on-surface); }
.consultation-legal { font-size: var(--text-2xs); color: var(--color-on-surface-variant); }
</style>

