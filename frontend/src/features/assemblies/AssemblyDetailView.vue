<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useAuthStore } from '@/stores/auth'
import type {
  Meeting,
  MeetingAttendance,
  MeetingAttendanceEntry,
  MeetingAttendanceMode,
  MeetingDocument,
  MeetingType,
  Resolution,
  VoteResult,
  VoteType,
} from '@/types/domain'
import { buildingApi, documentsApi, meetingsApi } from '@/api'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const { formatDate } = useFormatters()
const auth = useAuthStore()

const buildingName = ref('')
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

const meetingFeedback = useFormFeedback()
const resolutionFeedback = useFormFeedback()
const documentFeedback = useFormFeedback()
const attendanceFeedback = useFormFeedback()

const savingMeeting = ref(false)
const savingResolution = ref(false)
const savingDocument = ref(false)
const savingAttendance = ref(false)
const callingMeeting = ref(false)
const closingMeeting = ref(false)

const showVoteModal = ref(false)
const votingResolution = ref<Resolution | null>(null)
const unitVotes = ref<Record<string, string>>({})
const savingVotes = ref(false)
const voteFeedback = useFormFeedback()

const { schedule: scheduleMeetingClose, forceExpire: forceMeetingClose } = useAutoClose()
const { schedule: scheduleResolutionClose, forceExpire: forceResolutionClose } = useAutoClose()
const { schedule: scheduleDocumentClose, forceExpire: forceDocumentClose } = useAutoClose()
const { schedule: scheduleAttendanceClose, forceExpire: forceAttendanceClose } = useAutoClose()
const { schedule: scheduleVoteClose, forceExpire: forceVoteClose } = useAutoClose()

watch(showEditMeetingModal, (open) => { if (!open) forceMeetingClose() })
watch(showAddResolutionModal, (open) => { if (!open) forceResolutionClose() })
watch(showDocumentModal, (open) => { if (!open) forceDocumentClose() })
watch(showAttendanceModal, (open) => { if (!open) forceAttendanceClose() })
watch(showVoteModal, (open) => { if (!open) forceVoteClose() })

function defaultMeetingForm() {
  return {
    meeting_type: 'annual' as MeetingType,
    meeting_date: '',
    location: '',
  }
}

function defaultResolutionForm() {
  return {
    number: 1,
    title: '',
    description: '',
    vote_type: 'simple_majority' as VoteType,
  }
}

function defaultDocumentForm() {
  return {
    kind: 'notice' as 'notice' | 'attachment',
    file: null as File | null,
  }
}

const meetingForm = ref(defaultMeetingForm())
const resolutionForm = ref(defaultResolutionForm())
const documentForm = ref(defaultDocumentForm())
const attendanceDraft = ref<MeetingAttendanceEntry[]>([])
const meetingAgendaItems = ref<string[]>([''])

const canAddResolution = computed(() => auth.isManager && ['draft', 'called'].includes(meeting.value?.status ?? ''))
const canManageVotes = computed(() => auth.isManager && meeting.value?.status === 'held')
const sortedAttendanceEntries = computed(() =>
  [...attendance.value.entries].sort((a, b) => parseInt(a.unit_number, 10) - parseInt(b.unit_number, 10))
)
const attendingUnits = computed(() => sortedAttendanceEntries.value.filter(e => e.mode !== 'absent'))
const votePositionOptions = computed(() => [
  { value: '', label: t('meetings.vote.notSet') },
  { value: 'in_favor', label: t('meetings.vote.in_favor') },
  { value: 'against', label: t('meetings.vote.against') },
  { value: 'abstain', label: t('meetings.vote.abstain') },
])
const noticeDocuments = computed(() => documents.value.filter(document => document.kind === 'notice'))
const attachmentDocuments = computed(() => documents.value.filter(document => document.kind === 'attachment'))
const isMeetingPast = computed(() => {
  if (!meeting.value) return false
  return new Date(meeting.value.meeting_date).getTime() < Date.now()
})
const topbarActionState = computed<'none' | 'draft' | 'called-waiting' | 'called-closable' | 'closed'>(() => {
  if (!auth.isManager || !meeting.value) return 'none'

  if (meeting.value.status === 'draft') return 'draft'
  if (meeting.value.status === 'closed') return 'closed'
  if (meeting.value.status === 'held') return 'called-closable'
  if (meeting.value.status === 'called') {
    return isMeetingPast.value ? 'called-closable' : 'called-waiting'
  }

  return 'none'
})
const attendancePct = computed(() => {
  if (!attendance.value.total_shares) return 0
  return Math.round((attendance.value.shares_present / attendance.value.total_shares) * 100)
})
const quorumThreshold = computed(() => Math.ceil((attendance.value.total_shares || 0) / 2))
const quorumReached = computed(() => attendance.value.shares_present >= quorumThreshold.value)
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
  const [buildingRes, meetingRes, resolutionRes, documentsRes, attendanceRes] = await Promise.all([
    buildingApi.get(),
    meetingsApi.get(meetingId),
    meetingsApi.listResolutions(meetingId),
    meetingsApi.listDocuments(meetingId),
    meetingsApi.attendance(meetingId),
  ])

  buildingName.value = buildingRes.data.name
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

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))

const meetingTypeOptions = computed(() => [
  { value: 'annual', label: t('meetings.types.annual') },
  { value: 'extraordinary', label: t('meetings.types.extraordinary') },
])

const voteTypeOptions = computed(() => [
  { value: 'simple_majority', label: t('meetings.vote.types.simple_majority') },
  { value: 'absolute_majority', label: t('meetings.vote.types.absolute_majority') },
  { value: 'double_majority', label: t('meetings.vote.types.double_majority') },
  { value: 'unanimity', label: t('meetings.vote.types.unanimity') },
])

const documentKindOptions = computed(() => [
  { value: 'notice', label: t('meetings.documents.notice') },
  { value: 'attachment', label: t('meetings.documents.attachment') },
])

const attendanceModeOptions = computed(() => [
  { value: 'present', label: t('meetings.attendanceModes.present') },
  { value: 'proxy', label: t('meetings.attendanceModes.proxy') },
  { value: 'correspondence', label: t('meetings.attendanceModes.correspondence') },
  { value: 'absent', label: t('meetings.attendanceModes.absent') },
])

function setResolutionNumber(v: string) {
  resolutionForm.value.number = parseInt(v, 10) || 1
}
onMounted(reload)

function typeLabel(type: MeetingType): string {
  return t(`meetings.typeLabel.${type}`)
}

function meetingStatusLabel(status: Meeting['status']): string {
  return t(`meetings.statuses.${status}`)
}

type BadgeVariant = 'open' | 'pending' | 'closed' | 'disabled' | 'valid' | 'err' | 'primary' | 'default'
type MeetingBannerState = 'draft' | 'called' | 'held' | 'closed'

function meetingStatusVariant(status: Meeting['status']): BadgeVariant {
  if (status === 'called') return 'primary'
  if (status === 'closed') return 'valid'
  return 'disabled'
}

function meetingBannerState(status: Meeting['status']): MeetingBannerState {
  if (status === 'closed') return 'closed'
  if (status === 'held' || topbarActionState.value === 'called-closable') return 'held'
  if (status === 'called') return 'called'
  return 'draft'
}

function meetingBannerTitle(status: Meeting['status']): string {
  const state = meetingBannerState(status)
  if (state === 'called') return t('meetings.bannerCalled')
  if (state === 'held') return t('meetings.bannerHeld')
  if (state === 'closed') return t('meetings.bannerClosed')
  return `${meetingStatusLabel(status)} · ${t('meetings.legalWarning')}`
}

function meetingBannerSubtitle(status: Meeting['status']): string {
  const state = meetingBannerState(status)

  if (state === 'closed') {
    return meeting.value?.minutes_document_id
      ? t('meetings.documents.minutesReady')
      : t('meetings.documents.minutesPending')
  }

  if (state === 'held') {
    return t('meetings.bannerHeldSub', {
      resolved: resolvedResolutionCount.value,
      pending: pendingResolutionCount.value,
      minutes: meeting.value?.minutes_document_id
        ? t('meetings.documents.minutesReady')
        : t('meetings.documents.minutesPending'),
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

function ownerInitials(name?: string | null): string {
  if (!name?.trim()) return '??'
  const parts = name.trim().split(/\s+/).filter(Boolean)
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase()
  return `${parts[0][0] ?? ''}${parts[parts.length - 1][0] ?? ''}`.toUpperCase()
}

function documentKindLabel(kind: MeetingDocument['kind']): string {
  return t(`meetings.documents.${kind}`)
}

function openEditMeetingModal() {
  if (!meeting.value) return

  const agenda = (meeting.value.agenda ?? '')
    .split(/\r?\n/)
    .map(item => item.trim())
    .filter(Boolean)

  meetingForm.value = {
    meeting_type: meeting.value.meeting_type,
    meeting_date: toDateTimeLocal(meeting.value.meeting_date),
    location: meeting.value.location ?? '',
  }
  meetingAgendaItems.value = agenda.length ? agenda : ['']
  meetingFeedback.reset()
  showEditMeetingModal.value = true
}

function addAgendaItem() {
  meetingAgendaItems.value.push('')
}

function removeAgendaItem(index: number) {
  if (meetingAgendaItems.value.length <= 1) {
    meetingAgendaItems.value = ['']
    return
  }
  meetingAgendaItems.value.splice(index, 1)
}

function openResolutionModal() {
  resolutionForm.value = {
    ...defaultResolutionForm(),
    number: resolutions.value.length + 1,
  }
  resolutionFeedback.reset()
  showAddResolutionModal.value = true
}

function openDocumentModal() {
  documentForm.value = defaultDocumentForm()
  documentFeedback.reset()
  showDocumentModal.value = true
}

function openAttendanceModal() {
  attendanceDraft.value = attendance.value.entries.map(entry => ({ ...entry }))
  attendanceFeedback.reset()
  showAttendanceModal.value = true
}

async function saveMeeting() {
  if (!meeting.value || !meetingForm.value.meeting_date) return

  savingMeeting.value = true
  meetingFeedback.reset()

  const normalizedAgenda = meetingAgendaItems.value
    .map(item => item.trim())
    .filter(Boolean)

  try {
    await meetingsApi.update(meeting.value.id, {
      meeting_type: meetingForm.value.meeting_type,
      meeting_date: new Date(meetingForm.value.meeting_date).toISOString(),
      location: meetingForm.value.location || undefined,
      agenda: normalizedAgenda.length ? normalizedAgenda.join('\n') : undefined,
    })

    meetingFeedback.setSuccess(t('common.saved'))
    scheduleMeetingClose(async () => {
      showEditMeetingModal.value = false
      await reload()
    })
  } catch (err) {
    meetingFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    savingMeeting.value = false
  }
}

async function saveResolution() {
  if (!meeting.value || !resolutionForm.value.title.trim()) return

  savingResolution.value = true
  resolutionFeedback.reset()

  try {
    await meetingsApi.createResolution(meeting.value.id, {
      number: resolutionForm.value.number,
      title: resolutionForm.value.title,
      description: resolutionForm.value.description || undefined,
      vote_type: resolutionForm.value.vote_type,
    })

    resolutionFeedback.setSuccess(t('common.saved'))
    scheduleResolutionClose(async () => {
      showAddResolutionModal.value = false
      await reload()
    })
  } catch (err) {
    resolutionFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    savingResolution.value = false
  }
}

async function openVoteModal(resolution: Resolution) {
  votingResolution.value = resolution
  unitVotes.value = {}
  voteFeedback.reset()
  showVoteModal.value = true
  try {
    const res = await meetingsApi.listVotes(meeting.value!.id, resolution.id)
    for (const v of res.data) {
      unitVotes.value[v.unit_id] = v.position
    }
  } catch {
    // pre-fill failure non-blocking — manager can still enter votes
  }
}

function fillAllVotes(position: 'in_favor' | 'against') {
  for (const entry of attendingUnits.value) {
    unitVotes.value[entry.unit_id] = position
  }
}

async function saveVotes() {
  if (!votingResolution.value || !meeting.value) return
  savingVotes.value = true
  voteFeedback.reset()
  try {
    await Promise.all(
      attendingUnits.value
        .filter(e => unitVotes.value[e.unit_id])
        .map(e => meetingsApi.vote(meeting.value!.id, votingResolution.value!.id, {
          unit_id: e.unit_id,
          position: unitVotes.value[e.unit_id],
        })),
    )
    const result = await meetingsApi.results(meeting.value.id, votingResolution.value.id)
    voteResults.value = { ...voteResults.value, [votingResolution.value.id]: result.data }
    voteFeedback.setSuccess(t('meetings.vote.voteSaved'))
    scheduleVoteClose(async () => { showVoteModal.value = false })
  } catch (err) {
    voteFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    savingVotes.value = false
  }
}

async function saveDocument() {
  if (!meeting.value || !documentForm.value.file) return

  savingDocument.value = true
  documentFeedback.reset()

  try {
    const file = documentForm.value.file
    const extension = file.name.includes('.') ? file.name.split('.').pop() ?? 'pdf' : 'pdf'
    const upload = await documentsApi.getUploadUrl(extension, file.type || 'application/pdf', file.size)
    await documentsApi.uploadToS3(file, upload.data.url, file.type || 'application/pdf')

    const confirmed = await documentsApi.confirmUpload({
      key: upload.data.key,
      bucket: upload.data.bucket,
      original_name: file.name,
      mime_type: file.type || 'application/pdf',
      size_bytes: file.size,
      visible_roles: ['manager', 'board_member', 'co_owner'],
    })

    await meetingsApi.attachDocument(meeting.value.id, {
      document_id: confirmed.data.id,
      kind: documentForm.value.kind,
    })

    documentFeedback.setSuccess(t('common.saved'))
    scheduleDocumentClose(async () => {
      showDocumentModal.value = false
      await reload()
    })
  } catch (err) {
    documentFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    savingDocument.value = false
  }
}

async function detachDocument(documentId: string) {
  if (!meeting.value) return

  try {
    await meetingsApi.detachDocument(meeting.value.id, documentId)
    await reload()
  } catch (err) {
    documentFeedback.setError(toErrorMessage(err, t('common.error')))
  }
}

async function openDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch (err) {
    documentFeedback.setError(toErrorMessage(err, t('common.error')))
  }
}

async function saveAttendance() {
  if (!meeting.value) return

  savingAttendance.value = true
  attendanceFeedback.reset()

  try {
    await meetingsApi.saveAttendance(meeting.value.id, attendanceDraft.value)
    attendanceFeedback.setSuccess(t('common.saved'))
    scheduleAttendanceClose(async () => {
      showAttendanceModal.value = false
      await reload()
    })
  } catch (err) {
    attendanceFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    savingAttendance.value = false
  }
}

async function callMeeting() {
  if (!meeting.value) return

  callingMeeting.value = true
  documentFeedback.reset()

  try {
    await meetingsApi.call(meeting.value.id)
    await reload()
  } catch (err) {
    documentFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    callingMeeting.value = false
  }
}

async function closeMeeting() {
  if (!meeting.value) return

  closingMeeting.value = true
  documentFeedback.reset()

  try {
    await meetingsApi.close(meeting.value.id)
    await reload()
  } catch (err) {
    documentFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    closingMeeting.value = false
  }
}

function toDateTimeLocal(value: string): string {
  const date = new Date(value)
  const offsetMs = date.getTimezoneOffset() * 60_000
  return new Date(date.getTime() - offsetMs).toISOString().slice(0, 16)
}

function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  documentForm.value.file = input.files?.[0] ?? null
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          <span
            class="breadcrumb-link"
            @click="router.push('/meetings')"
          >{{ buildingName || '...' }}</span>
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span
            class="breadcrumb-link"
            @click="router.push('/meetings')"
          >{{ t('meetings.title') }}</span>
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ meeting ? `${typeLabel(meeting.meeting_type)} ${new Date(meeting.meeting_date).getFullYear()}` : '…' }}</span>
        </div>
      </template>
      <template #right>
        <template v-if="topbarActionState === 'draft'">
          <CBtn
            variant="secondary"
            size="sm"
            @click="openEditMeetingModal"
          >
            <CIconEdit />
            <span class="btn-label">{{ t('common.edit') }}</span>
          </CBtn>
          <CBtn
            variant="primary"
            size="sm"
            :loading="callingMeeting"
            @click="callMeeting"
          >
            <CIconBell />
            <span class="btn-label">{{ t('meetings.call') }}</span>
          </CBtn>
        </template>

        <CBtn
          v-else-if="topbarActionState === 'called-waiting'"
          variant="secondary"
          size="sm"
          disabled
        >
          <CIconClock />
          <span class="btn-label">{{ t('meetings.waiting') }}</span>
        </CBtn>

        <CBtn
          v-else-if="topbarActionState === 'called-closable'"
          variant="primary"
          size="sm"
          :loading="closingMeeting"
          @click="closeMeeting"
        >
          <CIconCheck />
          <span class="btn-label">{{ t('meetings.close') }}</span>
        </CBtn>

        <CBtn
          v-else-if="topbarActionState === 'closed'"
          variant="secondary"
          size="sm"
          disabled
        >
          <CIconLock />
          <span class="btn-label">{{ t('meetings.closedAction') }}</span>
        </CBtn>
      </template>
    </CTopbar>

    <div class="view-container assembly-detail-page">
      <CPageError
        v-if="error"
        :message="error"
        @retry="reload"
      />
      <CPageLoader v-else-if="loading" />
      <template v-else-if="meeting">
        <header class="assembly-detail-header">
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

          <h1 class="display-md">
            {{ typeLabel(meeting.meeting_type) }} {{ new Date(meeting.meeting_date).getFullYear() }}
          </h1>

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
            <CSeparator v-if="meeting.location" />
            <CMetaItem variant="sub">
              <template #icon>
                <CIconBell />
              </template>
              {{ t('meetings.calledAt', { date: meeting.notice_date ? formatDate(meeting.notice_date) : '—' }) }}
            </CMetaItem>
          </CMetaLine>
        </header>

        <div :class="['status-banner', `status-banner--${meetingBannerState(meeting.status)}`]">
          <div class="status-banner__left">
            <div class="status-banner__icon">
              <CIconBell v-if="meetingBannerState(meeting.status) === 'called'" />
              <CIconCheck v-else-if="meetingBannerState(meeting.status) === 'held'" />
              <CIconLock v-else-if="meetingBannerState(meeting.status) === 'closed'" />
              <CIconEdit v-else />
            </div>
            <div>
              <p class="status-banner__title">
                {{ meetingBannerTitle(meeting.status) }}
              </p>
              <p class="status-banner__sub">
                {{ meetingBannerSubtitle(meeting.status) }}
              </p>
            </div>
          </div>
        </div>

        <div class="detail-layout">
          <section class="main-column">
            <div class="flex flex-row justify-between items-start">
              <div>
                <h2 class="headline-sm">
                  {{ t('meetings.resolutions') }}
                </h2>
                <p class="section-subtitle">
                  {{ t('meetings.resolutionCount', resolutions.length) }}
                </p>
              </div>
              <CBtn
                v-if="canAddResolution"
                variant="ghost"
                size="sm"
                @click="openResolutionModal"
              >
                <CIconPlus />
                <span class="btn-label">{{ t('meetings.addResolution') }}</span>
              </CBtn>
            </div>

            <div
              v-if="resolutions.length === 0"
              class="empty-state"
            >
              {{ t('meetings.resolutionsEmpty') }}
            </div>

            <div
              v-else
              class="resolution-list"
            >
              <CCard
                v-for="resolution in resolutions"
                :key="resolution.id"
                class="resolution-card gap-y-4"
              >
                <div class="resolution-card__header">
                  <div :class="['resolution-number',
                    resolution.status === 'passed' ? 'tone--accent' 
                      : voteResults[resolution.id]?.eligible_for_second_vote === true ? 'tone--warn' 
                      : resolution.status === 'rejected' ? 'tone--disabled' 
                      : '',
                  ]">
                    {{ resolution.number }}
                  </div>
                  <div class="resolution-card__body">
                    <h3 class="resolution-title">
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
                    <p class="font-(length:--text-2xs)">
                      {{ voteResultSubtitle(resolution, voteResults[resolution.id]) }}
                    </p>
                    <template #side>
                      <span class="font-mono font-medium self-center">{{ voteResults[resolution.id].for_votes }} / {{ voteResults[resolution.id].shares_counted }}</span>
                    </template>
                  </CAlert>
                  <CAlert
                    v-if="voteResults[resolution.id]?.eligible_for_second_vote"
                    variant="warning"
                    :title="t('meetings.vote.art25_1_title')"
                  >
                    <p class="font-(length:--text-2xs)">
                      {{ t('meetings.vote.explanations.absolute_majority_second_vote', { threshold: Math.ceil(voteResults[resolution.id].shares_counted / 3) }) }}
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
                  v-if="canManageVotes"
                  class="vote-entry-action"
                >
                  <CBtn
                    size="sm"
                    variant="ghost"
                    @click="openVoteModal(resolution)"
                  >
                    {{ t('meetings.vote.enterVotes') }}
                  </CBtn>
                </div>
              </CCard>
            </div>
          </section>

          <aside class="side-column">
            <CCard class="gap-y-4">
              <h2 class="side-head headline-sm card-title">
                {{ t('meetings.info') }}
              </h2>
              <CFlexGroup tag="dl" class="[&_dt]:text-xs [&_dd]:font-medium [&_dd]:text(--color-on-surface) " variant="vertical">
                <div class="info-list__row">
                  <dt>{{ t('meetings.infoFields.type') }}</dt>
                  <dd>{{ typeLabel(meeting.meeting_type) }}</dd>
                </div>
                <div class="info-list__row">
                  <dt>{{ t('meetings.infoFields.date') }}</dt>
                  <dd>{{ formatDate(meeting.meeting_date) }}</dd>
                </div>
                <div class="info-list__row">
                  <dt>{{ t('meetings.infoFields.location') }}</dt>
                  <dd>{{ meeting.location || '—' }}</dd>
                </div>
                <div class="info-list__row">
                  <dt>{{ t('meetings.infoFields.notice') }}</dt>
                  <dd>{{ meeting.notice_date ? formatDate(meeting.notice_date) : '—' }}</dd>
                </div>
                <div class="info-list__row">
                  <dt>{{ t('meetings.infoFields.resolutions') }}</dt>
                  <dd>{{ resolutions.length }}</dd>
                </div>
              </CFlexGroup>
            </CCard>

            <CCard class="gap-y-4">
              <div class="side-head">
                <div>
                  <h2 class="headline-sm card-title">
                    {{ t('meetings.documents.title') }}
                  </h2>
                  <p class="side-subtitle">
                    {{ t('meetings.callReady') }}
                  </p>
                </div>
                <CBtn
                  v-if="auth.isManager"
                  variant="ghost"
                  size="sm"
                  @click="openDocumentModal"
                >
                  <CIconPlus />
                  <span class="btn-label">{{ t('meetings.documents.uploadAction') }}</span>
                </CBtn>
              </div>

              <CFormFeedback
                :status="documentFeedback.status.value"
                :message="documentFeedback.message.value"
              />

              <div class="doc-group">
                <p class="doc-group__title">
                  {{ t('meetings.documents.notice') }}
                </p>
                <div
                  v-if="noticeDocuments.length === 0"
                  class="empty-state empty-state--compact"
                >
                  {{ t('meetings.documents.empty') }}
                </div>
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
                        @click="detachDocument(document.id)"
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
                <div
                  v-if="attachmentDocuments.length === 0"
                  class="empty-state empty-state--compact"
                >
                  {{ t('meetings.documents.empty') }}
                </div>
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
                        @click="detachDocument(document.id)"
                      >
                        {{ t('common.delete') }}
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <div :class="['minutes-pill', meeting.minutes_document_id ? 'tone--valid' : 'tone--disabled']">
                {{ meeting.minutes_document_id ? t('meetings.documents.minutesReady') : t('meetings.documents.minutesPending') }}
              </div>
            </CCard>

            <CCard class="gap-y-4">
              <div class="side-head">
                <div>
                  <h2 class="headline-sm card-title">
                    {{ t('meetings.presenceQuorum') }}
                  </h2>
                  <p class="side-subtitle">
                    {{ t('meetings.sharesRepresented') }}
                  </p>
                </div>
                <CBtn
                  v-if="auth.isManager"
                  variant="ghost"
                  size="sm"
                  @click="openAttendanceModal"
                >
                  <CIconEdit />
                  <span class="btn-label">{{ t('meetings.attendanceAction') }}</span>
                </CBtn>
              </div>

              <div class="flex flex-col gap-2">
                <p class="font-serif font-bold tracking-tight">
                  <span class="text-on-surface text-[40px]">{{ attendance.shares_present }}</span> 
                  <span class="text-[20px] ">/ {{ attendance.total_shares }}</span>
                </p>
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
                  {{ quorumReached ? t('meetings.quorumOkDetail') : t('meetings.quorumKoDetail', { threshold: quorumThreshold }) }}
                </CAlert>
              </div>
              <CFlexGroup variant="vertical">
                <div
                  v-for="entry in sortedAttendanceEntries"
                  :key="entry.unit_id"
                  class="participant-row"
                >
                    <div class="participant-row__identity">
                      <div class="attendance-avatar">
                        {{ ownerInitials(entry.owner_name) }}
                      </div>
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

            <CCard class="gap-y-4">
              <h2 class="side-head headline-sm card-title">
                {{ t('meetings.agenda') }}
              </h2>
              <CFlexGroup tag="ol" class="agenda-list" variant="vertical">
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
          </aside>
        </div>

        <CLegalCard>
          <CFlexGroup variant="vertical">
            <CLegalNotice
              v-for="entry in majorityLegalDescriptions"
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

    <CModal
      :open="showEditMeetingModal"
      :title="t('common.edit')"
      @update:open="showEditMeetingModal = $event"
    >
      <form
        class="form-grid"
        @submit.prevent="saveMeeting"
      >
        <CFormSelect
          v-model="meetingForm.meeting_type"
          :label="t('meetings.infoFields.type')"
          :options="meetingTypeOptions"
        />
        <CFormInput
          v-model="meetingForm.meeting_date"
          :label="t('meetings.meetingDate')"
          type="datetime-local"
        />
        <CFormInput
          v-model="meetingForm.location"
          :label="t('meetings.location')"
        />
        <div class="form-field form-field--full">
          <span class="form-label">{{ t('meetings.agenda') }}</span>
          <div class="agenda-editor">
            <div
              v-for="(_, index) in meetingAgendaItems"
              :key="`agenda-${index}`"
              class="agenda-editor__row"
            >
              <CFormInput
                v-model="meetingAgendaItems[index]"
                :placeholder="`${t('meetings.agenda')} ${index + 1}`"
              />
              <CBtn
                variant="ghost"
                size="sm"
                @click="removeAgendaItem(index)"
              >
                {{ t('common.delete') }}
              </CBtn>
            </div>

            <CBtn
              variant="secondary"
              size="sm"
              @click="addAgendaItem"
            >
              <CIconPlus />
              <span class="btn-label">{{ t('common.add') }}</span>
            </CBtn>
          </div>
        </div>
      </form>

      <template #footer>
        <CFormFeedback
          :status="meetingFeedback.status.value"
          :message="meetingFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showEditMeetingModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="savingMeeting"
          @click="saveMeeting"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>

    <CModal
      :open="showAddResolutionModal"
      :title="t('meetings.newResolution')"
      @update:open="showAddResolutionModal = $event"
    >
      <form
        class="form-grid"
        @submit.prevent="saveResolution"
      >
        <CFormInput
          :model-value="resolutionForm.number"
          label="#"
          type="number"
          @update:model-value="setResolutionNumber"
        />
        <CFormInput
          v-model="resolutionForm.title"
          :label="t('meetings.resolutionLabel')"
          :placeholder="t('meetings.resolutionLabelPlaceholder')"
        />
        <CFormTextarea
          v-model="resolutionForm.description"
          :label="t('meetings.resolutionDescription')"
          :placeholder="t('meetings.resolutionDescriptionPlaceholder')"
        />
        <CFormSelect
          v-model="resolutionForm.vote_type"
          :label="t('meetings.voteRule')"
          :options="voteTypeOptions"
        />
      </form>

      <template #footer>
        <CFormFeedback
          :status="resolutionFeedback.status.value"
          :message="resolutionFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showAddResolutionModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="savingResolution"
          @click="saveResolution"
        >
          {{ t('common.add') }}
        </CBtn>
      </template>
    </CModal>

    <CModal
      :open="showVoteModal"
      size="lg"
      :title="votingResolution ? t('meetings.vote.enterVotesTitle', { title: votingResolution.title }) : ''"
      @update:open="showVoteModal = $event"
    >
      <p
        v-if="attendingUnits.length === 0"
        class="body-sm tone--muted"
      >
        {{ t('meetings.vote.noAttendees') }}
      </p>
      <template v-else>
        <div class="vote-quick-actions">
          <CBtn
            size="sm"
            variant="ghost"
            @click="fillAllVotes('in_favor')"
          >
            {{ t('meetings.vote.fillAllFor') }}
          </CBtn>
          <CBtn
            size="sm"
            variant="ghost"
            @click="fillAllVotes('against')"
          >
            {{ t('meetings.vote.fillAllAgainst') }}
          </CBtn>
        </div>
        <table class="vote-table">
          <thead>
            <tr>
              <th>{{ t('units.number') }}</th>
              <th>{{ t('units.owner') }}</th>
              <th>{{ t('meetings.participants') }}</th>
              <th>{{ t('meetings.vote.myVote') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="entry in attendingUnits"
              :key="entry.unit_id"
            >
              <td class="font-mono vote-table__unit">
                {{ entry.unit_number }}
              </td>
              <td class="vote-table__owner">
                {{ entry.owner_name || '—' }}
              </td>
              <td>
                <CBadge :variant="attendanceModeVariant(entry.mode)">
                  {{ attendanceModeLabel(entry.mode) }}
                </CBadge>
              </td>
              <td :class="['vote-table__select', !unitVotes[entry.unit_id] && 'vote-table__select--empty']">
                <CFormSelect
                  :model-value="unitVotes[entry.unit_id] ?? ''"
                  :options="votePositionOptions"
                  @update:model-value="unitVotes[entry.unit_id] = $event"
                />
              </td>
            </tr>
          </tbody>
        </table>
      </template>
      <template #footer>
        <CFormFeedback
          :status="voteFeedback.status.value"
          :message="voteFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showVoteModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="savingVotes"
          :disabled="attendingUnits.length === 0"
          @click="saveVotes"
        >
          {{ t('meetings.vote.submit') }}
        </CBtn>
      </template>
    </CModal>

    <CModal
      :open="showDocumentModal"
      :title="t('meetings.documents.uploadTitle')"
      @update:open="showDocumentModal = $event"
    >
      <form
        class="form-grid"
        @submit.prevent="saveDocument"
      >
        <CFormSelect
          v-model="documentForm.kind"
          :label="t('meetings.documents.uploadType')"
          :options="documentKindOptions"
        />
        <label class="form-field">
          <span class="form-label">{{ t('meetings.documents.uploadFile') }}</span>
          <input
            class="form-input"
            type="file"
            accept=".pdf,.doc,.docx,.png,.jpg,.jpeg"
            @change="onFileChange"
          />
        </label>
      </form>

      <template #footer>
        <CFormFeedback
          :status="documentFeedback.status.value"
          :message="documentFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showDocumentModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="savingDocument"
          @click="saveDocument"
        >
          {{ t('common.add') }}
        </CBtn>
      </template>
    </CModal>

    <CModal
      :open="showAttendanceModal"
      :title="t('meetings.attendanceTitle')"
      @update:open="showAttendanceModal = $event"
    >
      <form
        class="form-grid"
        @submit.prevent="saveAttendance"
      >
        <p class="modal-subtitle">
          {{ t('meetings.attendanceHint') }}
        </p>
        <div class="attendance-edit-list">
          <div
            v-for="entry in attendanceDraft"
            :key="entry.unit_id"
            class="attendance-edit-row"
          >
            <div class="attendance-edit-row__identity">
              <strong>{{ entry.unit_number }}</strong>
              <span>{{ entry.owner_name || '—' }}</span>
            </div>
            <CFormSelect
              v-model="entry.mode"
              :options="attendanceModeOptions"
            />
            <CFormInput
              v-if="entry.mode === 'proxy'"
              v-model="entry.representative_name"
              :placeholder="t('meetings.attendanceRepresentative')"
            />
          </div>
        </div>
      </form>

      <template #footer>
        <CFormFeedback
          :status="attendanceFeedback.status.value"
          :message="attendanceFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="showAttendanceModal = false"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="savingAttendance"
          @click="saveAttendance"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>
  </div>
</template>

<style scoped>
.assembly-detail-page {
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
}

.assembly-detail-header {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.breadcrumb-link {
  cursor: pointer;
  color: var(--color-on-surface-variant);
}

.status-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-4);
  border-radius: var(--radius-lg);
}

.status-banner__left {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.status-banner__icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.status-banner__icon :deep(svg) {
  width: 20px;
  height: 20px;
  stroke: currentColor;
  fill: none;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.status-banner__title {
  margin: 0;
  font-size: var(--text-sm);
  font-weight: var(--fw-semibold);
}

.status-banner__sub {
  margin: 2px 0 0;
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

.status-banner--called {
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
}

.status-banner--called .status-banner__icon {
  background: color-mix(in srgb, var(--color-primary) 16%, transparent);
  color: var(--color-primary);
}

.status-banner--closed {
  background: color-mix(in srgb, var(--color-success-container) 100%, transparent);
  color: var(--color-on-success-container);
}

.status-banner--closed .status-banner__icon {
  background: color-mix(in srgb, var(--color-success) 16%, transparent);
  color: var(--color-success);
}

.status-banner--held {
  background: color-mix(in srgb, var(--color-success-container) 100%, transparent);
  color: var(--color-on-success-container);
}

.status-banner--held .status-banner__icon {
  background: color-mix(in srgb, var(--color-success) 16%, transparent);
  color: var(--color-success);
}

.status-banner--draft {
  background: var(--color-surface-container-low);
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

.side-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--space-3);
}

.card-title {
  font-size: var(--text-lg);
}

.section-subtitle,
.side-subtitle,
.modal-subtitle {
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

.modal-subtitle {
  margin: 0;
}

.resolution-list,
.doc-list,
.attendance-list,
.attendance-edit-list {
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
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
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

.resolution-title {
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: var(--fw-semibold);
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
  color: var(--color-on-surface-variant);
}

.tally-track {
  height: 8px;
  border-radius: var(--radius-full);
  overflow: hidden;
  background: var(--color-surface-container-high);
}

.tally-fill {
  height: 100%;
  border-radius: var(--radius-full);
}

.tally-fill--for {
  background: linear-gradient(135deg, #1b5e20, #43a047);
}

.tally-fill--against {
  background: linear-gradient(135deg, #ba1a1a, #e53935);
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

.result-pill,
.pending-box,
.minutes-pill,
.quorum-kpi {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: var(--space-3);
  border-radius: var(--radius-md);
}

.result-pill {
  background: var(--tone-bg, transparent);
  color: var(--tone-fg, inherit);
}

.pending-box {
  background: var(--color-surface-container-low);
  color: var(--color-on-surface-variant);
}

.vote-entry-action {
  display: flex;
  justify-content: flex-end;
}

.vote-quick-actions {
  display: flex;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.vote-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--text-sm);
}

.vote-table th {
  text-align: left;
  padding: var(--space-2) var(--space-3);
  font-size: var(--text-xs);
  font-weight: var(--fw-medium);
  color: var(--color-on-surface-variant);
  border-bottom: 1px solid var(--color-outline-variant);
}

.vote-table td {
  padding: var(--space-2) var(--space-3);
  border-bottom: 1px solid var(--color-outline-variant);
  vertical-align: middle;
}

.vote-table tbody tr:last-child td {
  border-bottom: none;
}

.vote-table__unit {
  width: 64px;
  white-space: nowrap;
}

.vote-table__owner {
  min-width: 120px;
}

.vote-table__select {
  width: 160px;
}

.vote-table__select--empty :deep(select) {
  box-shadow: var(--shadow-error-ring);
}

.vote-result {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-md);
}

.vote-result--passed {
  background: color-mix(in srgb, var(--color-success-container) 100%, transparent);
  color: var(--color-on-success-container);
}

.vote-result--rejected {
  background: color-mix(in srgb, var(--color-error-container) 100%, transparent);
  color: var(--color-on-error-container);
}

.vote-result__text,
.vote-result__sub {
  margin: 0;
}

.vote-result__text {
  font-size: var(--text-sm);
  font-weight: var(--fw-semibold);
}

.vote-result__sub {
  font-size: var(--text-xs);
  opacity: 0.82;
}

.vote-result__value {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: var(--fw-medium);
  white-space: nowrap;
}

.quorum-kpi {
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
}

.quorum-kpi strong {
  font-size: 28px;
  font-family: var(--font-display);
}

.info-list__row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-3);
}

.info-list__row dt,
.doc-kind,
.attendance-row p,
.attendance-edit-row__identity span {
  color: var(--color-on-surface-variant);
}

.info-list__row dd {
  margin: 0;
  text-align: right;
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

.doc-row,
.attendance-edit-row {
  display: flex;
  justify-content: space-between;
  gap: var(--space-3);
  align-items: center;
  padding: var(--space-3);
  border-radius: var(--radius-md);
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

.presence-kpi__label,
.presence-kpi__value,
.presence-kpi__detail {
  margin: 0;
}

.presence-kpi__label {
  font-size: var(--text-xs);
}

.presence-kpi__value {
  font-family: var(--font-display);
  font-size: 40px;
  font-weight: var(--fw-bold);
  letter-spacing: -0.03em;
  line-height: 1;
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
  border-radius: var(--radius-sm);
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

.attendance-avatar {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-family: var(--font-display);
  font-size: var(--text-2xs);
  font-weight: var(--fw-bold);
  background: color-mix(in srgb, var(--color-primary) 16%, var(--color-surface));
  color: var(--color-primary);
}

.attendance-shares {
  font-family: var(--font-mono);
  color: var(--color-on-surface-variant);
}

.agenda-editor {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.agenda-editor__row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--space-2);
}

.form-field--full {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.attendance-edit-row {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(160px, 220px) minmax(0, 1fr);
}

.attendance-edit-row__identity {
  display: flex;
  flex-direction: column;
  gap: 4px;
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
  width: 20px;
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

.empty-state--compact {
  padding: var(--space-4);
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

  .attendance-edit-row {
    grid-template-columns: 1fr;
  }

  .agenda-editor__row {
    grid-template-columns: 1fr;
  }
}
</style>
