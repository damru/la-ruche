<script setup lang="ts">
import { useI18n } from 'vue-i18n';

export type MeetingStatus = 'draft' | 'called' | 'ongoing' | 'closed'

const { t } = useI18n()

const props = defineProps<{
  status: MeetingStatus
  label?: string
}>()

const toneClass: Record<MeetingStatus, string> = {
  draft:   'tone--muted',
  called:  'tone--info',
  ongoing: 'tone--accent',
  closed:  'tone--valid',
}

const defaultLabel: Record<MeetingStatus, string> = {
  draft:   t('meetings.statuses.draft'),
  called:  t('meetings.statuses.called'),
  ongoing: t('meetings.statuses.ongoing'),
  closed:  t('meetings.statuses.closed'),
}
</script>

<template>
  <div
    :class="toneClass[props.status]"
    class="flex items-center gap-2 px-4 py-2 rounded-radius-md text-sm font-medium"
    style="background: var(--tone-bg); color: var(--tone-fg);"
    role="status"
  >
    <span
      class="size-2 rounded-radius-full shrink-0"
      style="background: currentColor; opacity: 0.6;"
    ></span>
    {{ props.label ?? defaultLabel[props.status] }}
  </div>
</template>
