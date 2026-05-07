<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useAuthStore } from '@/stores/auth'
import type { Meeting, MeetingType } from '@/types/domain'
import { buildingApi, meetingsApi } from '@/api'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CSeparator from '@/components/atoms/CSeparator.vue'
import CCard from '@/components/molecules/CCard.vue'
import CCardMeta from '@/components/molecules/CCardMeta.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'

const router = useRouter()
const { t } = useI18n()
const { formatDate } = useFormatters()
const auth = useAuthStore()

const buildingName = ref('')
const allMeetings = ref<Meeting[]>([])
const filter = ref<'all' | 'upcoming' | 'closed'>('all')

const canManage = computed(() => auth.isManager)

const showCreateModal = ref(false)
const createFeedback = useFormFeedback()
const saving = ref(false)
const { schedule: scheduleCreateClose, forceExpire: forceCreateClose } = useAutoClose()
watch(showCreateModal, (open) => { if (!open) forceCreateClose() })

function defaultMeetingForm() {
  return {
    meeting_type: 'annual' as MeetingType,
    meeting_date: '',
    location: '',
    agenda: '',
  }
}

const createForm = ref(defaultMeetingForm())

async function loadData() {
  const [buildingRes, meetingsRes] = await Promise.all([
    buildingApi.get(),
    meetingsApi.list(),
  ])
  buildingName.value = buildingRes.data.name
  allMeetings.value = meetingsRes.data.slice().sort((left, right) => right.meeting_date.localeCompare(left.meeting_date))
}

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
onMounted(reload)

const meetings = computed(() => {
  if (filter.value === 'upcoming') return allMeetings.value.filter(meeting => meeting.status !== 'closed')
  if (filter.value === 'closed') return allMeetings.value.filter(meeting => meeting.status === 'closed')
  return allMeetings.value
})

const upcomingCount = computed(() => allMeetings.value.filter(meeting => meeting.status !== 'closed').length)
const closedCount = computed(() => allMeetings.value.filter(meeting => meeting.status === 'closed').length)

function typeLabel(type: MeetingType): string {
  return t(`meetings.typeLabel.${type}`)
}

function statusLabel(status: Meeting['status']): string {
  return t(`meetings.statuses.${status}`)
}

function statusChipClass(status: Meeting['status']): string {
  if (status === 'called') return 'tone--default'
  if (status === 'closed') return 'tone--valid'
  return 'tone--disabled'
}

function meetingBadgeLabel(type: MeetingType): string {
  return type === 'annual' ? 'AG' : 'AGE'
}

function openMeeting(meetingId: string) {
  router.push(`/meetings/${meetingId}`)
}

function openCreateModal() {
  createForm.value = defaultMeetingForm()
  createFeedback.reset()
  showCreateModal.value = true
}

async function saveMeeting() {
  if (createFeedback.status.value === 'success') {
    forceCreateClose()
    return
  }

  if (!createForm.value.meeting_date) {
    createFeedback.setError(t('common.error'))
    return
  }

  saving.value = true
  createFeedback.reset()

  try {
    const response = await meetingsApi.create({
      meeting_type: createForm.value.meeting_type,
      meeting_date: new Date(createForm.value.meeting_date).toISOString(),
      location: createForm.value.location || undefined,
      agenda: createForm.value.agenda || undefined,
    })

    createFeedback.setSuccess(t('common.saved'))
    scheduleCreateClose(async () => {
      showCreateModal.value = false
      await reload()
      router.push(`/meetings/${response.data.id}`)
    })
  } catch (err) {
    createFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName || '...' }}
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('meetings.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canManage"
          variant="primary"
          size="sm"
          @click="openCreateModal"
        >
          <CIconPlus />
          <span class="btn-label">{{ t('meetings.create') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container assembly-list-page">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else>
        <header class="view-header">
          <p class="page-eyebrow">
            {{ t('page.eyebrows.assemblies') }}
          </p>
          <h1 class="display-md">
            {{ t('meetings.title') }}
          </h1>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ buildingName }} · {{ t('meetings.count', allMeetings.length) }}
          </p>
        </header>

        <div class="tabs">
          <button
            :class="['tab', { 'tab--active': filter === 'all' }]"
            @click="filter = 'all'"
          >
            {{ t('common.allFem') }}
            <span class="tab-count">{{ allMeetings.length }}</span>
          </button>
          <button
            :class="['tab', { 'tab--active': filter === 'upcoming' }]"
            @click="filter = 'upcoming'"
          >
            {{ t('meetings.filterUpcoming') }}
            <span class="tab-count">{{ upcomingCount }}</span>
          </button>
          <button
            :class="['tab', { 'tab--active': filter === 'closed' }]"
            @click="filter = 'closed'"
          >
            {{ t('meetings.filterClosed') }}
            <span class="tab-count">{{ closedCount }}</span>
          </button>
        </div>

        <div
          v-if="meetings.length === 0"
          class="empty-state"
        >
          {{ t('meetings.empty') }}
        </div>

        <div
          v-else
          class="meeting-list"
        >
          <CCard
            v-for="meeting in meetings"
            :key="meeting.id"
            padding="none"
            interactive
            class="activity-card meeting-card"
            role="button"
            tabindex="0"
            @click="openMeeting(meeting.id)"
            @keydown.enter.prevent="openMeeting(meeting.id)"
            @keydown.space.prevent="openMeeting(meeting.id)"
          >
            <div :class="['meeting-badge', { 'meeting-badge--closed': meeting.status === 'closed' }]">
              <span class="meeting-badge__type">{{ meetingBadgeLabel(meeting.meeting_type) }}</span>
              <span class="meeting-badge__year">{{ new Date(meeting.meeting_date).getFullYear() }}</span>
            </div>

            <div class="meeting-card__main">
              <div class="meeting-card__title-row">
                <h2 class="activity-card__title meeting-card__title">
                  {{ typeLabel(meeting.meeting_type) }} {{ new Date(meeting.meeting_date).getFullYear() }}
                </h2>
                <span :class="['status-chip', statusChipClass(meeting.status)]">
                  <span class="status-chip__dot"></span>
                  {{ statusLabel(meeting.status) }}
                </span>
              </div>

              <CCardMeta>
                <span class="activity-card__line">
                  <CIconCalendar />
                  {{ formatDate(meeting.meeting_date) }}
                </span>
                <CSeparator />
                <span
                  v-if="meeting.location"
                  class="activity-card__line"
                >
                  <CIconLocation />
                  {{ meeting.location }}
                </span>
              </CCardMeta>

              <div class="meeting-card__resolution-row body-xs">
                <span class="meeting-card__res-count">
                  <span class="text-headline-sm">{{ meeting.resolution_count }}</span>
                  {{ t('meetings.statResolutions', meeting.resolution_count) }}
                </span>
                <span
                  v-if="meeting.status === 'closed'"
                  class="meeting-card__res-passed tone--valid"
                >
                  <span class="text-headline-sm">{{ meeting.passed_count }}</span>
                  {{ t('meetings.statPassed', meeting.passed_count) }}
                </span>
                <span
                  v-else
                  class="text-muted"
                >
                  — {{ t('meetings.statPendingVotes') }}
                </span>
              </div>
            </div>
          </CCard>
        </div>
      </template>
    </div>

    <CModal
      :open="showCreateModal"
      :title="t('meetings.create')"
      @update:open="showCreateModal = $event"
    >
      <form
        class="form-grid"
        @submit.prevent="saveMeeting"
      >
        <label class="form-field">
          <span class="form-label">{{ t('meetings.infoFields.type') }}</span>
          <select
            v-model="createForm.meeting_type"
            class="form-input"
          >
            <option value="annual">{{ t('meetings.types.annual') }}</option>
            <option value="extraordinary">{{ t('meetings.types.extraordinary') }}</option>
          </select>
        </label>

        <label class="form-field">
          <span class="form-label">{{ t('meetings.meetingDate') }}</span>
          <input
            v-model="createForm.meeting_date"
            class="form-input"
            type="datetime-local"
          />
        </label>

        <label class="form-field">
          <span class="form-label">{{ t('meetings.location') }}</span>
          <input
            v-model="createForm.location"
            class="form-input"
            type="text"
          />
        </label>

        <label class="form-field">
          <span class="form-label">{{ t('meetings.agenda') }}</span>
          <textarea
            v-model="createForm.agenda"
            class="form-input"
          ></textarea>
        </label>
      </form>

      <template #footer>
        <CFormFeedback
          :status="createFeedback.status.value"
          :message="createFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showCreateModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="saving"
          :closing="createFeedback.status.value === 'success'"
          @click="saveMeeting"
        >
          {{ saving ? t('common.saving') : t('common.save') }}
        </CBtn>
      </template>
    </CModal>
  </div>
</template>

<style scoped>
.assembly-list-page {
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
}


.meeting-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.meeting-card {
  width: 100%;
  display: flex;
  flex-direction: row;
  align-items: stretch;
  overflow: hidden;
  gap: 0;
}

.meeting-card__main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-5);
}

.meeting-badge {
  width: 88px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 4px;
  padding: var(--space-4);
  background: var(--gradient-primary);
  color: white;
}

.meeting-badge--closed {
  background: linear-gradient(135deg, #1b5e20, #43a047);
}

.meeting-badge__type {
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--fw-bold);
}

.meeting-badge__year {
  font-family: var(--font-mono);
  font-size: 16px;
  font-weight: var(--fw-bold);
}

.meeting-card__title-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--space-4);
}

.meeting-card__title {
  font-family: var(--font-display);
}

.meeting-card__resolution-row {
  display: flex;
  align-items: baseline;
  gap: var(--space-3);
  flex-wrap: wrap;
  color: var(--color-on-surface-variant);
}

.meeting-card__res-count {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
}

.meeting-card__res-passed {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  color: var(--tone-fg, inherit);
}

.status-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: var(--radius-full);
  font-size: var(--text-2xs);
  font-weight: var(--fw-medium);
  background: var(--tone-bg, transparent);
  color: var(--tone-fg, inherit);
}

.status-chip__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.7;
}

.meeting-card__title-row {
  flex-wrap: wrap;
}

@media (max-width: 900px) {
  .meeting-badge {
    width: 72px;
  }

  .meeting-card__title-row {
    flex-direction: column;
  }
}

</style>
