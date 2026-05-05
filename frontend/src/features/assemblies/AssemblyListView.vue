<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CMockBanner from '@/components/atoms/CMockBanner.vue'
import { mockMeetings, mockResolutions, mockBuildingStats } from '@/mock/data'

const router = useRouter()
const { t } = useI18n()
const { formatDate } = useFormatters()

const filter = ref<'all' | 'upcoming' | 'closed'>('all')

const meetings = computed(() => {
  if (filter.value === 'upcoming') return mockMeetings.filter(m => m.status !== 'closed')
  if (filter.value === 'closed') return mockMeetings.filter(m => m.status === 'closed')
  return mockMeetings
})

function resolutionCount(meetingId: string): number {
  return mockResolutions.filter(r => r.meeting_id === meetingId).length
}
function passedCount(meetingId: string): number {
  return mockResolutions.filter(r => r.meeting_id === meetingId && r.status === 'passed').length
}
function statusLabel(s: string): string {
  return t(`meetings.statuses.${s}`)
}
function statusChipClass(s: string): string {
  if (s === 'called') return 'tone--default'
  if (s === 'closed') return 'tone--valid'
  return 'tone--disabled'
}
function typeLabel(type: string): string {
  return t(`meetings.typeLabel.${type === 'annual' ? 'annual' : 'extraordinary'}`)
}

</script>

<template>
  <div class="assembly-list-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ mockBuildingStats.name }}
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('meetings.eyebrow') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
        >
          <CIconPlus />
          <span class="btn-label">{{ t('meetings.callNew') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="content">
      <CMockBanner />

      <!-- Header -->
      <div class="page-header">
        <p class="page-eyebrow">
          {{ t('meetings.eyebrow') }}
        </p>
        <h1 class="page-title">
          {{ t('meetings.title') }}
        </h1>
        <p class="page-sub">
          {{ mockBuildingStats.name }} · {{ t('meetings.count', mockMeetings.length) }}
        </p>
      </div>

      <!-- Filters -->
      <div class="tabs">
        <button
          :class="['tab', { 'tab--active': filter === 'all' }]"
          @click="filter = 'all'"
        >
          {{ t('common.allFem') }}
          <span class="tab-count">{{ mockMeetings.length }}</span>
        </button>
        <button
          :class="['tab', { 'tab--active': filter === 'upcoming' }]"
          @click="filter = 'upcoming'"
        >
          {{ t('meetings.filterUpcoming') }}
        </button>
        <button
          :class="['tab', { 'tab--active': filter === 'closed' }]"
          @click="filter = 'closed'"
        >
          {{ t('meetings.filterClosed') }}
        </button>
      </div>

      <!-- Meeting cards -->
      <div class="meeting-list">
        <div
          v-for="meeting in meetings"
          :key="meeting.id"
          class="meeting-card"
          @click="router.push(`/meetings/${meeting.id}`)"
        >
          <!-- Badge -->
          <div :class="['meeting-badge', meeting.status === 'closed' ? 'meeting-badge--closed' : '']">
            <span class="badge-type">{{ meeting.meeting_type === 'annual' ? 'AG' : 'AGE' }}</span>
            <span class="badge-year">{{ new Date(meeting.meeting_date).getFullYear() }}</span>
          </div>

          <!-- Info -->
          <div class="meeting-info">
            <div class="meeting-top">
              <div>
                <h2 class="meeting-title">
                  {{ typeLabel(meeting.meeting_type) }} {{ new Date(meeting.meeting_date).getFullYear() }}
                </h2>
                <div class="meeting-meta">
                  <span class="meta-item">
                    <CIconCalendar />
                    {{ formatDate(meeting.meeting_date) }}
                  </span>
                  <span class="meta-sep"></span>
                  <span class="meta-item">
                    <CIconLocation />
                    {{ meeting.location }}
                  </span>
                </div>
              </div>
              <span :class="['chip', statusChipClass(meeting.status)]">
                <span class="chip-dot"></span>
                {{ statusLabel(meeting.status) }}
              </span>
            </div>

            <!-- Stats -->
            <div class="meeting-stats">
              <div class="stat">
                <span class="stat-value">{{ resolutionCount(meeting.id) }}</span>
                <span class="stat-label">{{ t('meetings.statResolutions', resolutionCount(meeting.id)) }}</span>
              </div>
              <div
                v-if="meeting.status === 'closed'"
                class="stat"
              >
                <span class="stat-value tone--valid">{{ passedCount(meeting.id) }}</span>
                <span class="stat-label">{{ t('meetings.statPassed', passedCount(meeting.id)) }}</span>
              </div>
              <div
                v-if="meeting.status !== 'closed'"
                class="stat"
              >
                <span class="stat-value tone--disabled">—</span>
                <span class="stat-label">{{ t('meetings.statPendingVotes') }}</span>
              </div>
            </div>
          </div>

          <!-- Chevron -->
          <div class="meeting-chevron">
            <CIconChevronRight />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.assembly-list-page {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: var(--color-surface);
}
.content {
  padding: var(--space-10);
  flex: 1;
  max-width: 900px;
}

/* Header */
.page-eyebrow {
  font-size: var(--text-2xs);
  font-weight: var(--fw-semibold);
  letter-spacing: .1em;
  text-transform: uppercase;
  color: var(--color-primary);
  margin-bottom: var(--space-2);
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
.page-sub { font-size: var(--text-md); color: var(--color-on-surface-variant); margin-top: 6px; margin-bottom: var(--space-8); }
.tabs { margin-bottom: var(--space-8); }

/* Meeting list */
.meeting-list { display: flex; flex-direction: column; gap: var(--space-3); }
.meeting-card {
  display: flex;
  align-items: stretch;
  gap: 0;
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  overflow: hidden;
  cursor: pointer;
  transition: box-shadow .2s ease, transform .2s ease;
}
.meeting-card:hover { box-shadow: var(--shadow-lg); transform: translateY(-1px); }

/* Badge (colored left strip) */
.meeting-badge {
  width: 80px;
  flex-shrink: 0;
  background: var(--gradient-primary);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-5) var(--space-3);
  gap: var(--space-1);
}
.meeting-badge--closed { background: linear-gradient(135deg, #1b5e20, #43a047); }
.badge-type {
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--fw-bold);
  color: rgba(255,255,255,.9);
  letter-spacing: .02em;
}
.badge-year {
  font-family: var(--font-mono);
  font-size: 16px;
  font-weight: var(--fw-bold);
  color: #fff;
}

/* Info */
.meeting-info { flex: 1; padding: var(--space-5) var(--space-6); display: flex; flex-direction: column; gap: var(--space-4); }
.meeting-top { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--space-4); }
.meeting-title {
  font-family: var(--font-display);
  font-size: 17px;
  font-weight: var(--fw-semibold);
  font-variation-settings: 'opsz' 36;
  letter-spacing: -.01em;
  color: var(--color-on-surface);
  margin-bottom: var(--space-2);
}
.meeting-meta { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
.meta-item { display: flex; align-items: center; gap: 5px; font-size: var(--text-sm); color: var(--color-on-surface-variant); }
.meta-item svg { width: 13px; height: 13px; flex-shrink: 0; }
.meta-sep { width: 3px; height: 3px; border-radius: 50%; background: var(--color-outline); }

/* Stats */
.meeting-stats { display: flex; gap: var(--space-6); }
.stat { display: flex; align-items: baseline; gap: 5px; }
.stat-value {
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: var(--fw-bold);
  font-variation-settings: 'opsz' 36;
  letter-spacing: -.02em;
  color: var(--tone-fg, var(--color-on-surface));
}
.stat-label { font-size: var(--text-xs); color: var(--color-on-surface-variant); }

/* Chevron */
.meeting-chevron {
  display: flex;
  align-items: center;
  padding: 0 var(--space-5);
  color: var(--color-on-surface-variant);
}
.meeting-chevron svg { width: 18px; height: 18px; }

/* Chips */
.chip { display: inline-flex; align-items: center; gap: 5px; padding: var(--space-1) 10px; border-radius: var(--radius-full); font-size: var(--text-2xs); font-weight: var(--fw-medium); white-space: nowrap; background: var(--tone-bg, transparent); color: var(--tone-fg, inherit); }
.chip-dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; opacity: .65; flex-shrink: 0; }

</style>
