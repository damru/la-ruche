<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader } from '@/composables/usePageLoader'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconArrowLeft from '@/components/atoms/CIconArrowLeft.svg'
import CIconCheckCircle from '@/components/atoms/CIconCheckCircle.svg'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'
import CIconLink from '@/components/atoms/CIconLink.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconRefresh from '@/components/atoms/CIconRefresh.svg'
import CIconUser from '@/components/atoms/CIconUser.svg'
import CIconWrench from '@/components/atoms/CIconWrench.svg'
import { buildingApi, contractorsApi, maintenanceApi, tasksApi } from '@/api'
import type { Contractor, MaintenanceEntry, Task } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatRelative, formatCurrency } = useFormatters()
const route = useRoute()
const router = useRouter()

const taskId = computed(() => route.params.id as string)
const buildingName = ref('')
const task = ref<Task | null>(null)
const contractor = ref<Contractor | null>(null)
const logbookEntries = ref<MaintenanceEntry[]>([])

// ── Load data ─────────────────────────────────────────────────────
async function loadData() {
  const [buildingRes, taskRes, entriesRes] = await Promise.all([
    buildingApi.get(),
    tasksApi.get(taskId.value),
    maintenanceApi.list(),
  ])
  buildingName.value = buildingRes.data.name
  task.value = taskRes.data

  logbookEntries.value = entriesRes.data
    .filter(entry => entry.task_id === taskId.value)
    .sort((left, right) => left.intervention_date.localeCompare(right.intervention_date))

  // Load contractor if assigned
  if (taskRes.data.contractor_id) {
    const cRes = await contractorsApi.get(taskRes.data.contractor_id)
    contractor.value = cRes.data
  } else {
    contractor.value = null
  }
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

// ── Helpers ───────────────────────────────────────────────────────
function statusVariant(s: string): 'valid' | 'open' | 'pending' | 'closed' | 'default' {
  if (s === 'open') return 'valid'
  if (s === 'in_progress') return 'open'
  if (s === 'awaiting_vendor') return 'default'
  if (s === 'awaiting_validation') return 'pending'
  if (s === 'closed') return 'closed'
  return 'default'
}

function priorityVariant(p: string): 'err' | 'pending' | 'primary' | 'valid' | 'default' {
  if (p === 'urgent') return 'err'
  if (p === 'high') return 'pending'
  if (p === 'normal') return 'primary'
  if (p === 'low') return 'valid'
  return 'default'
}

// ── Lifecycle timeline events ──────────────────────────────────────
interface TimelineEvent {
  key: string
  date?: string
  label: string
  detail?: string
  icon: unknown
  variant?: 'primary' | 'success' | 'warning' | 'error' | 'secondary'
}

const timelineEvents = computed((): TimelineEvent[] => {
  if (!task.value) return []
  const t_ = task.value
  const events: TimelineEvent[] = []

  // 1. Creation
  events.push({
    key: 'created',
    date: t_.created_at,
    label: t('tasks.lifecycle.created'),
    icon: CIconPlus,
    variant: 'primary',
  })

  // 2. Contractor assignment or self-assignment (no timestamp — shown as undated step)
  if ((t_.contractor_id && contractor.value) || t_.contractor_name) {
    events.push({
      key: 'assigned_contractor',
      label: t('tasks.lifecycle.assignedContractor'),
      detail: contractor.value?.name ?? t_.contractor_name,
      icon: CIconWrench,
      variant: 'warning',
    })
  } else if (t_.status === 'in_progress' || t_.assigned_to) {
    events.push({
      key: 'assigned_self',
      label: t('tasks.lifecycle.assignedSelf'),
      icon: CIconUser,
      variant: 'primary',
    })
  }

  // 3. Vendor link active
  if (t_.has_vendor_token && t_.token_expires_at) {
    events.push({
      key: 'vendor_link',
      date: t_.token_expires_at,
      label: t('tasks.lifecycle.vendorLink'),
      detail: t('tasks.lifecycle.vendorLinkExpiry', { date: formatDate(t_.token_expires_at) }),
      icon: CIconLink,
      variant: 'secondary',
    })
  }

  // 4. Logbook entries
  for (const logbookEntry of logbookEntries.value) {
    let logbookDetail: string | undefined
    if (logbookEntry.vendor_name) {
      logbookDetail = `🔧 ${logbookEntry.vendor_name}`
      if (logbookEntry.cost != null) {
        logbookDetail += ` · ${formatCurrency(logbookEntry.cost)}`
      }
    } else {
      logbookDetail = logbookEntry.description ?? undefined
    }
    events.push({
      key: `logbook-${logbookEntry.id}`,
      date: logbookEntry.intervention_date,
      label: logbookEntry.title,
      detail: logbookDetail,
      icon: CIconCheckCircle,
      variant: 'success',
    })
  }

  // 5. Closure
  if (t_.closed_at) {
    events.push({
      key: 'closed',
      date: t_.closed_at,
      label: t('tasks.lifecycle.closed'),
      icon: CIconCheckCircle,
      variant: 'success',
    })
  }

  return events
})

function dotClass(variant?: TimelineEvent['variant']): string {
  switch (variant) {
    case 'success': return 'timeline-dot--success'
    case 'warning': return 'timeline-dot--warning'
    case 'error': return 'timeline-dot--error'
    case 'secondary': return 'timeline-dot--secondary'
    default: return ''
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
          <RouterLink
            :to="{ name: 'tasks' }"
            class="topbar-breadcrumb-link"
          >
            {{ t('tasks.title') }}
          </RouterLink>
          <CIconChevronRight />
          <span>{{ task?.title ?? '…' }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="ghost"
          size="sm"
          @click="router.push({ name: 'tasks' })"
        >
          <CIconArrowLeft style="width:16px;height:16px" />
          {{ t('common.back') }}
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
      <template v-else-if="task">
        <!-- Task header card -->
        <header class="view-header">
          <CEyebrow>
            <CBadge :variant="priorityVariant(task.priority)">
              {{ t(`tasks.priorities.${task.priority}`) }}
            </CBadge>
            <CBadge
              :variant="statusVariant(task.status)"
              dot
            >
              {{ t(`tasks.statuses.${task.status}`) }}
            </CBadge>
            {{ t('page.eyebrows.taskDetail') }}
          </CEyebrow>
          <h1 class="display-md task-title">
            {{ task.title }}
          </h1>
          <p
            v-if="task.description"
            class="body-md task-desc"
          >
            {{ task.description }}
          </p>
          <div class="task-meta">
            <span>{{ t('tasks.lifecycle.createdRelative', { relative: formatRelative(task.created_at) }) }}</span>
            <span v-if="task.closed_at">
              · {{ t('tasks.closedAt', { date: formatDate(task.closed_at) }) }}
            </span>
            <span v-if="contractor || task.contractor_name">
              · <RouterLink
                v-if="contractor"
                :to="{ name: 'contractor-detail', params: { id: contractor.id } }"
                class="contractor-link"
              >🔧 {{ contractor.name }}</RouterLink>
              <span
                v-else
                class="contractor-link contractor-link--text"
              >🔧 {{ task.contractor_name }}</span>
            </span>
          </div>
        </header>

        <!-- Lifecycle timeline -->
        <h2 class="section-title title-sm">
          {{ t('tasks.lifecycle.title') }}
        </h2>
        <div class="timeline">
          <div
            v-for="(ev, idx) in timelineEvents"
            :key="ev.key"
            class="timeline-item"
            :class="{ 'timeline-item--last': idx === timelineEvents.length - 1 }"
          >
            <div :class="['timeline-dot', dotClass(ev.variant)]">
              <component
                :is="ev.icon"
                class="timeline-icon"
              />
            </div>
            <CCard class="timeline-card">
              <div class="event-row">
                <span class="title-sm event-label">{{ ev.label }}</span>
                <span
                  v-if="ev.date"
                  class="event-date"
                >{{ formatDate(ev.date) }}</span>
              </div>
              <p
                v-if="ev.detail"
                class="body-xs event-detail"
              >
                {{ ev.detail }}
              </p>
              <!-- Contractor link for assigned_contractor event -->
              <RouterLink
                v-if="ev.key === 'assigned_contractor' && contractor"
                :to="{ name: 'contractor-detail', params: { id: contractor.id } }"
                class="contractor-link body-xs"
              >
                {{ contractor.email || contractor.phone || '' }}
              </RouterLink>
            </CCard>
          </div>
        </div>

        <!-- Reopening indicator for closed tasks -->
        <div
          v-if="task.status === 'closed'"
          class="reopen-hint body-xs"
        >
          <CIconRefresh
            width="13"
            height="13"
          />
          {{ t('tasks.lifecycle.closedNote') }}
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
/* ── Task header card ────────────────────────────────────────────── */
.task-header-card {
  display: flex;
  align-items: stretch;
  padding: 0;
  overflow: hidden;
  margin-bottom: var(--space-6);
}

.task-title { margin: 0; }
.task-desc { color: var(--color-on-surface-variant); margin: 0; }
.task-meta {
  font-size: var(--text-2xs);
  color: var(--color-on-surface-variant);
  margin-top: var(--space-1);
}

/* ── Section title ───────────────────────────────────────────────── */
.section-title {
  margin-bottom: var(--space-4);
  color: var(--color-on-surface-variant);
}

/* ── Timeline ────────────────────────────────────────────────────── */
.timeline { display: flex; flex-direction: column; gap: 0; }

.timeline-item {
  display: flex;
  gap: var(--space-4);
  align-items: stretch;
  padding-bottom: var(--space-4);
  position: relative;
}

.timeline-item::before {
  content: '';
  position: absolute;
  left: 17px;
  top: 36px;
  bottom: 0;
  width: 2px;
  background: var(--color-outline-variant);
}

.timeline-item--last::before { display: none; }

.timeline-dot {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--color-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1;
  position: relative;
  margin-top: 4px;
}

.timeline-dot--success  { background: var(--color-success); }
.timeline-dot--warning  { background: var(--color-warning); }
.timeline-dot--error    { background: var(--color-error); }
.timeline-dot--secondary { background: var(--color-secondary, var(--color-on-surface-variant)); }

.timeline-icon {
  width: 16px;
  height: 16px;
  color: #fff;
  flex-shrink: 0;
}

.timeline-card { flex: 1; }

.event-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
}

.event-label { flex: 1; }

.event-date {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
  flex-shrink: 0;
}

.event-detail {
  color: var(--color-on-surface-variant);
  margin-top: var(--space-1);
}

/* ── Contractor link ─────────────────────────────────────────────── */
.contractor-link {
  color: var(--color-primary);
  text-decoration: none;
  font-weight: var(--fw-medium);
}
.contractor-link:hover { text-decoration: underline; }

/* ── Reopen hint ─────────────────────────────────────────────────── */
.reopen-hint {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  color: var(--color-on-surface-variant);
  margin-top: var(--space-2);
}
</style>
