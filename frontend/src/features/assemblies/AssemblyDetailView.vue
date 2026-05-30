<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useAuthStore } from '@/stores/auth'
import type {
  ConvocationChannel,
  ConvocationChecklist,
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
import CIconBell from '@/components/atoms/CIconBell.svg'
import CIconCheck from '@/components/atoms/CIconCheck.svg'
import CIconClock from '@/components/atoms/CIconClock.svg'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconLock from '@/components/atoms/CIconLock.svg'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconUpload from '@/components/atoms/CIconUpload.svg'

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
const showCloseMeetingModal = ref(false)
const closeMeetingFeedback = useFormFeedback()

const showDeleteDocumentModal = ref(false)
const pendingDeleteDocumentId = ref<string | null>(null)
const pendingDeleteDocumentName = ref<string | null>(null)

const showVoteModal = ref(false)
const votingResolution = ref<Resolution | null>(null)
const unitVotes = ref<Record<string, string>>({})
const savingVotes = ref(false)
const voteFeedback = useFormFeedback()

const startingMeeting = ref(false)

const showStartMeetingModal = ref(false)
const startMeetingForm = ref(defaultStartMeetingForm())
const startMeetingFeedback = useFormFeedback()

function defaultStartMeetingForm() {
  return {
    president_name: '',
    secretary_name: '',
    scrutineers: [] as Array<{ name: string }>,
  }
}

// EP-003-S09 — Convocation : checklist + canal + date d'envoi
const showConvocationModal = ref(false)
const convocationForm = ref<{ channel: ConvocationChannel; sent_at: string }>({
  channel: 'lrar',
  sent_at: '',
})
const convocationChecklist = ref<ConvocationChecklist | null>(null)
const loadingConvocationChecklist = ref(false)
const convocationFeedback = useFormFeedback()

const showMinutesModal = ref(false)
const minutesForm = ref<{ file: File | null }>({ file: null })
const savingMinutes = ref(false)
const downloadingMinutesDraft = ref(false)
const notifyingMinutes = ref(false)
const minutesFeedback = useFormFeedback()

const showSignaturesModal = ref(false)
const signaturesForm = ref({
  president_signed: false,
  secretary_signed: false,
  scrutineers_signed: [] as string[],
})
const savingSignatures = ref(false)
const signaturesFeedback = useFormFeedback()

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
const editingResolution = ref<Resolution | null>(null)
const editResolutionForm = ref({ description: '' })
const savingEditResolution = ref(false)
const editResolutionFeedback = useFormFeedback()

const showArt24Modal = ref(false)
const art24ResolutionToCreate = ref<Resolution | null>(null)
const savingArt24 = ref(false)
const art24Feedback = useFormFeedback()

const showArt25Modal = ref(false)
const art25ResolutionToCreate = ref<Resolution | null>(null)
const savingArt25 = ref(false)
const art25Feedback = useFormFeedback()

const { schedule: scheduleMeetingClose, forceExpire: forceMeetingClose } = useAutoClose()
const { schedule: scheduleResolutionClose, forceExpire: forceResolutionClose } = useAutoClose()
const { schedule: scheduleDocumentClose, forceExpire: forceDocumentClose } = useAutoClose()
const { schedule: scheduleAttendanceClose, forceExpire: forceAttendanceClose } = useAutoClose()
const { schedule: scheduleVoteClose, forceExpire: forceVoteClose } = useAutoClose()
const { schedule: scheduleEditResolutionClose, forceExpire: forceEditResolutionClose } = useAutoClose()
const { schedule: scheduleArt24Close, forceExpire: forceArt24Close } = useAutoClose()
const { schedule: scheduleArt25Close, forceExpire: forceArt25Close } = useAutoClose()

watch(showEditMeetingModal, (open) => { if (!open) forceMeetingClose() })
watch(showAddResolutionModal, (open) => { if (!open) forceResolutionClose() })
watch(showDocumentModal, (open) => { if (!open) forceDocumentClose() })
watch(showAttendanceModal, (open) => { if (!open) forceAttendanceClose() })
watch(showVoteModal, (open) => { if (!open) forceVoteClose() })
watch(showEditResolutionModal, (open) => { if (!open) forceEditResolutionClose() })
watch(showArt24Modal, (open) => { if (!open) forceArt24Close() })
watch(showArt25Modal, (open) => { if (!open) forceArt25Close() })

const { schedule: scheduleStartMeetingClose, forceExpire: forceStartMeetingClose } = useAutoClose()
watch(showStartMeetingModal, (open) => { if (!open) forceStartMeetingClose() })

const { schedule: scheduleMinutesClose, forceExpire: forceMinutesClose } = useAutoClose()
watch(showMinutesModal, (open) => { if (!open) forceMinutesClose() })

const { schedule: scheduleConvocationClose, forceExpire: forceConvocationClose } = useAutoClose()
watch(showConvocationModal, (open) => { if (!open) forceConvocationClose() })

const { schedule: scheduleSignaturesClose, forceExpire: forceSignaturesClose } = useAutoClose()
watch(showSignaturesModal, (open) => { if (!open) forceSignaturesClose() })

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
const canManageVotes = computed(() => auth.isManager && ['ongoing'].includes(meeting.value?.status ?? ''))
const canEditResolution = computed(() => auth.isManager && ['draft', 'ongoing'].includes(meeting.value?.status ?? ''))
const canEditAttendance = computed(() => auth.isManager && meeting.value?.status !== 'closed')
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
  const [buildingRes, meetingRes, resolutionRes, documentsRes, attendanceRes] = await Promise.all([
    buildingApi.get(),
    meetingsApi.get(meetingId),
    meetingsApi.listResolutions(meetingId),
    meetingsApi.listDocuments(meetingId),
    meetingsApi.attendance(meetingId),
  ])

  buildingName.value = buildingRes.data.name
  buildingSharesTotal.value = buildingRes.data.shares_total
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
  { value: 'vote_by_mail_form', label: t('meetings.documents.vote_by_mail_form') },
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
    meetingFeedback.setError(toErrorMessage(err))
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
    resolutionFeedback.setError(toErrorMessage(err))
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

    // Attempt finalization — server checks if all attending units have voted
    const finalizeRes = await meetingsApi.finalizeResolution(
      meeting.value.id,
      votingResolution.value.id,
    )

    if (finalizeRes.data.finalized && finalizeRes.data.result) {
      // Update local resolution status immediately without a full reload
      const rid = votingResolution.value.id
      resolutions.value = resolutions.value.map(r =>
        r.id === rid
          ? { ...r, status: finalizeRes.data.result!.passed ? 'passed' : 'rejected' }
          : r,
      )
      voteResults.value = { ...voteResults.value, [rid]: finalizeRes.data.result }
      const msgKey = finalizeRes.data.eligible_for_second_vote === 'art_25_1'
        ? 'meetings.vote.eligibleSecondVote25_1'
        : finalizeRes.data.eligible_for_second_vote === 'art_26_1'
          ? 'meetings.vote.eligibleSecondVote26_1'
          : finalizeRes.data.result.passed
            ? 'meetings.vote.passed'
            : 'meetings.vote.rejected'
      voteFeedback.setSuccess(t(msgKey))
    } else {
      // Votes saved but not all units have voted yet — just refresh the tally
      const result = await meetingsApi.results(meeting.value.id, votingResolution.value.id)
      voteResults.value = { ...voteResults.value, [votingResolution.value.id]: result.data }
      voteFeedback.setSuccess(t('meetings.vote.voteSaved'))
    }

    // EP-013-S03: Re-fetch attendance after each vote so the denominator
    // (shares_present) reflects the updated server-side AG record.
    const attendanceRes = await meetingsApi.attendance(meeting.value.id)
    attendance.value = attendanceRes.data

    scheduleVoteClose(async () => { showVoteModal.value = false })
  } catch (err) {
    voteFeedback.setError(toErrorMessage(err))
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
    documentFeedback.setError(toErrorMessage(err))
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
    documentFeedback.setError(toErrorMessage(err))
  }
}

function promptDeleteDocument(documentId: string, name: string) {
  pendingDeleteDocumentId.value = documentId
  pendingDeleteDocumentName.value = name
  showDeleteDocumentModal.value = true
}

async function confirmDetachDocument() {
  if (!pendingDeleteDocumentId.value) return
  await detachDocument(pendingDeleteDocumentId.value)
  showDeleteDocumentModal.value = false
  pendingDeleteDocumentId.value = null
  pendingDeleteDocumentName.value = null
}

async function openDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch (err) {
    documentFeedback.setError(toErrorMessage(err))
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
    attendanceFeedback.setError(toErrorMessage(err))
  } finally {
    savingAttendance.value = false
  }
}

// EP-003-S09 — Ouvre la modale convocation (checklist + canal + date d'envoi).
async function openConvocationModal() {
  if (!meeting.value) return
  convocationFeedback.reset()
  convocationForm.value = { channel: 'lrar', sent_at: '' }
  loadingConvocationChecklist.value = true
  try {
    const res = await meetingsApi.getConvocationChecklist(meeting.value.id)
    convocationChecklist.value = res.data
  } catch (err) {
    convocationFeedback.setError(toErrorMessage(err))
  } finally {
    loadingConvocationChecklist.value = false
  }
  showConvocationModal.value = true
}

async function confirmCallMeeting() {
  if (!meeting.value) return
  if (!convocationChecklist.value?.ready_to_send) {
    convocationFeedback.setError(t('meetings.convocation.checklist_incomplete'))
    return
  }
  callingMeeting.value = true
  convocationFeedback.reset()
  try {
    await meetingsApi.call(meeting.value.id, {
      channel: convocationForm.value.channel,
      sent_at: convocationForm.value.sent_at || undefined,
    })
    convocationFeedback.setSuccess(t('common.saved'))
    scheduleConvocationClose(async () => {
      showConvocationModal.value = false
      await reload()
    })
  } catch (err) {
    convocationFeedback.setError(toErrorMessage(err))
  } finally {
    callingMeeting.value = false
  }
}

async function closeMeeting() {
  if (!meeting.value) return

  closingMeeting.value = true
  closeMeetingFeedback.reset()

  try {
    await meetingsApi.close(meeting.value.id)
    showCloseMeetingModal.value = false
    await reload()
  } catch (err) {
    closeMeetingFeedback.setError(toErrorMessage(err))
  } finally {
    closingMeeting.value = false
  }
}

function openStartMeetingModal() {
  startMeetingForm.value = defaultStartMeetingForm()
  startMeetingFeedback.reset()
  showStartMeetingModal.value = true
}

function addScrutineer() {
  startMeetingForm.value.scrutineers.push({ name: '' })
}

function removeScrutineer(index: number) {
  startMeetingForm.value.scrutineers.splice(index, 1)
}

async function downloadMinutesDraft() {
  if (!meeting.value) return
  downloadingMinutesDraft.value = true
  try {
    const response = await meetingsApi.getMinutesDraft(meeting.value.id)
    const blob = new Blob([response.data.markdown], { type: 'text/markdown;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `pv-ag-${meeting.value.id}.md`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
  } catch (err) {
    minutesFeedback.setError(toErrorMessage(err))
  } finally {
    downloadingMinutesDraft.value = false
  }
}

function openMinutesModal() {
  minutesForm.value = { file: null }
  minutesFeedback.reset()
  showMinutesModal.value = true
}

function openSignaturesModal() {
  if (!meeting.value) return
  signaturesForm.value = {
    president_signed: !!meeting.value.minutes_president_signed_at,
    secretary_signed: !!meeting.value.minutes_secretary_signed_at,
    scrutineers_signed: (meeting.value.minutes_scrutineers_signed ?? []).map(s => s.name),
  }
  signaturesFeedback.reset()
  showSignaturesModal.value = true
}

async function saveSignatures() {
  if (!meeting.value) return
  savingSignatures.value = true
  signaturesFeedback.reset()
  try {
    await meetingsApi.signMinutes(meeting.value.id, signaturesForm.value)
    signaturesFeedback.setSuccess(t('common.saved'))
    scheduleSignaturesClose(async () => {
      showSignaturesModal.value = false
      await reload()
    })
  } catch (err) {
    signaturesFeedback.setError(toErrorMessage(err))
  } finally {
    savingSignatures.value = false
  }
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

async function saveMinutes() {
  if (!meeting.value || !minutesForm.value.file) return
  savingMinutes.value = true
  minutesFeedback.reset()
  try {
    const file = minutesForm.value.file
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
    await meetingsApi.attachMinutes(meeting.value.id, confirmed.data.id)
    minutesFeedback.setSuccess(t('common.saved'))
    scheduleMinutesClose(async () => {
      showMinutesModal.value = false
      await reload()
    })
  } catch (err) {
    minutesFeedback.setError(toErrorMessage(err))
  } finally {
    savingMinutes.value = false
  }
}

async function confirmStartMeeting() {
  if (!meeting.value) return
  if (!startMeetingForm.value.president_name.trim() || !startMeetingForm.value.secretary_name.trim()) {
    startMeetingFeedback.setError(t('meetings.start_modal.required'))
    return
  }
  startingMeeting.value = true
  startMeetingFeedback.reset()
  try {
    await meetingsApi.start(meeting.value.id, {
      president_name: startMeetingForm.value.president_name.trim(),
      secretary_name: startMeetingForm.value.secretary_name.trim(),
      scrutineers: startMeetingForm.value.scrutineers
        .map(s => ({ name: s.name.trim() }))
        .filter(s => s.name.length > 0),
    })
    startMeetingFeedback.setSuccess(t('common.saved'))
    scheduleStartMeetingClose(async () => {
      showStartMeetingModal.value = false
      await reload()
    })
  } catch (err) {
    startMeetingFeedback.setError(toErrorMessage(err))
  } finally {
    startingMeeting.value = false
  }
}

function openEditResolutionModal(resolution: Resolution) {
  editingResolution.value = resolution
  editResolutionForm.value = { description: resolution.description ?? '' }
  editResolutionFeedback.reset()
  showEditResolutionModal.value = true
}

async function saveEditResolution() {
  if (!editingResolution.value || !meeting.value) return
  savingEditResolution.value = true
  editResolutionFeedback.reset()
  try {
    if (meeting.value.status === 'draft') {
      await meetingsApi.updateResolution(meeting.value.id, editingResolution.value.id, {
        number: editingResolution.value.number,
        title: editingResolution.value.title,
        description: editResolutionForm.value.description || undefined,
        vote_type: editingResolution.value.vote_type,
      })
    } else {
      await meetingsApi.patchResolution(meeting.value.id, editingResolution.value.id, {
        description: editResolutionForm.value.description || undefined,
      })
    }
    editResolutionFeedback.setSuccess(t('common.saved'))
    scheduleEditResolutionClose(async () => {
      showEditResolutionModal.value = false
      await reload()
    })
  } catch (err) {
    editResolutionFeedback.setError(toErrorMessage(err))
  } finally {
    savingEditResolution.value = false
  }
}

function openArt24Modal(resolution: Resolution) {
  art24ResolutionToCreate.value = resolution
  art24Feedback.reset()
  showArt24Modal.value = true
}

async function confirmArt24Resolution() {
  if (!meeting.value || !art24ResolutionToCreate.value) return
  savingArt24.value = true
  art24Feedback.reset()
  try {
    await meetingsApi.createResolution(meeting.value.id, {
      number: resolutions.value.length + 1,
      title: `${art24ResolutionToCreate.value.title} (Art. 24)`,
      description: art24ResolutionToCreate.value.description,
      vote_type: 'simple_majority',
    })
    art24Feedback.setSuccess(t('common.saved'))
    scheduleArt24Close(async () => {
      showArt24Modal.value = false
      await reload()
    })
  } catch (err) {
    art24Feedback.setError(toErrorMessage(err))
  } finally {
    savingArt24.value = false
  }
}

function openArt25Modal(resolution: Resolution) {
  art25ResolutionToCreate.value = resolution
  art25Feedback.reset()
  showArt25Modal.value = true
}

async function confirmArt25Resolution() {
  if (!meeting.value || !art25ResolutionToCreate.value) return
  savingArt25.value = true
  art25Feedback.reset()
  try {
    await meetingsApi.createResolution(meeting.value.id, {
      number: resolutions.value.length + 1,
      title: `${art25ResolutionToCreate.value.title} (Art. 25)`,
      description: art25ResolutionToCreate.value.description,
      vote_type: 'absolute_majority',
    })
    art25Feedback.setSuccess(t('common.saved'))
    scheduleArt25Close(async () => {
      showArt25Modal.value = false
      await reload()
    })
  } catch (err) {
    art25Feedback.setError(toErrorMessage(err))
  } finally {
    savingArt25.value = false
  }
}

function toDateTimeLocal(value: string): string {
  const date = new Date(value)
  const offsetMs = date.getTimezoneOffset() * 60_000
  return new Date(date.getTime() - offsetMs).toISOString().slice(0, 16)
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
            :loading="callingMeeting"
            @click="openConvocationModal"
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
          :loading="startingMeeting"
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
                  variant="ghost"
                  size="sm"
                  type="button"
                  :icon="CIconDownload"
                  :title="t('meetings.minutes.download_draft')"
                  :loading="downloadingMinutesDraft"
                  @click="downloadMinutesDraft"
                />
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
                v-for="resolution in resolutions"
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
                  v-if="canEditResolution || canManageVotes" 
                  class="resolution-card__actions"
                >
                  <CBtn
                    v-if="canEditResolution"
                    size="sm"
                    variant="ghost"
                    :title="t('meetings.editResolution')"
                    @click="openEditResolutionModal(resolution)"
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
                    {{ meeting.secretary_name }}azdazd zef zr
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

              <CFormFeedback
                :status="documentFeedback.status.value"
                :message="documentFeedback.message.value"
              />

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
                  <span class="text-[20px] ">/ {{ attendance.total_shares }}</span>
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

    <CModal
      :open="showEditMeetingModal"
      :title="t('common.edit')"
      @update:open="showEditMeetingModal = $event"
    >
      <form
        class="flex flex-col gap-4"
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
        <div class="flex flex-col gap-1 col-span-full">
          <span class="text-sm font-medium text-on-surface-variant">{{ t('meetings.agenda.title') }}</span>
          <div class="agenda-editor">
            <div
              v-for="(_, index) in meetingAgendaItems"
              :key="`agenda-${index}`"
              class="agenda-editor__row"
            >
              <CFormInput
                v-model="meetingAgendaItems[index]"
                :placeholder="`${t('meetings.agenda.title')} ${index + 1}`"
              />
              <CBtn
                variant="icon"
                size="sm"
                :icon="CIconTrash"
                :title="t('common.delete')"
                :aria-label="t('common.delete')"
                @click="removeAgendaItem(index)"
                :disabled="meetingAgendaItems.length <= 1"
              />
            </div>

            <CBtn
              variant="secondary"
              size="sm"
              :icon="CIconPlus"
              :title="t('common.add')"
              @click="addAgendaItem"
            />
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
          :title="t('common.cancel')"
          @click="showEditMeetingModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="savingMeeting"
          @click="saveMeeting"
        />
      </template>
    </CModal>

    <CModal
      :open="showAddResolutionModal"
      :title="t('meetings.newResolution')"
      @update:open="showAddResolutionModal = $event"
    >
      <form
        class="flex flex-col gap-4"
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
          :title="t('common.cancel')"
          @click="showAddResolutionModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.add')"
          :loading="savingResolution"
          @click="saveResolution"
        />
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
            :title="t('meetings.vote.fillAllFor')"
            @click="fillAllVotes('in_favor')"
          />
          <CBtn
            size="sm"
            variant="ghost"
            :title="t('meetings.vote.fillAllAgainst')"
            @click="fillAllVotes('against')"
          />
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
              <td class="font-mono">
                {{ entry.unit_number }}
              </td>
              <td>
                {{ entry.owner_name || '—' }}
              </td>
              <td>
                <CBadge :variant="attendanceModeVariant(entry.mode)">
                  {{ attendanceModeLabel(entry.mode) }}
                </CBadge>
              </td>
              <td :class="[!unitVotes[entry.unit_id] && 'vote-table__select--empty']">
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
          :title="t('common.cancel')"
          @click="showVoteModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('meetings.vote.submit')"
          :loading="savingVotes"
          :disabled="attendingUnits.length === 0"
          @click="saveVotes"
        />
      </template>
    </CModal>

    <CModal
      :open="showEditResolutionModal"
      :title="t('meetings.editResolution')"
      @update:open="showEditResolutionModal = $event"
    >
      <form
        class="flex flex-col gap-4"
        @submit.prevent="saveEditResolution"
      >
        <template v-if="meeting?.status === 'ongoing'">
          <p class="modal-subtitle resolution-locked-notice">
            {{ t('meetings.resolutionDescriptionOnly') }}
          </p>
          <div class="flex flex-col gap-1">
            <span class="text-sm font-medium text-on-surface-variant">{{ t('meetings.resolutionLabel') }}</span>
            <span class="form-static">{{ editingResolution?.title }}</span>
          </div>
        </template>
        <CFormTextarea
          v-model="editResolutionForm.description"
          :label="t('meetings.resolutionDescription')"
          :placeholder="t('meetings.resolutionDescriptionPlaceholder')"
        />
      </form>
      <template #footer>
        <CFormFeedback
          :status="editResolutionFeedback.status.value"
          :message="editResolutionFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showEditResolutionModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="savingEditResolution"
          @click="saveEditResolution"
        />
      </template>
    </CModal>

    <CModal
      :open="showDocumentModal"
      :title="t('meetings.documents.uploadTitle')"
      @update:open="showDocumentModal = $event"
    >
      <form
        class="flex flex-col gap-4"
        @submit.prevent="saveDocument"
      >
        <CFormSelect
          v-model="documentForm.kind"
          :label="t('meetings.documents.uploadType')"
          :options="documentKindOptions"
        />
        <CFormDropzone
          accept="application/pdf,.pdf,.doc,.docx,.png,.jpg,.jpeg"
          :uploading="savingDocument"
          @files="documentForm.file = $event[0] ?? null"
          @error="documentFeedback.setError($event)"
        />
        <p
          v-if="documentForm.file"
          class="text-xs text-on-surface-variant"
        >
          {{ documentForm.file.name }}
        </p>
      </form>

      <template #footer>
        <CFormFeedback
          :status="documentFeedback.status.value"
          :message="documentFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showDocumentModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.add')"
          :loading="savingDocument"
          @click="saveDocument"
        />
      </template>
    </CModal>

    <CModal
      :open="showAttendanceModal"
      :title="t('meetings.attendanceTitle')"
      @update:open="showAttendanceModal = $event"
    >
      <form
        class="flex flex-col gap-4"
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
          :title="t('common.cancel')"
          @click="showAttendanceModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="savingAttendance"
          @click="saveAttendance"
        />
      </template>
    </CModal>

    <CModal
      :open="showArt24Modal"
      :title="t('meetings.createArt24Title')"
      size="sm"
      @update:open="showArt24Modal = $event"
    >
      <p class="body-sm">
        {{ t('meetings.createArt24Message', { title: art24ResolutionToCreate?.title ?? '' }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="art24Feedback.status.value"
          :message="art24Feedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showArt24Modal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.create')"
          :loading="savingArt24"
          @click="confirmArt24Resolution"
        />
      </template>
    </CModal>

    <CModal
      :open="showArt25Modal"
      :title="t('meetings.createArt25Title')"
      size="sm"
      @update:open="showArt25Modal = $event"
    >
      <p class="body-sm">
        {{ t('meetings.createArt25Message', { title: art25ResolutionToCreate?.title ?? '' }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="art25Feedback.status.value"
          :message="art25Feedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showArt25Modal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.create')"
          :loading="savingArt25"
          @click="confirmArt25Resolution"
        />
      </template>
    </CModal>

    <CModal
      :open="showSignaturesModal"
      :title="t('meetings.signatures.modal_title')"
      size="md"
      @update:open="showSignaturesModal = $event"
    >
      <form
        class="flex flex-col gap-4"
        @submit.prevent="saveSignatures"
      >
        <p class="body-sm text-on-surface-variant">
          {{ t('meetings.signatures.modal_lead') }}<sup><a
            href="https://www.legifrance.gouv.fr/loda/article_lc/LEGIARTI000042078689/"
            target="_blank"
            rel="noopener noreferrer"
            class="legal-ref"
          >art.&nbsp;17–17-1</a></sup>
        </p>
        <CFormCheckbox v-model="signaturesForm.president_signed">
          {{ t('meetings.signatures.president') }}
          <template v-if="meeting?.president_name">
            — {{ meeting.president_name }}
          </template>
        </CFormCheckbox>
        <CFormCheckbox v-model="signaturesForm.secretary_signed">
          {{ t('meetings.signatures.secretary') }}
          <template v-if="meeting?.secretary_name">
            — {{ meeting.secretary_name }}
          </template>
        </CFormCheckbox>
        <div
          v-if="(meeting?.scrutineers ?? []).length > 0"
          class="flex flex-col gap-1"
        >
          <span class="label-sm">{{ t('meetings.signatures.scrutineers') }}</span>
          <CFormCheckbox
            v-for="scrutineer in meeting?.scrutineers ?? []"
            :key="scrutineer.name"
            v-model="signaturesForm.scrutineers_signed"
            :value="scrutineer.name"
            :label="scrutineer.name"
          />
        </div>
      </form>
      <template #footer>
        <CFormFeedback
          :status="signaturesFeedback.status.value"
          :message="signaturesFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showSignaturesModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="savingSignatures"
          @click="saveSignatures"
        />
      </template>
    </CModal>

    <CModal
      :open="showCloseMeetingModal"
      :title="t('meetings.closeMeetingTitle')"
      size="sm"
      @update:open="showCloseMeetingModal = $event"
    >
      <p class="body-sm">
        {{ t('meetings.closeMeetingSub') }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="closeMeetingFeedback.status.value"
          :message="closeMeetingFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showCloseMeetingModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('meetings.closeMeetingConfirm')"
          :loading="closingMeeting"
          @click="closeMeeting"
        />
      </template>
    </CModal>

    <CModal
      :open="showDeleteDocumentModal"
      :title="t('meetings.documents.detachTitle')"
      size="sm"
      @update:open="showDeleteDocumentModal = $event"
    >
      <p class="body-sm">
        {{ t('meetings.documents.detachMessage') }}
        <template v-if="pendingDeleteDocumentName">
          <br />
          <span class="font-medium">{{ pendingDeleteDocumentName }}</span>
        </template>
      </p>
      <template #footer>
        <CFormFeedback
          :status="documentFeedback.status.value"
          :message="documentFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showDeleteDocumentModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('meetings.documents.detachConfirm')"
          @click="confirmDetachDocument"
        />
      </template>
    </CModal>

    <!-- EP-003-S09 — Convocation : checklist + canal + date d'envoi -->
    <CModal
      :open="showConvocationModal"
      :title="t('meetings.convocation.modal_title')"
      size="md"
      @update:open="showConvocationModal = $event"
    >
      <form
        class="flex flex-col gap-4"
        @submit.prevent="confirmCallMeeting"
      >
        <p class="body-sm text-on-surface-variant">
          {{ t('meetings.convocation.modal_lead') }}<sup><a
            href="https://www.legifrance.gouv.fr/loda/id/JORFTEXT000000508310"
            target="_blank"
            rel="noopener noreferrer"
            class="legal-ref"
          >art.&nbsp;7–13</a></sup>
        </p>
        <div
          v-if="loadingConvocationChecklist"
          class="text-sm text-on-surface-variant"
        >
          {{ t('common.loading') }}
        </div>
        <div
          v-else-if="convocationChecklist"
          class="flex flex-col gap-2"
        >
          <span class="label-sm">{{ t('meetings.convocation.checklist_title') }}</span>
          <ul class="flex flex-col gap-1">
            <li
              v-for="item in convocationChecklist.items"
              :key="item.key"
              class="flex items-center gap-2 text-sm"
            >
              <CIconCheck
                v-if="item.satisfied"
                class="text-success icon-sm"
              />
              <CIconX
                v-else
                :class="[item.severity === 'Blocking' ? 'text-error' : 'text-on-surface-variant', 'icon-sm']"
              />
              <span>{{ t('meetings.convocation.items.' + item.key) }}</span>
              <span
                v-if="!item.satisfied && item.severity === 'Advisory'"
                class="text-xs text-on-surface-variant"
              >
                ({{ t('meetings.convocation.advisory') }})
              </span>
            </li>
          </ul>
        </div>
        <CFormSelect
          v-model="convocationForm.channel"
          :label="t('meetings.convocation.channel_label')"
          :options="[
            { value: 'lrar', label: t('meetings.convocation.channels.lrar') },
            { value: 'hand_delivery', label: t('meetings.convocation.channels.hand_delivery') },
            { value: 'email', label: t('meetings.convocation.channels.email') },
          ]"
        />
        <CFormInput
          v-model="convocationForm.sent_at"
          type="datetime-local"
          :label="t('meetings.convocation.sent_at_label')"
          :placeholder="t('meetings.convocation.sent_at_placeholder')"
        />
      </form>
      <template #footer>
        <CFormFeedback
          :status="convocationFeedback.status.value"
          :message="convocationFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showConvocationModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('meetings.call')"
          :loading="callingMeeting"
          :disabled="!convocationChecklist?.ready_to_send"
          @click="confirmCallMeeting"
        />
      </template>
    </CModal>

    <CModal
      :open="showMinutesModal"
      :title="t('meetings.minutes.modal_title')"
      size="md"
      @update:open="showMinutesModal = $event"
    >
      <form
        class="flex flex-col gap-4"
        @submit.prevent="saveMinutes"
      >
        <p class="body-sm text-on-surface-variant">
          {{ t('meetings.minutes.modal_lead') }}
        </p>
        <CFormDropzone
          accept="application/pdf,.pdf,.md,.txt"
          :uploading="savingMinutes"
          @files="minutesForm.file = $event[0] ?? null"
          @error="minutesFeedback.setError($event)"
        />
        <p
          v-if="minutesForm.file"
          class="text-xs text-on-surface-variant"
        >
          {{ minutesForm.file.name }}
        </p>
      </form>
      <template #footer>
        <CFormFeedback
          :status="minutesFeedback.status.value"
          :message="minutesFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showMinutesModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('meetings.minutes.upload_signed')"
          :loading="savingMinutes"
          :disabled="!minutesForm.file"
          @click="saveMinutes"
        />
      </template>
    </CModal>

    <CModal
      :open="showStartMeetingModal"
      :title="t('meetings.start_modal.title')"
      size="md"
      @update:open="showStartMeetingModal = $event"
    >
      <form
        class="flex flex-col gap-4"
        @submit.prevent="confirmStartMeeting"
      >
        <p class="body-sm text-on-surface-variant">
          {{ t('meetings.start_modal.lead') }}
        </p>
        <CFormInput
          v-model="startMeetingForm.president_name"
          :label="t('meetings.start_modal.president_label')"
          required
        />
        <CFormInput
          v-model="startMeetingForm.secretary_name"
          :label="t('meetings.start_modal.secretary_label')"
          required
        />
        <div class="flex flex-col gap-2">
          <span class="label-sm">{{ t('meetings.start_modal.scrutineers_label') }}</span>
          <div
            v-for="(scrutineer, index) in startMeetingForm.scrutineers"
            :key="index"
            class="flex gap-2 items-center"
          >
            <CFormInput
              v-model="scrutineer.name"
              :placeholder="t('meetings.start_modal.scrutineer_placeholder')"
              class="grow"
            />
            <CBtn
              variant="ghost"
              size="sm"
              type="button"
              :icon="CIconTrash"
              :title="t('common.delete')"
              @click="removeScrutineer(index)"
            />
          </div>
          <CBtn
            variant="ghost"
            size="sm"
            type="button"
            :icon="CIconPlus"
            :title="t('meetings.start_modal.add_scrutineer')"
            @click="addScrutineer"
          />
        </div>
      </form>
      <template #footer>
        <CFormFeedback
          :status="startMeetingFeedback.status.value"
          :message="startMeetingFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="showStartMeetingModal = false"
        />
        <CBtn
          variant="primary"
          :title="t('meetings.start')"
          :loading="startingMeeting"
          @click="confirmStartMeeting"
        />
      </template>
    </CModal>
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
  width: 40px;
  height: 40px;
  border-radius: var(--radius-xl);
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

.pending-box {
  display: flex;
  flex-direction: column;
  gap: 4px;
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

.resolution-locked-notice {
  grid-column: 1 / -1;
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
  padding: var(--space-2) var(--space-3);
  background: var(--color-surface-container-low);
  border-radius: var(--radius-lg);
  margin: 0;
}

.form-static {
  font-size: var(--text-sm);
  color: var(--color-on-surface);
  padding: var(--space-2) 0;
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

.vote-table__select--empty :deep(select) {
  box-shadow: var(--shadow-error-ring);
}

.vote-result {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-xl);
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
