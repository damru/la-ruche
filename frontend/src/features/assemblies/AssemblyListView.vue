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
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CSeparator from '@/components/atoms/CSeparator.vue'
import CCard from '@/components/molecules/CCard.vue'
import CMetaLine from '@/components/molecules/CMetaLine.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CTabs, { type Tab } from '@/components/atoms/CTabs.vue'
import CHeading from '@/components/atoms/CHeading.vue'

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

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

const meetings = computed(() => {
  if (filter.value === 'upcoming') return allMeetings.value.filter(meeting => meeting.status !== 'closed')
  if (filter.value === 'closed') return allMeetings.value.filter(meeting => meeting.status === 'closed')
  return allMeetings.value
})

const upcomingCount = computed(() => allMeetings.value.filter(meeting => meeting.status !== 'closed').length)
const closedCount = computed(() => allMeetings.value.filter(meeting => meeting.status === 'closed').length)

const meetingTabs = computed<Tab[]>(() => [
  { value: 'all', label: t('common.allFem'), count: allMeetings.value.length },
  { value: 'upcoming', label: t('meetings.filterUpcoming'), count: upcomingCount.value },
  { value: 'closed', label: t('meetings.filterClosed'), count: closedCount.value },
])

function typeLabel(type: MeetingType): string {
  return t(`meetings.typeLabel.${type}`)
}

function statusLabel(status: Meeting['status']): string {
  return t(`meetings.statuses.${status}`)
}

function statusBadgeVariant(status: Meeting['status']): 'primary' | 'valid' | 'disabled' {
  if (status === 'called') return 'primary'
  if (status === 'closed' || status === 'ongoing') return 'valid'
  return 'disabled'
}

function meetingBadgeLabel(type: MeetingType): string {
  return type === 'annual' ? t('meetings.typeLabel.annualShort') : t('meetings.typeLabel.extraordinaryShort')
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
    createFeedback.setError(toErrorMessage(err))
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
          <CIconChevronRight />
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

    <div class="flex flex-col p-10 max-w-[1200px] gap-6 max-sm:p-5 assembly-list-page">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else>
        <header class="flex flex-col items-start gap-2">
          <CEyebrow>
            {{ t('page.eyebrows.assemblies') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('meetings.title') }}
          </CHeading>
          <CMetaLine>
            <CMetaItem variant="sub">
              {{ buildingName }}
            </CMetaItem>
            <CSeparator />
            <CMetaItem variant="sub">
              {{ t('meetings.count', allMeetings.length) }}
            </CMetaItem>
          </CMetaLine>
        </header>

        <CTabs variant="filter" v-model="filter" :tabs="meetingTabs" />

        <div
          v-if="meetings.length === 0"
          class="py-8 text-center text-md text-on-surface-variant"
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
            class="meeting-card"
            role="button"
            tabindex="0"
            @click="openMeeting(meeting.id)"
            @keydown.enter.prevent="openMeeting(meeting.id)"
            @keydown.space.prevent="openMeeting(meeting.id)"
          >
            <div :class="['meeting-badge', { 'meeting-badge--closed': meeting.status === 'closed', 'meeting-badge--ongoing': meeting.status === 'ongoing', 'meeting-badge--draft': statusBadgeVariant(meeting.status) === 'disabled', 'meeting-badge--pulse': meeting.status === 'ongoing' }]">
              <span class="meeting-badge__type">{{ meetingBadgeLabel(meeting.meeting_type) }}</span>
              <span class="meeting-badge__year">{{ new Date(meeting.meeting_date).getFullYear() }}</span>
            </div>

            <div class="meeting-card__main">
              <div class="flex flex-col gap-x-4 gap-y-1 mb-4">
                <div class="flex flex-row justify-between">
                  <CHeading type="card">{{ typeLabel(meeting.meeting_type) }} {{ new Date(meeting.meeting_date).getFullYear() }}</CHeading>
                  <CBadge
                    :variant="statusBadgeVariant(meeting.status)"
                    :pulse="meeting.status === 'called' || meeting.status === 'ongoing'"
                    class="text-xs"
                    dot
                  >
                    {{ statusLabel(meeting.status) }}
                  </CBadge>
                </div>
                <CMetaLine>
                  <CMetaItem variant="sub">
                    <template #icon>
                      <CIconCalendar />
                    </template>
                    {{ formatDate(meeting.meeting_date) }}
                  </CMetaItem>
                  <CSeparator />
                  <CMetaItem
                    v-if="meeting.location"
                    variant="sub"
                  >
                    <template #icon>
                      <CIconLocation />
                    </template>
                    {{ meeting.location }}
                  </CMetaItem>
                </CMetaLine>
              </div>

              <div class="meeting-card__resolution-row body-xs">
                <span class="meeting-card__res-count">
                  <span class="font-serif font-semibold text-xl">{{ meeting.resolution_count }}</span>
                  {{ t('meetings.statResolutions', meeting.resolution_count) }}
                </span>
                <span
                  v-if="meeting.status === 'closed'"
                  class="meeting-card__res-passed tone--valid"
                >
                    <span class="font-serif font-semibold text-xl">{{ meeting.passed_count }}</span>
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
        class="flex flex-col gap-4"
        @submit.prevent="saveMeeting"
      >
        <CFormSelect
          v-model="createForm.meeting_type"
          :label="t('meetings.infoFields.type')"
          :options="[
            { value: 'annual', label: t('meetings.types.annual') },
            { value: 'extraordinary', label: t('meetings.types.extraordinary') },
          ]"
        />
        <CFormInput
          v-model="createForm.meeting_date"
          :label="t('meetings.meetingDate')"
          type="datetime-local"
        />
        <CFormInput
          v-model="createForm.location"
          :label="t('meetings.location')"
        />
        <CFormTextarea
          v-model="createForm.agenda"
          :label="t('meetings.agenda')"
        />
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
  overflow: clip;
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
  width: fit-content;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-4);
  background: var(--gradient-primary);
  color: white;
}

.meeting-badge--closed,
.meeting-badge--ongoing {
  background: linear-gradient(135deg, #1b5e20, #43a047);
}

.meeting-badge--pulse {
  position: relative;
}

.meeting-badge--pulse::after {
  content: '';
  position: absolute;
  top: 8px;
  right: 8px;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: white;
  animation: pulse-dot 1.8s ease-in-out infinite;
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.4; transform: scale(0.7); }
}

.meeting-badge--draft {
  background:
    repeating-linear-gradient(
      -45deg,
      rgba(255, 255, 255, 0.18) 0px,
      rgba(255, 255, 255, 0.18) 10px,
      transparent 10px,
      transparent 20px
    ),
    var(--gradient-primary);
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

@media (max-width: 900px) {
  .meeting-badge {
    width: 72px;
  }

}

</style>
