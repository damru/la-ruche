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
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'

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

const { schedule: scheduleMeetingClose, forceExpire: forceMeetingClose } = useAutoClose()
const { schedule: scheduleResolutionClose, forceExpire: forceResolutionClose } = useAutoClose()
const { schedule: scheduleDocumentClose, forceExpire: forceDocumentClose } = useAutoClose()
const { schedule: scheduleAttendanceClose, forceExpire: forceAttendanceClose } = useAutoClose()

watch(showEditMeetingModal, (open) => { if (!open) forceMeetingClose() })
watch(showAddResolutionModal, (open) => { if (!open) forceResolutionClose() })
watch(showDocumentModal, (open) => { if (!open) forceDocumentClose() })
watch(showAttendanceModal, (open) => { if (!open) forceAttendanceClose() })

function defaultMeetingForm() {
  return {
    meeting_type: 'annual' as MeetingType,
    meeting_date: '',
    location: '',
    agenda: '',
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

const canEditMeeting = computed(() => auth.isManager && meeting.value?.status === 'draft')
const canAddResolution = computed(() => auth.isManager && ['draft', 'called'].includes(meeting.value?.status ?? ''))
const canCallMeeting = computed(() => auth.isManager && meeting.value?.status === 'draft')
const noticeDocuments = computed(() => documents.value.filter(document => document.kind === 'notice'))
const attachmentDocuments = computed(() => documents.value.filter(document => document.kind === 'attachment'))
const attendancePct = computed(() => {
  if (!attendance.value.total_shares) return 0
  return Math.round((attendance.value.shares_present / attendance.value.total_shares) * 100)
})
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
onMounted(reload)

function typeLabel(type: MeetingType): string {
  return t(`meetings.typeLabel.${type}`)
}

function meetingStatusLabel(status: Meeting['status']): string {
  return t(`meetings.statuses.${status}`)
}

function meetingStatusChip(status: Meeting['status']): string {
  if (status === 'called') return 'tone--default'
  if (status === 'closed') return 'tone--valid'
  return 'tone--disabled'
}

function resolutionStatusLabel(status: Resolution['status']): string {
  return t(`meetings.resolutionStatuses.${status}`)
}

function resolutionStatusChip(status: Resolution['status']): string {
  if (status === 'passed') return 'tone--valid'
  if (status === 'rejected') return 'tone--err'
  return 'tone--disabled'
}

function voteTypeLabel(voteType: VoteType): string {
  const shortKey = `meetings.vote.typeShort.${voteType}`
  const shortLabel = t(shortKey)
  return shortLabel === shortKey ? t(`meetings.vote.types.${voteType}`) : shortLabel
}

function voteTypeChip(voteType: VoteType): string {
  if (voteType === 'simple_majority') return 'tone--default'
  if (voteType === 'absolute_majority') return 'chip-art25'
  if (voteType === 'double_majority') return 'chip-art26'
  return 'tone--muted'
}

function votePct(value: number, total: number): number {
  if (!total) return 0
  return Math.round((value / total) * 100)
}

function attendanceModeLabel(mode: MeetingAttendanceMode): string {
  return t(`meetings.attendanceModes.${mode}`)
}

function attendanceModeChip(mode: MeetingAttendanceMode): string {
  if (mode === 'present') return 'tone--valid'
  if (mode === 'proxy') return 'tone--default'
  if (mode === 'correspondence') return 'tone--muted'
  return 'tone--disabled'
}

function documentKindLabel(kind: MeetingDocument['kind']): string {
  return t(`meetings.documents.${kind}`)
}

function openEditMeetingModal() {
  if (!meeting.value) return
  meetingForm.value = {
    meeting_type: meeting.value.meeting_type,
    meeting_date: toDateTimeLocal(meeting.value.meeting_date),
    location: meeting.value.location ?? '',
    agenda: meeting.value.agenda ?? '',
  }
  meetingFeedback.reset()
  showEditMeetingModal.value = true
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

  try {
    await meetingsApi.update(meeting.value.id, {
      meeting_type: meetingForm.value.meeting_type,
      meeting_date: new Date(meetingForm.value.meeting_date).toISOString(),
      location: meetingForm.value.location || undefined,
      agenda: meetingForm.value.agenda || undefined,
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
        <CBtn
          v-if="canCallMeeting"
          variant="secondary"
          size="sm"
          :loading="callingMeeting"
          @click="callMeeting"
        >
          <CIconBell />
          <span class="btn-label">{{ t('meetings.call') }}</span>
        </CBtn>
        <CBtn
          v-if="canEditMeeting"
          variant="secondary"
          size="sm"
          @click="openEditMeetingModal"
        >
          <CIconEdit />
          <span class="btn-label">{{ t('common.edit') }}</span>
        </CBtn>
        <CBtn
          v-if="canAddResolution"
          variant="primary"
          size="sm"
          @click="openResolutionModal"
        >
          <CIconPlus />
          <span class="btn-label">{{ t('meetings.addResolution') }}</span>
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
          <div class="page-header__eyebrow">
            <span :class="['status-chip', meetingStatusChip(meeting.status)]">
              <span class="status-chip__dot"></span>
              {{ meetingStatusLabel(meeting.status) }}
            </span>
            <span>{{ t('page.eyebrows.assemblyDetail') }}</span>
          </div>

          <h1 class="display-md">
            {{ typeLabel(meeting.meeting_type) }} {{ new Date(meeting.meeting_date).getFullYear() }}
          </h1>

          <div class="page-meta">
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
            <span class="activity-card__line">
              <CIconBell />
              {{ t('meetings.calledAt', { date: meeting.notice_date ? formatDate(meeting.notice_date) : '—' }) }}
            </span>
          </div>
        </header>

        <div class="legal-callout">
          <strong>{{ t('meetings.legalWarning') }}</strong>
          <span>J-21 : {{ noticeDeadline }}</span>
        </div>

        <div class="detail-layout">
          <section class="main-column">
            <div class="section-head">
              <div>
                <h2 class="section-title">
                  {{ t('meetings.resolutions') }}
                </h2>
                <p class="section-subtitle">
                  {{ t('meetings.resolutionCount', resolutions.length) }}
                </p>
              </div>
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
                class="resolution-card"
              >
                <div class="resolution-card__header">
                  <div class="resolution-number">
                    {{ resolution.number }}
                  </div>
                  <div class="resolution-card__body">
                    <h3 class="resolution-title">
                      {{ resolution.title }}
                    </h3>
                    <div class="resolution-chips">
                      <span :class="['status-chip', voteTypeChip(resolution.vote_type)]">{{ voteTypeLabel(resolution.vote_type) }}</span>
                      <span :class="['status-chip', resolutionStatusChip(resolution.status)]">
                        <span class="status-chip__dot"></span>
                        {{ resolutionStatusLabel(resolution.status) }}
                      </span>
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
                  <div class="tally-row">
                    <span>{{ t('meetings.vote.in_favor') }}</span>
                    <div class="tally-track">
                      <div
                        class="tally-fill tally-fill--for"
                        :style="{ width: `${votePct(voteResults[resolution.id].votes_for, voteResults[resolution.id].shares_present)}%` }"
                      ></div>
                    </div>
                    <strong>{{ voteResults[resolution.id].votes_for }}</strong>
                  </div>
                  <div class="tally-row">
                    <span>{{ t('meetings.vote.against') }}</span>
                    <div class="tally-track">
                      <div
                        class="tally-fill tally-fill--against"
                        :style="{ width: `${votePct(voteResults[resolution.id].votes_against, voteResults[resolution.id].shares_present)}%` }"
                      ></div>
                    </div>
                    <strong>{{ voteResults[resolution.id].votes_against }}</strong>
                  </div>
                  <div class="tally-row">
                    <span>{{ t('meetings.vote.abstain') }}</span>
                    <div class="tally-track">
                      <div
                        class="tally-fill tally-fill--abstain"
                        :style="{ width: `${votePct(voteResults[resolution.id].abstentions, voteResults[resolution.id].shares_present)}%` }"
                      ></div>
                    </div>
                    <strong>{{ voteResults[resolution.id].abstentions }}</strong>
                  </div>
                  <div :class="['result-pill', voteResults[resolution.id].adopted ? 'tone--valid' : 'tone--err']">
                    <strong>{{ voteResults[resolution.id].adopted ? t('meetings.vote.passed') : t('meetings.vote.rejected') }}</strong>
                    <span>{{ voteResults[resolution.id].explanation }}</span>
                  </div>
                </div>

                <div
                  v-else
                  class="pending-box"
                >
                  {{ t('meetings.vote.pendingVote') }}
                </div>
              </CCard>
            </div>
          </section>

          <aside class="side-column">
            <CCard>
              <h2 class="side-title">
                {{ t('meetings.info') }}
              </h2>
              <dl class="info-list">
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
              </dl>
            </CCard>

            <CCard>
              <div class="side-head">
                <div>
                  <h2 class="side-title">
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

            <CCard>
              <div class="side-head">
                <div>
                  <h2 class="side-title">
                    {{ t('meetings.presenceQuorum') }}
                  </h2>
                  <p class="side-subtitle">
                    {{ t('meetings.attendanceSummary', { count: attendance.present_units }) }}
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

              <div class="quorum-kpi">
                <strong>{{ attendance.shares_present }}</strong>
                <span>{{ t('meetings.sharesRepresented') }} / {{ attendance.total_shares }}</span>
                <small>{{ attendancePct }}%</small>
              </div>

              <div class="attendance-list">
                <div
                  v-for="entry in attendance.entries"
                  :key="entry.unit_id"
                  class="attendance-row"
                >
                  <div>
                    <strong>{{ entry.unit_number }}</strong>
                    <p>{{ entry.owner_name || '—' }}</p>
                  </div>
                  <div class="attendance-row__meta">
                    <span :class="['status-chip', attendanceModeChip(entry.mode)]">{{ attendanceModeLabel(entry.mode) }}</span>
                    <small>{{ entry.shares_represented }}</small>
                  </div>
                </div>
              </div>
            </CCard>

            <CCard>
              <h2 class="side-title">
                {{ t('meetings.agenda') }}
              </h2>
              <ol class="agenda-list">
                <li
                  v-for="(item, index) in agendaItems"
                  :key="`${index}-${item}`"
                >
                  {{ item }}
                </li>
              </ol>
            </CCard>
          </aside>
        </div>
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
        <label class="form-field">
          <span class="form-label">{{ t('meetings.infoFields.type') }}</span>
          <select
            v-model="meetingForm.meeting_type"
            class="form-input"
          >
            <option value="annual">{{ t('meetings.types.annual') }}</option>
            <option value="extraordinary">{{ t('meetings.types.extraordinary') }}</option>
          </select>
        </label>
        <label class="form-field">
          <span class="form-label">{{ t('meetings.meetingDate') }}</span>
          <input
            v-model="meetingForm.meeting_date"
            class="form-input"
            type="datetime-local"
          />
        </label>
        <label class="form-field">
          <span class="form-label">{{ t('meetings.location') }}</span>
          <input
            v-model="meetingForm.location"
            class="form-input"
            type="text"
          />
        </label>
        <label class="form-field">
          <span class="form-label">{{ t('meetings.agenda') }}</span>
          <textarea
            v-model="meetingForm.agenda"
            class="form-input"
          ></textarea>
        </label>
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
        <label class="form-field">
          <span class="form-label">#</span>
          <input
            v-model.number="resolutionForm.number"
            class="form-input"
            type="number"
            min="1"
          />
        </label>
        <label class="form-field">
          <span class="form-label">{{ t('meetings.resolutionLabel') }}</span>
          <input
            v-model="resolutionForm.title"
            class="form-input"
            type="text"
            :placeholder="t('meetings.resolutionLabelPlaceholder')"
          />
        </label>
        <label class="form-field">
          <span class="form-label">{{ t('meetings.resolutionDescription') }}</span>
          <textarea
            v-model="resolutionForm.description"
            class="form-input"
            :placeholder="t('meetings.resolutionDescriptionPlaceholder')"
          ></textarea>
        </label>
        <label class="form-field">
          <span class="form-label">{{ t('meetings.voteRule') }}</span>
          <select
            v-model="resolutionForm.vote_type"
            class="form-input"
          >
            <option value="simple_majority">{{ t('meetings.vote.types.simple_majority') }}</option>
            <option value="absolute_majority">{{ t('meetings.vote.types.absolute_majority') }}</option>
            <option value="double_majority">{{ t('meetings.vote.types.double_majority') }}</option>
            <option value="unanimity">{{ t('meetings.vote.types.unanimity') }}</option>
          </select>
        </label>
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
      :open="showDocumentModal"
      :title="t('meetings.documents.uploadTitle')"
      @update:open="showDocumentModal = $event"
    >
      <form
        class="form-grid"
        @submit.prevent="saveDocument"
      >
        <label class="form-field">
          <span class="form-label">{{ t('meetings.documents.uploadType') }}</span>
          <select
            v-model="documentForm.kind"
            class="form-input"
          >
            <option value="notice">{{ t('meetings.documents.notice') }}</option>
            <option value="attachment">{{ t('meetings.documents.attachment') }}</option>
          </select>
        </label>
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
            <select
              v-model="entry.mode"
              class="form-input"
            >
              <option value="present">
                {{ t('meetings.attendanceModes.present') }}
              </option>
              <option value="proxy">
                {{ t('meetings.attendanceModes.proxy') }}
              </option>
              <option value="correspondence">
                {{ t('meetings.attendanceModes.correspondence') }}
              </option>
              <option value="absent">
                {{ t('meetings.attendanceModes.absent') }}
              </option>
            </select>
            <input
              v-model="entry.representative_name"
              class="form-input"
              type="text"
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

.page-header__eyebrow {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-2xs);
  font-weight: var(--fw-semibold);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-primary);
}

.page-meta {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
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

.legal-callout {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  padding: var(--space-4);
  border-radius: var(--radius-lg);
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
}

.detail-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 340px;
  gap: var(--space-4);
  align-items: start;
}

.main-column,
.side-column {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.section-head,
.side-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--space-3);
}

.section-title,
.side-title {
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: var(--fw-semibold);
}

.section-subtitle,
.side-subtitle,
.modal-subtitle {
  color: var(--color-on-surface-variant);
  font-size: var(--text-sm);
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
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
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
  font-size: 18px;
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

.tally-row {
  display: grid;
  grid-template-columns: 88px minmax(0, 1fr) 36px;
  align-items: center;
  gap: var(--space-3);
  font-size: var(--text-sm);
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

.quorum-kpi {
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
}

.quorum-kpi strong {
  font-size: 28px;
  font-family: var(--font-display);
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.info-list__row {
  display: flex;
  justify-content: space-between;
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
.attendance-row,
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
.attendance-row__meta {
  display: flex;
  align-items: center;
  gap: var(--space-2);
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

.attendance-row p {
  margin: 2px 0 0;
  font-size: var(--text-xs);
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
  padding-left: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
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
    grid-template-columns: 72px minmax(0, 1fr) 32px;
  }

  .attendance-edit-row {
    grid-template-columns: 1fr;
  }
}
</style>
