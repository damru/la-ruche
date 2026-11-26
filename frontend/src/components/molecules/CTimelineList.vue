<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CBadge from '@/components/atoms/CBadge.vue'
import CCard from '@/components/molecules/CCard.vue'
import CIconCheckCircle from '@/components/atoms/CIconCheckCircle.svg'
import CIconDot from '@/components/atoms/CIconDot.svg'
import CIconFile from '@/components/atoms/CIconFile.svg'
import CIconLink from '@/components/atoms/CIconLink.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconRefresh from '@/components/atoms/CIconRefresh.svg'
import CIconUser from '@/components/atoms/CIconUser.svg'
import CIconWrench from '@/components/atoms/CIconWrench.svg'
import type { ActivityAction, TimelineEvent } from '@/types/domain'

const props = withDefaults(defineProps<{
  events: TimelineEvent[]
  linkTasks?: boolean
  currentTaskId?: string
}>(), { linkTasks: true, currentTaskId: undefined })

const { t, te } = useI18n()
const { formatDate } = useFormatters()

interface ActionStyle {
  icon: typeof CIconPlus
  dot: string
}

const ACTION_STYLES: Record<ActivityAction, ActionStyle> = {
  created: { icon: CIconPlus, dot: 'bg-primary' },
  status_changed: { icon: CIconRefresh, dot: 'bg-on-surface-variant' },
  photo_added: { icon: CIconFile, dot: 'bg-on-surface-variant' },
  converted_to_task: { icon: CIconLink, dot: 'bg-primary' },
  assignee_changed: { icon: CIconUser, dot: 'bg-warning' },
  contractor_assigned: { icon: CIconWrench, dot: 'bg-warning' },
  vendor_link_generated: { icon: CIconLink, dot: 'bg-on-surface-variant' },
  document_uploaded: { icon: CIconFile, dot: 'bg-on-surface-variant' },
  vendor_submission_received: { icon: CIconWrench, dot: 'bg-warning' },
  closed: { icon: CIconCheckCircle, dot: 'bg-success' },
}

const FALLBACK_STYLE: ActionStyle = { icon: CIconDot, dot: 'bg-outline' }

function styleFor(action: string): ActionStyle {
  return ACTION_STYLES[action as ActivityAction] ?? FALLBACK_STYLE
}

function label(event: TimelineEvent): string {
  if (event.action === 'created') {
    return t(`timeline.actions.created.${event.entity_type}`)
  }
  const key = `timeline.actions.${event.action}`
  return te(key) ? t(key) : t('timeline.actions.unknown', { action: event.action })
}

function statusLabel(entityType: string, status: unknown): string {
  if (typeof status !== 'string') return ''
  const key = entityType === 'report' ? `reports.statuses.${status}` : `tasks.statuses.${status}`
  return t(key)
}

function payloadText(event: TimelineEvent): string | null {
  const payload = event.payload
  if (!payload) return null
  switch (event.action) {
    case 'status_changed': {
      const from = statusLabel(event.entity_type, payload.old_status)
      const to = statusLabel(event.entity_type, payload.new_status)
      return t('timeline.detail.statusTransition', { old: from, new: to })
    }
    case 'contractor_assigned':
      return typeof payload.contractor_name === 'string' ? payload.contractor_name : null
    case 'document_uploaded':
    case 'photo_added':
      return typeof payload.file_name === 'string' ? payload.file_name : null
    case 'vendor_link_generated':
      return typeof payload.expires_at === 'string'
        ? t('timeline.detail.expires', { date: formatDate(payload.expires_at) })
        : null
    case 'created':
      return payload.source_report_id ? t('timeline.detail.fromReport') : null
    default:
      return null
  }
}

function actorLabel(event: TimelineEvent): string | null {
  if (event.actor_name) return event.actor_name
  if (event.action === 'vendor_submission_received') return t('timeline.actorExternalVendor')
  return null
}

const rows = computed(() =>
  [...props.events].reverse().map(event => ({
    event,
    style: styleFor(event.action),
    label: label(event),
    detail: payloadText(event),
    actor: actorLabel(event),
  })))
</script>

<template>
  <ol class="flex flex-col m-0 p-0 list-none">
    <li
      v-for="(row, index) in rows"
      :key="row.event.id"
      class="flex gap-3"
    >
      <div class="flex flex-col items-center shrink-0">
        <div :class="['size-8 rounded-full grid place-items-center text-on-primary z-10 mt-1', row.style.dot, { 'ring-2 ring-primary/20': row.event.action === 'converted_to_task' }]">
          <component
            :is="row.style.icon"
            class="size-4"
          />
        </div>
        <div
          v-if="index < rows.length - 1"
          class="w-0.5 flex-1 bg-outline-variant"
        ></div>
      </div>

      <CCard
        :level="3"
        :class="['flex-1 mb-3', { 'tone--primary border border-primary/30': row.event.action === 'converted_to_task' }]"
      >
        <div class="flex items-start justify-between gap-3">
          <div class="flex flex-col gap-1 min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <span class="text-sm font-medium text-on-surface">{{ row.label }}</span>
              <CBadge
                v-if="row.event.entity_type === 'report'"
                variant="default"
                :title="t('timeline.reportBadge')"
              >
                {{ t('timeline.reportBadge') }}
              </CBadge>
            </div>
            <span
              v-if="row.detail"
              class="text-xs text-on-surface-variant"
            >
              {{ row.detail }}
            </span>
            <span
              v-if="row.actor"
              class="text-2xs text-on-surface-variant"
            >
              {{ row.actor }}
            </span>
          </div>
          <div class="flex flex-col items-end gap-1 shrink-0">
            <span class="text-xs text-on-surface-variant whitespace-nowrap">
              {{ formatDate(row.event.created_at, 'datetime') }}
            </span>
            <RouterLink
              v-if="props.linkTasks
                && row.event.action === 'converted_to_task'
                && typeof row.event.payload?.task_id === 'string'
                && row.event.payload.task_id !== props.currentTaskId"
              :to="{ name: 'task-detail', params: { id: row.event.payload.task_id } }"
              class="text-xs font-medium text-primary hover:underline"
            >
              {{ t('timeline.conversionLink') }}
            </RouterLink>
          </div>
        </div>
      </CCard>
    </li>
  </ol>
</template>
