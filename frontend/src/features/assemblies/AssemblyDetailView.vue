<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAuthStore } from '@/stores/auth'
import type {
  Meeting,
  MeetingAttendance,
  MeetingAttendanceMode,
  MeetingDocument,
  MeetingType,
  Resolution,
  VoteResult,
  VoteType,
} from '@/types/domain'
import { documentsApi, meetingsApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import CIconBell from '@/components/atoms/CIconBell.svg'
import CIconCheck from '@/components/atoms/CIconCheck.svg'
import CIconClock from '@/components/atoms/CIconClock.svg'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconLock from '@/components/atoms/CIconLock.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconUpload from '@/components/atoms/CIconUpload.svg'
import VoteModal from './VoteModal.vue'
import StartMeetingModal from './StartMeetingModal.vue'
import CloseMeetingModal from './CloseMeetingModal.vue'
import ConvocationModal from './ConvocationModal.vue'
import UploadDocumentModal from './UploadDocumentModal.vue'
import DeleteDocumentModal from './DeleteDocumentModal.vue'
import AttendanceModal from './AttendanceModal.vue'
import EditMeetingModal from './EditMeetingModal.vue'
import AddResolutionModal from './AddResolutionModal.vue'
import EditResolutionModal from './EditResolutionModal.vue'
import DeleteResolutionModal from './DeleteResolutionModal.vue'
import Art24Modal from './Art24Modal.vue'
import Art25Modal from './Art25Modal.vue'
import MinutesModal from './MinutesModal.vue'
import SignaturesModal from './SignaturesModal.vue'

const route = useRoute()
const { t } = useI18n()
const { formatDate } = useFormatters()
const auth = useAuthStore()

const buildingName = ref('')
const buildingSharesTotal = ref(0)
const meeting = ref<Meeting | null>(null)
const resolutions = ref<Resolution[]>([])
const voteResults = ref<Record<string, VoteResult>>({})
const documents = ref<MeetingDocument[]>([])
const attendance = ref<MeetingAttendance>({
  entries: [],
  shares_present: 0,
  total_shares: 0,
  present_units: 0,
})

const showEditMeetingModal = ref(false)
const showAddResolutionModal = ref(false)
const showDocumentModal = ref(false)
const showAttendanceModal = ref(false)

const showCloseMeetingModal = ref(false)

const showDeleteDocumentModal = ref(false)
const pendingDeleteDocumentId = ref<string | null>(null)
const pendingDeleteDocumentName = ref<string | null>(null)

const showVoteModal = ref(false)
const votingResolution = ref<Resolution | null>(null)

const showStartMeetingModal = ref(false)

const showConvocationModal = ref(false)

const showMinutesModal = ref(false)
const notifyingMinutes = ref(false)
const minutesFeedback = useFormFeedback()

const showSignaturesModal = ref(false)

const notificationDutyDeadline = computed(() => {
  if (!meeting.value?.meeting_date) return null
  const meetingDate = new Date(meeting.value.meeting_date)
  const deadline = new Date(meetingDate)
  deadline.setDate(deadline.getDate() + 30)
  return deadline
})

const notificationDeadline = computed(() => {
  if (!meeting.value?.minutes_notified_at) return null
  const notified = new Date(meeting.value.minutes_notified_at)
  const deadline = new Date(notified)
  deadline.setMonth(deadline.getMonth() + 2)
  return deadline
})

const notificationDutyVariant = computed<'info' | 'warning' | 'error'>(() => {
  if (!notificationDutyDeadline.value) return 'info'
  const daysLeft = Math.floor((notificationDutyDeadline.value.getTime() - Date.now()) / 86_400_000)
  if (daysLeft < 0) return 'error'
  if (daysLeft <= 7) return 'warning'
  return 'info'
})

const showEditResolutionModal = ref(false)
const resolutionToEdit = ref<Resolution | null>(null)

const showDeleteResolutionModal = ref(false)
const resolutionToDelete = ref<Resolution | null>(null)

const showArt24Modal = ref(false)
const art24SourceResolution = ref<Resolution | null>(null)

const showArt25Modal = ref(false)
const art25SourceResolution = ref<Resolution | null>(null)

const canManageResolutions = computed(() => auth.isManager && meeting.value?.status === 'draft')
const canAddResolution = canManageResolutions
const canManageVotes = computed(() => auth.isManager && ['ongoing'].includes(meeting.value?.status ?? ''))
const canEditResolution = computed(() => auth.isManager && ['draft', 'ongoing'].includes(meeting.value?.status ?? ''))
const canEditAttendance = computed(() => auth.isManager && meeting.value?.status !== 'closed')
const sortedAttendanceEntries = computed(() =>
  [...attendance.value.entries].sort((a, b) => parseInt(a.unit_number, 10) - parseInt(b.unit_number, 10))
)
const attendingUnits = computed(() => sortedAttendanceEntries.value.filter(e => e.mode !== 'absent'))
const noticeDocuments = computed(() => documents.value.filter(document => document.kind === 'notice'))
const attachmentDocuments = computed(() => documents.value.filter(document => document.kind === 'attachment'))
const isMeetingPast = computed(() => {
  if (!meeting.value) return false
  return new Date(meeting.value.meeting_date).getTime() < Date.now()
})
const topbarActionState = computed<'none' | 'draft' | 'called-waiting' | 'called-startable' | 'ongoing' | 'closed'>(() => {
  if (!auth.isManager || !meeting.value) return 'none'

  if (meeting.value.status === 'draft') return 'draft'
  if (meeting.value.status === 'closed') return 'closed'
  if (meeting.value.status === 'ongoing') return 'ongoing'
  if (meeting.value.status === 'called') {
    return isMeetingPast.value ? 'called-startable' : 'called-waiting'
  }

  return 'none'
})
const attendancePct = computed(() => {
  if (!attendance.value.total_shares) return 0
  return Math.round((attendance.value.shares_present / attendance.value.total_shares) * 100)
})
const hasSharesTotalMismatch = computed(() =>
  buildingSharesTotal.value > 0 && attendance.value.total_shares !== buildingSharesTotal.value,
)
const quorumReached = computed(() => attendance.value.present_units >= 1)
const attendanceFillStyle = computed(() => ({ width: `${Math.min(attendancePct.value, 100)}%` }))
const resolvedResolutionCount = computed(() => resolutions.value.filter(resolution => resolution.status !== 'pending').length)
const pendingResolutionCount = computed(() => resolutions.value.filter(resolution => resolution.status === 'pending').length)
const agendaItems = computed(() => {
  if (meeting.value?.agenda?.trim()) {
    return meeting.value.agenda
      .split(/\r?\n/)
      .map(item => item.trim())
      .filter(Boolean)
  }

  return resolutions.value.map(resolution => resolution.title)
})
const noticeDeadline = computed(() => {
  if (!meeting.value) return ''
  const date = new Date(meeting.value.meeting_date)
  date.setDate(date.getDate() - 21)
  return formatDate(date.toISOString())
})
const majorityLegalDescriptions = computed(() => {
  const legalEntries = [
    {
      id: 'simple_majority',
      anchorId: 'art24',
      title: t('meetings.vote.types.simple_majority'),
      summary: t('meetings.legalCard.descriptions.simple_majority'),
      legalLinks: [
        {
          label: t('meetings.legalCard.simple_majority.linkLabel'),
          href: t('meetings.legalCard.simple_majority.linkHref'),
        },
      ],
    },
    {
      id: 'absolute_majority',
      anchorId: 'art25',
      title: t('meetings.vote.types.absolute_majority'),
      summary: t('meetings.legalCard.descriptions.absolute_majority'),
      legalLinks: [
        {
          label: t('meetings.legalCard.absolute_majority.linkLabel'),
          href: t('meetings.legalCard.absolute_majority.linkHref'),
        },
      ],
    },
    {
      id: 'absolute_majority_second_vote',
      anchorId: 'art25-1',
      title: t('meetings.vote.art25_1_title'),
      summary: t('meetings.legalCard.descriptions.absolute_majority_second_vote'),
      legalLinks: [
        {
          label: t('meetings.legalCard.absolute_majority_second_vote.linkLabel'),
          href: t('meetings.legalCard.absolute_majority_second_vote.linkHref'),
        },
      ],
    },
    {
      id: 'double_majority',
      anchorId: 'art26',
      title: t('meetings.vote.types.double_majority'),
      summary: t('meetings.legalCard.descriptions.double_majority'),
      legalLinks: [
        {
          label: t('meetings.legalCard.double_majority.linkLabel'),
          href: t('meetings.legalCard.double_majority.linkHref'),
        },
      ],
    },
    {
      id: 'unanimity',
      anchorId: 'art26-al4',
      title: t('meetings.vote.types.unanimity'),
      summary: t('meetings.legalCard.descriptions.unanimity'),
      legalLinks: [
        {
          label: t('meetings.legalCard.unanimity.linkLabel'),
          href: t('meetings.legalCard.unanimity.linkHref'),
        },
      ],
    },
  ]

  return legalEntries
})

async function loadData() {
  const meetingId = route.params.id as string
  const [building, meetingRes, resolutionRes, documentsRes, attendanceRes] = await Promise.all([
    useBuildingStore().fetch(),
    meetingsApi.get(meetingId),
    meetingsApi.listResolutions(meetingId),
    meetingsApi.listDocuments(meetingId),
    meetingsApi.attendance(meetingId),
  ])

  buildingName.value = building.name
  buildingSharesTotal.value = building.shares_total
  meeting.value = meetingRes.data
  resolutions.value = resolutionRes.data.map(resolution => ({
    ...resolution,
    meeting_id: resolution.meeting_id ?? meetingRes.data.id,
  }))
  documents.value = documentsRes.data
  attendance.value = attendanceRes.data

  const tallies = await Promise.allSettled(
    resolutions.value.map(async (resolution) => {
      const result = await meetingsApi.results(meetingId, resolution.id)
      return [resolution.id, result.data] as const
    }),
  )

  voteResults.value = Object.fromEntries(
    tallies
      .filter((entry): entry is PromiseFulfilledResult<readonly [string, VoteResult]> => entry.status === 'fulfilled')
      .map(entry => entry.value),
  )
}

const { loading, error, reload } = usePageLoader(loadData)



onMounted(reload)

function typeLabel(type: MeetingType): string {
  return t(`meetings.typeLabel.${type}`)
}

function meetingStatusLabel(status: Meeting['status']): string {
  return t(`meetings.statuses.${status}`)
}

type BadgeVariant = 'open' | 'pending' | 'closed' | 'disabled' | 'valid' | 'err' | 'primary' | 'default'
type MeetingBannerState = 'draft' | 'called' | 'ongoing' | 'closed'

function meetingStatusVariant(status: Meeting['status']): BadgeVariant {
  if (status === 'called') return 'primary'
  if (status === 'ongoing' || status === 'closed') return 'valid'
  return 'disabled'
}

function meetingState(status: Meeting['status']): MeetingBannerState {
  if (status === 'closed') return 'closed'
  if (status === 'ongoing') return 'ongoing'
  if (status === 'called') return 'called'
  return 'draft'
}

function meetingBannerTitle(meeting: Meeting): string {
  const state = meetingState(meeting.status)
  if (state === 'called') return t('meetings.bannerCalled')
  if (state === 'ongoing') return t('meetings.bannerOngoing')
  if (state === 'closed') return `${t('meetings.bannerClosed')} · ${meeting.minutes_notified_at ? t('meetings.documents.minutesReady') : t('meetings.documents.minutesPending')}`
  return meetingStatusLabel(state)
}

function meetingBannerSubtitle(status: Meeting['status']): string {
  const state = meetingState(status)

  if (state === 'closed') {
    if (meeting.value?.minutes_notified_at) {
      return `${t('meetings.minutes.notified_at', { date: formatDate(meeting.value.minutes_notified_at) })} · ${t('meetings.minutes.deadline_hint', { date: formatDate(notificationDeadline.value!.toISOString()) })}`
    }
    return t('meetings.minutes.not_yet_notified_at')
  }

  if (state === 'ongoing') {
    return t('meetings.bannerOngoingSub', {
      resolved: resolvedResolutionCount.value,
      pending: pendingResolutionCount.value,
    })
  }

  return `J-21 : ${noticeDeadline.value}`
}

function hasFinalVoteResult(status: Resolution['status']): boolean {
  return status === 'passed' || status === 'rejected'
}

function voteResultSubtitle(resolution: Resolution, result: VoteResult): string {
  if (resolution.vote_type === 'simple_majority') {
    return t(`meetings.vote.explanations.simple_majority.${result.passed ? 'passed' : 'rejected'}`, {
      threshold: result.threshold,
    })
  }

  if (resolution.vote_type === 'absolute_majority') {
    return t(`meetings.vote.explanations.absolute_majority.${result.passed ? 'passed' : 'rejected'}`, {
      threshold: result.threshold,
    })
  }

  if (resolution.vote_type === 'double_majority') {
    return t(`meetings.vote.explanations.double_majority.${result.passed ? 'passed' : 'rejected'}`, {
      threshold: result.threshold,
    })
  }

  return t(`meetings.vote.explanations.unanimity.${result.passed ? 'passed' : 'rejected'}`, {
    threshold: result.threshold,
  })
}

function resolutionStatusLabel(status: Resolution['status']): string {
  return t(`meetings.resolutionStatuses.${status}`)
}

function resolutionStatusVariant(status: Resolution['status']): BadgeVariant {
  if (status === 'passed') return 'valid'
  if (status === 'rejected') return 'err'
  return 'disabled'
}

function voteTypeLabel(voteType: VoteType): string {
  const shortKey = `meetings.vote.typeShort.${voteType}`
  const shortLabel = t(shortKey)
  return shortLabel === shortKey ? t(`meetings.vote.types.${voteType}`) : shortLabel
}

function voteTypeVariant(voteType: VoteType): BadgeVariant {
  if (voteType === 'simple_majority') return 'primary'
  if (voteType === 'absolute_majority' || voteType === 'double_majority') return 'default'
  return 'disabled'
}

function voteTypeClass(voteType: VoteType): string {
  if (voteType === 'absolute_majority') return 'chip-art25'
  if (voteType === 'double_majority') return 'chip-art26'
  return ''
}

function votePct(value: number, total: number): number {
  if (!total) return 0
  return Math.round((value / total) * 100)
}

function showsAbstentionTallyPercent(voteType: VoteType): boolean {
  return voteType !== 'simple_majority'
}

function attendanceModeLabel(mode: MeetingAttendanceMode): string {
  return t(`meetings.attendanceModes.${mode}`)
}

function attendanceModeVariant(mode: MeetingAttendanceMode): BadgeVariant {
  if (mode === 'present') return 'valid'
  if (mode === 'proxy') return 'pending'
  if (mode === 'correspondence') return 'primary'
  return 'disabled'
}

function documentKindLabel(kind: MeetingDocument['kind']): string {
  return t(`meetings.documents.${kind}`)
}

function openEditMeetingModal() {
  showEditMeetingModal.value = true
}

function openResolutionModal() {
  showAddResolutionModal.value = true
}

function openDocumentModal() {
  showDocumentModal.value = true
}

function openAttendanceModal() {
  showAttendanceModal.value = true
}

async function openVoteModal(resolution: Resolution) {
  votingResolution.value = resolution
  showVoteModal.value = true
}

async function handleVoteSaved(payload: { resolutionId: string; result: VoteResult | null; finalized: boolean; eligibleForSecondVote: string | null }) {
  if (payload.result) {
    voteResults.value = { ...voteResults.value, [payload.resolutionId]: payload.result }
  }
  if (payload.finalized && payload.result) {
    resolutions.value = resolutions.value.map(r =>
      r.id === payload.resolutionId
        ? { ...r, status: payload.result!.passed ? 'passed' : 'rejected' }
        : r,
    )
  }
  if (meeting.value) {
    const attendanceRes = await meetingsApi.attendance(meeting.value.id)
    attendance.value = attendanceRes.data
  }
}

function promptDeleteDocument(documentId: string, name: string) {
  pendingDeleteDocumentId.value = documentId
  pendingDeleteDocumentName.value = name
  showDeleteDocumentModal.value = true
}

async function openDocument(documentId: string) {
  const response = await documentsApi.getDownloadUrl(documentId)
  window.open(response.data.url, '_blank', 'noopener')
}

function openStartMeetingModal() {
  showStartMeetingModal.value = true
}

function openMinutesModal() {
  showMinutesModal.value = true
}

function openSignaturesModal() {
  showSignaturesModal.value = true
}

async function markMinutesNotified() {
  if (!meeting.value) return
  notifyingMinutes.value = true
  minutesFeedback.reset()
  try {
    await meetingsApi.notifyMinutes(meeting.value.id)
    minutesFeedback.setSuccess(t('common.saved'))
    await reload()
  } catch (err) {
    minutesFeedback.setError(toErrorMessage(err))
  } finally {
    notifyingMinutes.value = false
  }
}

function openEditResolutionModal(resolution: Resolution) {
  resolutionToEdit.value = resolution
  showEditResolutionModal.value = true
}

function promptDeleteResolution(resolution: Resolution) {
  resolutionToDelete.value = resolution
  showDeleteResolutionModal.value = true
}

const reordering = ref(false)

async function moveResolution(index: number, direction: -1 | 1) {
  const current = meeting.value
  if (!current || reordering.value) return
  const target = index + direction
  if (target < 0 || target >= resolutions.value.length) return
  const ordered = [...resolutions.value]
  const moved = ordered.splice(index, 1)[0]
  if (!moved) return
  ordered.splice(target, 0, moved)
  reordering.value = true
  try {
    await meetingsApi.reorderResolutions(current.id, ordered.map(resolution => resolution.id))
    await reload()
  } finally {
    reordering.value = false
  }
}

function openArt24Modal(resolution: Resolution) {
  art24SourceResolution.value = resolution
  showArt24Modal.value = true
}

function openArt25Modal(resolution: Resolution) {
  art25SourceResolution.value = resolution
  showArt25Modal.value = true
}

</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          <span>{{ buildingName || '...' }}</span>
          <CIconChevronRight class="icon-xs" />
          <RouterLink
            :to="{ name: 'assembly-list' }"
            class="topbar-breadcrumb-link"
          >
            {{ t('meetings.title') }}
          </RouterLink>
          <CIconChevronRight class="icon-xs" />
          <span>{{ meeting ? `${typeLabel(meeting.meeting_type)} ${new Date(meeting.meeting_date).getFullYear()}` : '…' }}</span>
        </div>
      </template>
      <template #right>
        <template v-if="topbarActionState === 'draft'">
          <CBtn
            variant="secondary"
            size="sm"
            :icon="CIconEdit"
            :title="t('common.edit')"
            shrink="sm"
            @click="openEditMeetingModal"
          />
          <CBtn
            variant="primary"
            size="sm"
            :icon="CIconBell"
            :title="t('meetings.call')"
            @click="showConvocationModal = true"
          />
        </template>

        <CBtn
          v-else-if="topbarActionState === 'called-waiting'"
          variant="secondary"
          size="sm"
          :icon="CIconClock"
          :title="t('meetings.waiting')"
          disabled
        />

        <CBtn
          v-else-if="topbarActionState === 'called-startable'"
          variant="primary"
          size="sm"
          :icon="CIconCheck"
          :title="t('meetings.start')"
          @click="openStartMeetingModal"
        />

        <CBtn
          v-else-if="topbarActionState === 'ongoing'"
          variant="primary"
          size="sm"
          :icon="CIconLock"
          :title="t('meetings.close')"
          @click="showCloseMeetingModal = true"
        />

        <CBtn
          v-else-if="topbarActionState === 'closed'"
          variant="secondary"
          size="sm"
          :icon="CIconLock"
          :title="t('meetings.closedAction')"
          disabled
        />
      </template>
    </CTopbar>

    <div class="view-container">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else-if="meeting">
        <header class="view-header">
          <CEyebrow>
            <CBadge
              :variant="meetingStatusVariant(meeting.status)"
              dot
              :pulse="meeting.status === 'called'"
            >
              {{ meetingStatusLabel(meeting.status) }}
            </CBadge>
            <span>{{ t('page.eyebrows.assemblyDetail') }}</span>
          </CEyebrow>

          <CHeading type="header">
            {{ typeLabel(meeting.meeting_type) }} {{ new Date(meeting.meeting_date).getFullYear() }}
          </CHeading>

          <CMetaLine>
            <CMetaItem variant="sub">
              <template #icon>
                <CIconCalendar class="icon-xs" />
              </template>
              {{ formatDate(meeting.meeting_date) }}
            </CMetaItem>
            <CSeparator />
            <CMetaItem
              v-if="meeting.location"
              variant="sub"
            >
              <template #icon>
                <CIconLocation class="icon-xs" />
              </template>
              {{ meeting.location }}
            </CMetaItem>
            <CSeparator v-if="meeting.location" />
            <CMetaItem variant="sub">
              <template #icon>
                <CIconBell class="icon-xs" />
              </template>
              {{ t('meetings.calledAt', { date: meeting.notice_date ? formatDate(meeting.notice_date) : '—' }) }}
            </CMetaItem>
          </CMetaLine>
        </header>

        <div :class="['status-banner', `status-banner--${meetingState(meeting.status)}`]">
          <div class="status-banner__left">
            <div class="status-banner__icon">
              <CIconBell v-if="meetingState(meeting.status) === 'called'" />
              <CIconCheck v-else-if="meetingState(meeting.status) === 'ongoing'" />
              <CIconLock v-else-if="meetingState(meeting.status) === 'closed'" />
              <CIconEdit v-else />
            </div>
            <div>
              <p class="text-sm font-semibold">
                {{ meetingBannerTitle(meeting) }}
              </p>
              <p class="text-xs text-on-surface-variant">
                {{ meetingBannerSubtitle(meeting.status) }}
              </p>
            </div>
          </div>
          <CBtn
            v-if="meetingState(meeting.status) === 'closed' && meeting.minutes_document_id"
            variant="ghost"
            size="sm"
            type="button"
            :icon="CIconDownload"
            :title="t('meetings.minutes.download_signed')"
            @click="openDocument(meeting.minutes_document_id!)"
          />
        </div>

        <CCard
          v-if="meetingState(meeting.status) === 'closed' && auth.isManager"
          class="minutes-card"
        >
          <div class="flex flex-col gap-3">
            <span class="label-sm">{{ t('meetings.minutes.title') }}</span>
            <template v-if="meeting.minutes_document_id">
              <div class="flex items-center gap-2 flex-wrap">
                <CIconLock class="text-on-surface-variant icon-sm" />
                <span class="text-sm">{{ t('meetings.minutes.signed_attached') }}</span>
                <CBtn
                  variant="ghost"
                  size="sm"
                  type="button"
                  :icon="CIconDownload"
                  :title="t('meetings.minutes.download_signed')"
                  @click="openDocument(meeting.minutes_document_id!)"
                />
                <CBtn
                  v-if="auth.isManager"
                  variant="ghost"
                  size="sm"
                  type="button"
                  :icon="CIconEdit"
                  :title="t('meetings.signatures.action_label')"
                  @click="openSignaturesModal"
                />
              </div>
              <div class="flex flex-col gap-1 text-xs text-on-surface-variant">
                <span>
                  {{ t('meetings.signatures.president') }} :
                  <template v-if="meeting.minutes_president_signed_at">
                    {{ t('meetings.signatures.signed_on', { date: formatDate(meeting.minutes_president_signed_at) }) }}
                  </template>
                  <span v-else>{{ t('meetings.signatures.not_signed') }}</span>
                </span>
                <span>
                  {{ t('meetings.signatures.secretary') }} :
                  <template v-if="meeting.minutes_secretary_signed_at">
                    {{ t('meetings.signatures.signed_on', { date: formatDate(meeting.minutes_secretary_signed_at) }) }}
                  </template>
                  <span v-else>{{ t('meetings.signatures.not_signed') }}</span>
                </span>
                <span v-if="(meeting.minutes_scrutineers_signed ?? []).length > 0">
                  {{ t('meetings.signatures.scrutineers') }} :
                  {{ (meeting.minutes_scrutineers_signed ?? []).map(s => s.name).join(', ') }}
                </span>
              </div>
              <template v-if="auth.isManager">
                <CAlert
                  v-if="notificationDutyDeadline"
                  :variant="notificationDutyVariant"
                  class="w-full"
                >
                  {{ t('meetings.minutes.notification_duty_hint', { date: formatDate(notificationDutyDeadline.toISOString()) }) }}<sup><a
                    href="https://www.legifrance.gouv.fr/loda/article_lc/LEGIARTI000039329680"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="legal-ref"
                  >art.&nbsp;42&nbsp;al.&nbsp;3</a></sup>
                </CAlert>
                <div class="flex items-center gap-2 flex-wrap">
                  <CBtn
                    variant="ghost"
                    size="sm"
                    type="button"
                    :icon="CIconBell"
                    :title="t('meetings.minutes.mark_notified')"
                    :loading="notifyingMinutes"
                    @click="markMinutesNotified"
                  />
                </div>
              </template>
            </template>
            <template v-else>
              <p class="text-xs text-on-surface-variant">
                {{ t('meetings.minutes.lead') }}
              </p>
              <div
                v-if="auth.isManager"
                class="flex gap-2 flex-wrap"
              >
                <CBtn
                  variant="primary"
                  size="sm"
                  type="button"
                  :icon="CIconUpload"
                  :title="t('meetings.minutes.upload_signed')"
                  @click="openMinutesModal"
                />
              </div>
            </template>
            <CFormFeedback
              v-if="minutesFeedback.status.value !== 'idle'"
              :status="minutesFeedback.status.value"
              :message="minutesFeedback.message.value"
            />
          </div>
        </CCard>

        <div class="detail-layout">
          <section class="main-column">
            <CSectionHeader>
              <CHeading>
                {{ t('meetings.resolutions') }}
              </CHeading>
              <p class="section-subtitle">
                {{ t('meetings.resolutionCount', resolutions.length) }}
              </p>
              <template
                v-if="canAddResolution"
                #actions
              >
                <CBtn
                  variant="ghost"
                  size="sm"
                  :icon="CIconPlus"
                  :title="t('meetings.addResolution')"
                  @click="openResolutionModal"
                />
              </template>
            </CSectionHeader>

            <div
              v-if="resolutions.length === 0"
              class="py-8 text-center text-md text-on-surface-variant"
            >
              {{ t('meetings.resolutionsEmpty') }}
            </div>

            <div
              v-else
              class="resolution-list"
            >
              <CCard
                v-for="(resolution, index) in resolutions"
                :key="resolution.id"
                class="resolution-card "
              >
                <div class="resolution-card__header">
                  <div
                    :class="['resolution-number',
                             resolution.status === 'passed' ? 'tone--accent' 
                             : voteResults[resolution.id]?.eligible_for_second_vote != null ? 'tone--warn' 
                               : resolution.status === 'rejected' ? 'tone--disabled' 
                                 : '',
                    ]"
                  >
                    {{ resolution.number }}
                  </div>
                  <div class="resolution-card__body">
                    <h3 class="font-serif text-md font-semibold">
                      {{ resolution.title }}
                    </h3>
                    <div class="resolution-chips">
                      <CBadge
                        :variant="voteTypeVariant(resolution.vote_type)"
                        :class="voteTypeClass(resolution.vote_type)"
                      >
                        {{ voteTypeLabel(resolution.vote_type) }}
                      </CBadge>
                      <CBadge
                        :variant="resolutionStatusVariant(resolution.status)"
                        dot
                      >
                        {{ resolutionStatusLabel(resolution.status) }}
                      </CBadge>
                    </div>
                  </div>
                </div>

                <p
                  v-if="resolution.description"
                  class="resolution-description"
                >
                  {{ resolution.description }}
                </p>

                <div
                  v-if="voteResults[resolution.id]"
                  class="tally-box"
                >
                  <p class="tally-title">
                    {{ t('meetings.vote.tally') }}
                  </p>
                  <p
                    v-if="resolution.status === 'pending'"
                    class="text-2xs text-on-surface-variant"
                  >
                    {{ t('meetings.vote.inProgress') }}
                  </p>
                  <div class="tally-bars">
                    <div class="tally-row">
                      <span class="tally-label">{{ t('meetings.vote.in_favor') }}</span>
                      <div class="tally-track">
                        <div
                          class="tally-fill tally-fill--for"
                          :style="{ width: `${votePct(voteResults[resolution.id].for_votes, voteResults[resolution.id].shares_counted)}%` }"
                        ></div>
                      </div>
                      <span class="tally-pct">{{ votePct(voteResults[resolution.id].for_votes, voteResults[resolution.id].shares_counted) }}%</span>
                      <span class="tally-shares">{{ t('units.sharesAmount', { amount: voteResults[resolution.id].for_votes }) }}</span>
                    </div>
                    <div class="tally-row">
                      <span class="tally-label">{{ t('meetings.vote.against') }}</span>
                      <div class="tally-track">
                        <div
                          class="tally-fill tally-fill--against"
                          :style="{ width: `${votePct(voteResults[resolution.id].against_votes, voteResults[resolution.id].shares_counted)}%` }"
                        ></div>
                      </div>
                      <span class="tally-pct">{{ votePct(voteResults[resolution.id].against_votes, voteResults[resolution.id].shares_counted) }}%</span>
                      <span class="tally-shares">{{ t('units.sharesAmount', { amount: voteResults[resolution.id].against_votes }) }}</span>
                    </div>
                    <div class="tally-row">
                      <span class="tally-label">{{ t('meetings.vote.abstain') }}</span>
                      <div class="tally-track">
                        <div
                          class="tally-fill tally-fill--abstain"
                          :style="{ width: `${showsAbstentionTallyPercent(resolution.vote_type) ? votePct(voteResults[resolution.id].abstentions, voteResults[resolution.id].shares_counted) : 0}%` }"
                        ></div>
                      </div>
                      <span
                        class="tally-pct"
                      >{{ showsAbstentionTallyPercent(resolution.vote_type) ? votePct(voteResults[resolution.id].abstentions, voteResults[resolution.id].shares_counted) + '%' : '-' }}</span>
                      <span class="tally-shares">{{ t('units.sharesAmount', { amount: voteResults[resolution.id].abstentions }) }}</span>
                    </div>
                  </div>
                  <CAlert
                    v-if="hasFinalVoteResult(resolution.status)"
                    :variant="voteResults[resolution.id].passed ? 'success' : 'error'"
                    :title="voteResults[resolution.id].passed ? t('meetings.vote.passed') : t('meetings.vote.rejected')"
                  >
                    <p class="text-2xs">
                      {{ voteResultSubtitle(resolution, voteResults[resolution.id]) }}
                    </p>
                    <template #side>
                      <span class="font-mono font-medium self-center">{{ voteResults[resolution.id].for_votes }} / {{ voteResults[resolution.id].shares_counted }}</span>
                    </template>
                  </CAlert>
                  <CAlert
                    v-if="voteResults[resolution.id]?.eligible_for_second_vote === 'art_25_1'"
                    variant="warning"
                    :title="t('meetings.vote.art25_1_title')"
                  >
                    <p class="text-2xs">
                      {{ t('meetings.vote.explanations.absolute_majority_second_vote', { threshold: Math.ceil(voteResults[resolution.id].shares_counted / 3) }) }}
                    </p>
                  </CAlert>
                  <CAlert
                    v-if="voteResults[resolution.id]?.eligible_for_second_vote === 'art_26_1'"
                    variant="warning"
                    :title="t('meetings.vote.art26_1_title')"
                  >
                    <p class="text-2xs">
                      {{ t('meetings.vote.explanations.double_majority_second_vote', { threshold: Math.ceil(voteResults[resolution.id].shares_counted / 2) }) }}
                    </p>
                  </CAlert>
                </div>

                <div
                  v-else
                  class="pending-box"
                >
                  {{ t('meetings.vote.pendingVote') }}
                </div>

                <div
                  v-if="canManageResolutions || canEditResolution || canManageVotes"
                  class="resolution-card__actions"
                >
                  <CBtn
                    v-if="canManageResolutions"
                    size="sm"
                    variant="ghost"
                    :disabled="index === 0 || reordering"
                    :title="t('meetings.moveResolutionUp')"
                    @click="moveResolution(index, -1)"
                  />
                  <CBtn
                    v-if="canManageResolutions"
                    size="sm"
                    variant="ghost"
                    :disabled="index === resolutions.length - 1 || reordering"
                    :title="t('meetings.moveResolutionDown')"
                    @click="moveResolution(index, 1)"
                  />
                  <CBtn
                    v-if="canEditResolution"
                    size="sm"
                    variant="ghost"
                    :title="t('meetings.editResolution')"
                    @click="openEditResolutionModal(resolution)"
                  />
                  <CBtn
                    v-if="canManageResolutions"
                    size="sm"
                    variant="ghost"
                    :title="t('meetings.deleteResolution')"
                    @click="promptDeleteResolution(resolution)"
                  />
                  <CBtn
                    v-if="voteResults[resolution.id]?.eligible_for_second_vote === 'art_25_1' && meeting?.status === 'ongoing'"
                    size="sm"
                    variant="ghost"
                    :title="t('meetings.createArt24')"
                    @click="openArt24Modal(resolution)"
                  />
                  <CBtn
                    v-if="voteResults[resolution.id]?.eligible_for_second_vote === 'art_26_1' && meeting?.status === 'ongoing'"
                    size="sm"
                    variant="ghost"
                    :title="t('meetings.createArt25')"
                    @click="openArt25Modal(resolution)"
                  />
                  <CBtn
                    v-if="canManageVotes"
                    size="sm"
                    variant="ghost"
                    :title="t('meetings.vote.enterVotes')"
                    @click="openVoteModal(resolution)"
                  />
                </div>
              </CCard>
            </div>
          </section>

          <aside class="side-column">
            <CCard
              v-if="meeting && ['ongoing', 'closed'].includes(meeting.status)"
              class="session-roles-card"
            >
              <CHeading type="card">
                {{ t('meetings.session_roles.title') }}
              </CHeading>
              <CFlexGroup>
                <div>
                  <p class="text-xs text-on-surface-variant">
                    {{ t('meetings.session_roles.president') }}
                  </p>
                  <p class="text-sm font-medium text-on-surface">
                    {{ meeting.president_name }}
                  </p>
                </div>
                <div>
                  <p class="text-xs text-on-surface-variant">
                    {{ t('meetings.session_roles.secretary') }}
                  </p>
                  <p class="text-sm font-medium text-on-surface">
                    {{ meeting.secretary_name }}
                  </p>
                </div>
                <div>
                  <p class="text-xs text-on-surface-variant">
                    {{ t('meetings.session_roles.scrutineers') }}
                  </p>
                  <p class="text-sm font-medium text-on-surface">
                    <template v-if="meeting.scrutineers.length > 0">
                      {{ meeting.scrutineers.map(s => s.name).join(', ') }}
                    </template>
                    <span v-else>—</span>
                  </p>
                </div>
              </CFlexGroup>
            </CCard>

            <CCard>
              <CHeading type="card">
                {{ t('meetings.agenda.title') }}
              </CHeading>
              <template v-if="agendaItems.length === 0">
                <CEmptyState
                  size="compact"
                  :title="t('meetings.agenda.empty')"
                  :description="t('meetings.agenda.emptyHint')"
                />
              </template>
              <CFlexGroup
                tag="ol"
                class="agenda-list"
                direction="column"
              >
                <li
                  v-for="(item, index) in agendaItems"
                  :key="`${index}-${item}`"
                  class="agenda-item"
                >
                  <span class="agenda-item__n">{{ index + 1 }}.</span>
                  <span class="agenda-item__text">{{ item }}</span>
                </li>
              </CFlexGroup>
            </CCard>

            <CCard>
              <CSectionHeader>
                <CHeading type="card">
                  {{ t('meetings.documents.title') }}
                </CHeading>
                <p class="side-subtitle">
                  {{ t('meetings.callReady') }}
                </p>
                <template
                  v-if="auth.isManager"
                  #actions
                >
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :icon="CIconPlus"
                    :title="t('meetings.documents.uploadAction')"
                    @click="openDocumentModal"
                  />
                </template>
              </CSectionHeader>

              <div class="doc-group">
                <p class="doc-group__title">
                  {{ t('meetings.documents.notice') }}
                </p>
                <template v-if="noticeDocuments.length === 0">
                  <CEmptyState
                    size="compact"
                    :title="t('meetings.documents.empty')"
                  />
                </template>
                <div
                  v-else
                  class="doc-list"
                >
                  <div
                    v-for="document in noticeDocuments"
                    :key="document.id"
                    class="doc-row"
                  >
                    <button
                      class="doc-link"
                      @click="openDocument(document.id)"
                    >
                      {{ document.original_name }}
                    </button>
                    <div class="doc-row__actions">
                      <span class="doc-kind">{{ documentKindLabel(document.kind) }}</span>
                      <button
                        v-if="auth.isManager"
                        class="doc-remove"
                        @click="promptDeleteDocument(document.id, document.original_name)"
                      >
                        {{ t('common.delete') }}
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <div class="doc-group">
                <p class="doc-group__title">
                  {{ t('meetings.documents.attachment') }}
                </p>
                <template v-if="attachmentDocuments.length === 0">
                  <CEmptyState
                    size="compact"
                    :title="t('meetings.documents.empty')"
                  />
                </template>
                <div
                  v-else
                  class="doc-list"
                >
                  <div
                    v-for="document in attachmentDocuments"
                    :key="document.id"
                    class="doc-row"
                  >
                    <button
                      class="doc-link"
                      @click="openDocument(document.id)"
                    >
                      {{ document.original_name }}
                    </button>
                    <div class="doc-row__actions">
                      <span class="doc-kind">{{ documentKindLabel(document.kind) }}</span>
                      <button
                        v-if="auth.isManager"
                        class="doc-remove"
                        @click="promptDeleteDocument(document.id, document.original_name)"
                      >
                        {{ t('common.delete') }}
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <CAlert
                v-if="meetingState(meeting.status) === 'closed'"
                :variant="meeting.minutes_document_id ? 'success' : 'warning'"
                :title="meeting.minutes_document_id ? t('meetings.documents.minutesReady') : t('meetings.documents.minutesPending')"
              >
                <CBtn
                  v-if="meeting.minutes_document_id"
                  variant="ghost"
                  size="sm"
                  type="button"
                  class="self-center"
                  :icon="CIconDownload"
                  :title="t('meetings.minutes.download_signed')"
                  @click="openDocument(meeting.minutes_document_id!)"
                />
              </CAlert>
            </CCard>

            <CCard class="">
              <CSectionHeader>
                <CHeading type="card">
                  {{ t('meetings.presenceQuorum') }}
                </CHeading>
                <p class="side-subtitle">
                  {{ t('meetings.sharesRepresented') }}
                </p>
                <template
                  v-if="canEditAttendance"
                  #actions
                >
                  <CBtn
                    variant="ghost"
                    size="sm"
                    :icon="CIconEdit"
                    :title="t('meetings.attendanceAction')"
                    @click="openAttendanceModal"
                  />
                </template>
              </CSectionHeader>

              <div class="flex flex-col gap-2">
                <p class="font-serif font-bold tracking-tight">
                  <span class="text-on-surface text-[40px]">{{ attendance.shares_present }}</span> 
                  <span class="text-xl">/ {{ attendance.total_shares }}</span>
                </p>
                <CAlert
                  v-if="hasSharesTotalMismatch"
                  variant="error"
                  class="quorum-alert"
                  :title="t('meetings.sharesTotalMismatchTitle')"
                >
                  {{ t('meetings.sharesTotalMismatchDetail', { expected: buildingSharesTotal, actual: attendance.total_shares }) }}
                </CAlert>
                <p class="presence-kpi__detail">
                  {{ t('meetings.attendanceSummary', { count: attendance.present_units }) }}
                </p>
                <div class="presence-track">
                  <div
                    class="presence-fill"
                    :style="attendanceFillStyle"
                  ></div>
                </div>
                <div class="presence-legend">
                  <span>0</span>
                  <span>{{ attendancePct }}%</span>
                  <span>{{ attendance.total_shares }}</span>
                </div>
                <CAlert
                  :variant="quorumReached ? 'success' : 'warning'"
                  class="quorum-alert"
                  :title="quorumReached ? t('meetings.quorumOk') : t('meetings.quorumKo')"
                >
                  {{ quorumReached ? t('meetings.quorumOkDetail') : t('meetings.quorumKoDetail', { threshold: 1 }) }}
                </CAlert>
              </div>
              <CFlexGroup direction="column">
                <div
                  v-for="entry in sortedAttendanceEntries"
                  :key="entry.unit_id"
                  class="participant-row"
                >
                  <div class="participant-row__identity">
                    <CAvatar
                      :name="entry.owner_name"
                      size="sm"
                    />
                    <div>
                      <p class="participant-name">
                        {{ entry.owner_name || '—' }}
                      </p>
                      <p class="participant-meta">
                        Lot {{ entry.unit_number }}
                      </p>
                    </div>
                  </div>
                  <div class="participant-row__meta">
                    <small class="attendance-shares">{{ t('units.sharesAmount', { amount: entry.shares_represented }) }}</small>
                    <CBadge :variant="attendanceModeVariant(entry.mode)">
                      {{ attendanceModeLabel(entry.mode) }}
                    </CBadge>
                  </div>
                </div>
              </CFlexGroup>
            </CCard>
          </aside>
        </div>

        <CLegalCard>
          <CFlexGroup direction="column">
            <CLegalNotice
              v-for="entry in majorityLegalDescriptions"
              :id="entry.anchorId"
              :key="entry.title"
              :links="entry.legalLinks"
            >
              <CLegalDescription
                :title="entry.title"
                :summary="entry.summary"
              />
            </CLegalNotice>
          </CFlexGroup>
        </CLegalCard>
      </template>
    </div>

    <EditMeetingModal
      v-if="meeting"
      v-model:open="showEditMeetingModal"
      :meeting="meeting"
      @updated="reload"
    />

    <AddResolutionModal
      v-if="meeting"
      v-model:open="showAddResolutionModal"
      :meeting-id="meeting.id"
      :next-number="resolutions.length + 1"
      @created="reload"
    />

    <VoteModal
      :open="showVoteModal"
      :meeting-id="meeting?.id ?? ''"
      :resolution="votingResolution"
      :attending-units="attendingUnits"
      @update:open="showVoteModal = $event"
      @saved="handleVoteSaved"
    />

    <EditResolutionModal
      v-if="meeting"
      v-model:open="showEditResolutionModal"
      :meeting-id="meeting.id"
      :meeting-status="meeting.status"
      :resolution="resolutionToEdit"
      @updated="reload"
    />

    <DeleteResolutionModal
      v-if="meeting"
      :open="showDeleteResolutionModal"
      :meeting-id="meeting.id"
      :resolution="resolutionToDelete"
      @update:open="showDeleteResolutionModal = $event"
      @deleted="reload"
    />

    <UploadDocumentModal
      :open="showDocumentModal"
      :meeting-id="meeting?.id ?? ''"
      @update:open="showDocumentModal = $event"
      @created="reload"
    />

    <AttendanceModal
      :open="showAttendanceModal"
      :meeting-id="meeting?.id ?? ''"
      :entries="attendance.entries"
      @update:open="showAttendanceModal = $event"
      @updated="reload"
    />

    <Art24Modal
      v-model:open="showArt24Modal"
      :meeting-id="meeting?.id ?? ''"
      :resolution="art24SourceResolution"
      :next-number="resolutions.length + 1"
      @created="reload"
    />

    <Art25Modal
      v-model:open="showArt25Modal"
      :meeting-id="meeting?.id ?? ''"
      :resolution="art25SourceResolution"
      :next-number="resolutions.length + 1"
      @created="reload"
    />

    <SignaturesModal
      v-model:open="showSignaturesModal"
      :meeting-id="meeting?.id ?? ''"
      :meeting="meeting!"
      @saved="reload"
    />

    <CloseMeetingModal
      :open="showCloseMeetingModal"
      :meeting-id="meeting?.id ?? ''"
      @update:open="showCloseMeetingModal = $event"
      @closed="reload"
    />

    <DeleteDocumentModal
      :open="showDeleteDocumentModal"
      :meeting-id="meeting?.id ?? ''"
      :document-id="pendingDeleteDocumentId"
      :document-name="pendingDeleteDocumentName"
      @update:open="showDeleteDocumentModal = $event"
      @deleted="reload"
    />

    <ConvocationModal
      :open="showConvocationModal"
      :meeting-id="meeting?.id ?? ''"
      @update:open="showConvocationModal = $event"
      @called="reload"
    />

    <MinutesModal
      v-model:open="showMinutesModal"
      :meeting-id="meeting?.id ?? ''"
      @saved="reload"
    />

    <StartMeetingModal
      :open="showStartMeetingModal"
      :meeting-id="meeting?.id ?? ''"
      @update:open="showStartMeetingModal = $event"
      @started="reload"
    />
  </div>
</template>

<style scoped>
.status-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-4);
  border-radius: var(--radius-2xl);
}

.status-banner__left {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.status-banner__icon {
  width: var(--space-10);
  height: var(--space-10);
  border-radius: var(--radius-xl);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.status-banner__icon :deep(svg) {
  width: var(--space-5);
  height: var(--space-5);
  stroke: currentColor;
  fill: none;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.status-banner--called {
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
}

.status-banner--called .status-banner__icon {
  background: color-mix(in srgb, var(--color-primary) var(--color-tint), transparent);
  color: var(--color-primary);
}

.status-banner--closed {
  background: var(--color-success-container);
  color: var(--color-on-success-container);
}

.status-banner--closed .status-banner__icon {
  background: color-mix(in srgb, var(--color-success) var(--color-tint), transparent);
  color: var(--color-success);
}

.status-banner--ongoing {
  background: var(--color-secondary-container);
  color: var(--color-on-primary-container);
}

.status-banner--ongoing .status-banner__icon {
  background: color-mix(in srgb, var(--color-primary) var(--color-tint), transparent);
  color: var(--color-primary);
}

.status-banner--draft {
  background: var(--color-surface-container);
  color: var(--color-on-surface);
}

.status-banner--draft .status-banner__icon {
  background: color-mix(in srgb, var(--color-outline) 24%, transparent);
  color: var(--color-on-surface-variant);
}

.detail-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 320px;
  gap: var(--space-4);
  align-items: start;
}

.main-column,
.side-column {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.section-subtitle,
.side-subtitle {
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

.resolution-list,
.doc-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.resolution-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.resolution-card__header {
  display: flex;
  gap: var(--space-3);
}

.resolution-number {
  width: var(--space-10);
  height: var(--space-10);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-xl);
  background: var(--tone-bg, var(--color-secondary-container));
  color: var(--tone-fg, var(--color-on-secondary-container));
  font-family: var(--font-display);
  font-weight: var(--fw-bold);
}

.resolution-card__body {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  min-width: 0;
}

.resolution-chips {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.resolution-description {
  color: var(--color-on-surface-variant);
  line-height: 1.5;
  margin: 0;
}

.tally-box {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.tally-title {
  margin: 0;
  font-size: var(--text-2xs);
  font-weight: var(--fw-semibold);
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.tally-bars {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.tally-row {
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr) 44px 52px;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-sm);
}

.tally-label {
  font-size: var(--text-xs);
  font-weight: var(--fw-medium);
}

.tally-track {
  height: var(--space-2);
  border-radius: var(--radius-full);
  overflow: hidden;
  background: var(--color-surface-container-high);
}

.tally-fill {
  height: 100%;
  border-radius: var(--radius-full);
}

.tally-fill--for {
  background: var(--gradient-success);
}

.tally-fill--against {
  background: var(--gradient-error);
}

.tally-fill--abstain {
  background: var(--color-outline);
}

.tally-pct {
  text-align: right;
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.tally-shares {
  text-align: right;
  font-family: var(--font-mono);
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

.pending-box {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding: var(--space-3);
  border-radius: var(--radius-xl);
}

.pending-box {
  background: var(--color-surface-container-low);
  color: var(--color-on-surface-variant);
}

.resolution-card__actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
}

.doc-kind {
  color: var(--color-on-surface-variant);
}

.doc-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.doc-group__title {
  margin: 0;
  font-size: var(--text-xs);
  font-weight: var(--fw-semibold);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.doc-row {
  display: flex;
  justify-content: space-between;
  gap: var(--space-3);
  align-items: center;
  padding: var(--space-3);
  border-radius: var(--radius-xl);
  background: var(--color-surface-container-low);
}

.doc-row__actions,
.participant-row__meta {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-2xs);
}

.doc-link,
.doc-remove {
  border: none;
  background: transparent;
  font: inherit;
}

.doc-link {
  color: var(--color-primary);
  cursor: pointer;
  text-align: left;
}

.doc-remove {
  cursor: pointer;
  color: var(--color-error);
}

.presence-kpi__detail {
  margin: 0;
}

.presence-kpi__detail {
  font-size: var(--text-xs);
  color: var(--color-on-secondary-container);
  opacity: 0.84;
}

.presence-track {
  height: 10px;
  border-radius: var(--radius-full);
  background: color-mix(in srgb, var(--color-outline) 45%, transparent);
  overflow: hidden;
}

.presence-fill {
  height: 100%;
  border-radius: var(--radius-full);
  background: var(--gradient-primary);
}

.presence-legend {
  display: flex;
  justify-content: space-between;
  gap: var(--space-2);
  font-size: var(--text-2xs);
  color: var(--color-on-secondary-container);
  opacity: 0.86;
}

.quorum-alert {
  border-radius: var(--radius-lg);
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-xs);
  font-weight: var(--fw-medium);
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.participant-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
}

.participant-row__identity {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
}

.participant-name,
.participant-meta {
  margin: 0;
}

.participant-name {
  font-size: var(--text-sm);
  color: var(--color-on-surface);
}

.participant-meta {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
}

.attendance-shares {
  font-family: var(--font-mono);
  color: var(--color-on-surface-variant);
}

.agenda-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.agenda-item {
  display: flex;
  gap: var(--space-2);
}

.agenda-item__n {
  width: var(--space-5);
  flex-shrink: 0;
  padding-top: 1px;
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: var(--fw-medium);
  color: var(--color-primary);
}

.agenda-item__text {
  font-size: var(--text-sm);
  color: var(--color-on-surface);
  line-height: 1.4;
}

.chip-art25 {
  --tone-bg: #e8def8;
  --tone-fg: #21005d;
}

.chip-art26 {
  --tone-bg: #fce8d5;
  --tone-fg: #7a3600;
}

@media (max-width: 960px) {
  .detail-layout {
    grid-template-columns: 1fr;
  }

  .tally-row {
    grid-template-columns: 72px minmax(0, 1fr) 44px;
  }

  .tally-shares {
    display: none;
  }
}

</style>
