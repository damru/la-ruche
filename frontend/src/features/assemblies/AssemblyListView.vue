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

function noticeDeadlineLabel(meetingDate: string): string {
  const date = new Date(meetingDate)
  date.setDate(date.getDate() - 21)
  return formatDate(date.toISOString())
}

function agendaExcerpt(agenda?: string): string {
  if (!agenda?.trim()) return '—'
  const normalized = agenda.replace(/\s+/g, ' ').trim()
  return normalized.length > 140 ? `${normalized.slice(0, 137)}...` : normalized
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

        <div class="filter-row">
          <button
            :class="['filter-pill', { 'filter-pill--active': filter === 'all' }]"
            @click="filter = 'all'"
          >
            {{ t('common.allFem') }}
            <span>{{ allMeetings.length }}</span>
          </button>
          <button
            :class="['filter-pill', { 'filter-pill--active': filter === 'upcoming' }]"
            @click="filter = 'upcoming'"
          >
            {{ t('meetings.filterUpcoming') }}
            <span>{{ upcomingCount }}</span>
          </button>
          <button
            :class="['filter-pill', { 'filter-pill--active': filter === 'closed' }]"
            @click="filter = 'closed'"
          >
            {{ t('meetings.filterClosed') }}
            <span>{{ closedCount }}</span>
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
          <button
            v-for="meeting in meetings"
            :key="meeting.id"
            class="meeting-card"
            @click="router.push(`/meetings/${meeting.id}`)"
          >
            <div :class="['meeting-badge', { 'meeting-badge--closed': meeting.status === 'closed' }]">
              <span class="meeting-badge__type">{{ meetingBadgeLabel(meeting.meeting_type) }}</span>
              <span class="meeting-badge__year">{{ new Date(meeting.meeting_date).getFullYear() }}</span>
            </div>

            <div class="meeting-card__body">
              <div class="meeting-card__header">
                <div>
                  <h2 class="meeting-card__title">
                    {{ typeLabel(meeting.meeting_type) }} {{ new Date(meeting.meeting_date).getFullYear() }}
                  </h2>
                  <div class="meeting-card__meta">
                    <span class="activity-card__line">
                      <CIconCalendar />
                      {{ formatDate(meeting.meeting_date) }}
                    </span>
                    <span
                      v-if="meeting.location"
                      class="activity-card__line"
                    >
                      <CIconLocation />
                      {{ meeting.location }}
                    </span>
                  </div>
                </div>
                <span :class="['status-chip', statusChipClass(meeting.status)]">
                  <span class="status-chip__dot"></span>
                  {{ statusLabel(meeting.status) }}
                </span>
              </div>

              <p class="meeting-card__agenda">
                {{ agendaExcerpt(meeting.agenda) }}
              </p>

              <div class="meeting-card__footer">
                <span class="footer-pill">
                  {{ t('meetings.calledAt', { date: meeting.notice_date ? formatDate(meeting.notice_date) : '—' }) }}
                </span>
                <span class="footer-pill">
                  J-21 : {{ noticeDeadlineLabel(meeting.meeting_date) }}
                </span>
              </div>
            </div>

            <div class="meeting-card__chevron">
              <CIconChevronRight />
            </div>
          </button>
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
          @click="saveMeeting"
        >
          {{ t('common.save') }}
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

.filter-row {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.filter-pill {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0 var(--space-4);
  min-height: 40px;
  border: none;
  border-radius: var(--radius-full);
  background: var(--color-surface-container-low);
  color: var(--color-on-surface-variant);
  font-weight: var(--fw-medium);
  cursor: pointer;
}

.filter-pill span {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.filter-pill--active {
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
}

.meeting-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.meeting-card {
  display: flex;
  width: 100%;
  border: none;
  padding: 0;
  background: var(--color-surface-container-lowest);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  overflow: hidden;
  text-align: left;
  cursor: pointer;
}

.meeting-card:hover {
  box-shadow: var(--shadow-lg);
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
}

.meeting-card__body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-5);
}

.meeting-card__header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--space-4);
}

.meeting-card__title {
  font-family: var(--font-display);
  font-size: 18px;
  font-weight: var(--fw-semibold);
  margin-bottom: var(--space-2);
}

.meeting-card__meta {
  display: flex;
  gap: var(--space-3);
  flex-wrap: wrap;
  color: var(--color-on-surface-variant);
  font-size: var(--text-sm);
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

.meeting-card__agenda {
  color: var(--color-on-surface-variant);
  line-height: 1.5;
  margin: 0;
}

.meeting-card__footer {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.footer-pill {
  display: inline-flex;
  align-items: center;
  min-height: 30px;
  padding: 0 10px;
  border-radius: var(--radius-full);
  background: var(--color-surface-container-low);
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

.meeting-card__chevron {
  display: flex;
  align-items: center;
  padding: 0 var(--space-4);
  color: var(--color-on-surface-variant);
}

@media (max-width: 900px) {
  .meeting-card {
    flex-direction: column;
  }

  .meeting-badge {
    width: 100%;
    flex-direction: row;
    justify-content: flex-start;
  }

  .meeting-card__header {
    flex-direction: column;
  }

  .meeting-card__chevron {
    display: none;
  }
}
</style>
