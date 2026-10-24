<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useConfirm } from '@/composables/useConfirm'
import { useAuthStore } from '@/stores/auth'
import { usePermissions } from '@/composables/usePermissions'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconCheckCircle from '@/components/atoms/CIconCheckCircle.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconEdit from '@/components/atoms/CIconEdit.svg'
import CIconBell from '@/components/atoms/CIconBell.svg'
import CreateConsultationModal from './CreateConsultationModal.vue'
import CloseConsultationModal from './CloseConsultationModal.vue'
import DeleteConsultationModal from './DeleteConsultationModal.vue'
import { consultationsApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
import type { Consultation, PollStatus } from '@/types/domain'
import { ALL_FILTER, POLL_STATUS_CLOSED, POLL_STATUS_DRAFT, POLL_STATUS_OPEN } from '@/utils/constants'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()
const authStore = useAuthStore()
const perms = usePermissions()
const { confirm } = useConfirm()

// Managers and board members manage consultations (close, remind, delete others).
const canManage = perms.canManageConsultation
// Any authenticated user can create a consultation (EP-010: initiable par tout resident).
const canCreate = computed(() => authStore.isAuthenticated)

const buildingName = ref('')
const allConsultations = ref<Consultation[]>([])
const statusFilter = ref<typeof ALL_FILTER | PollStatus>(ALL_FILTER)
const pageError = ref('')

// ── Create / edit modal ──

// ── Create modal ───────────────────────────────────────────────────
const showCreateModal = ref(false)
const editingConsultation = ref<Consultation | null>(null)

// ── Non-respondents: per-consultation count for open cards ──
const nonRespondentsCount = ref<Record<string, number>>({})
const remindingId = ref<string | null>(null)

// ── Inline vote state (per consultation ID) ────────────────────────
interface InlineVote {
  selections: string[]   // choice IDs currently checked
  comment: string
  submitting: boolean
  error: string
  unlocked: boolean      // true while the user is editing a previous vote
}
const inlineVotes = ref<Record<string, InlineVote>>({})

function getInline(c: Consultation): InlineVote {
  if (!inlineVotes.value[c.id]) {
    inlineVotes.value[c.id] = {
      selections: [...(c.my_choice_ids ?? [])],
      comment: '',
      submitting: false,
      error: '',
      unlocked: false,
    }
  }
  return inlineVotes.value[c.id]
}

function hasVoted(c: Consultation): boolean {
  return (c.my_choice_ids ?? []).length > 0
}

function isVoteLocked(c: Consultation): boolean {
  return hasVoted(c) && !getInline(c).unlocked
}

function getTotalVotes(c: Consultation): number {
  return consultationTotalVotes.value[c.id] ?? 0
}

function barWidth(c: Consultation, votes: number): number {
  // Only show filled bars if user has already voted OR consultation is closed
  if (!(hasVoted(c) || c.status === POLL_STATUS_CLOSED)) return 0
  const totalVotes = getTotalVotes(c)
  if (totalVotes === 0) return 0
  return Math.round(votes * 100 / totalVotes)
}

function unlockVote(c: Consultation) {
  getInline(c).unlocked = true
}

// ── Close modal ────────────────────────────────────────────────────
const closingConsultation = ref<Consultation | null>(null)

// ── Delete modal ───────────────────────────────────────────────────
const deletingConsultation = ref<Consultation | null>(null)

async function loadData() {
  const [building, listRes] = await Promise.all([
    useBuildingStore().fetch(),
    consultationsApi.list(),
  ])
  buildingName.value = building.name
  allConsultations.value = listRes.data.consultations ?? []

  // Non-respondents counters for open consultations (managers only)
  if (canManage.value) {
    const counts: Record<string, number> = {}
    await Promise.all(
      allConsultations.value
        .filter(c => c.status === POLL_STATUS_OPEN)
        .map(async (c) => {
          try {
            const res = await consultationsApi.nonRespondents(c.id)
            counts[c.id] = res.data.count
          } catch {
            counts[c.id] = 0
          }
        }),
    )
    nonRespondentsCount.value = counts
  }

  // Re-initialize inline vote state for existing consultations
  for (const c of allConsultations.value) {
    inlineVotes.value[c.id] = {
      selections: [...(c.my_choice_ids ?? [])],
      comment: inlineVotes.value[c.id]?.comment ?? '',
      submitting: false,
      error: '',
      unlocked: false,
    }
  }
}

const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)

// Clear inline error messages when switching tabs
watch(statusFilter, () => {
  for (const c of allConsultations.value) {
    if (inlineVotes.value[c.id]) {
      inlineVotes.value[c.id].error = ''
    }
  }
})

const consultations = computed(() =>
  allConsultations.value.filter(c => statusFilter.value === ALL_FILTER || c.status === statusFilter.value)
    .slice().sort((left, right) => right.updated_at.localeCompare(left.updated_at)),
)
const allCount = computed(() => allConsultations.value.length)
const openCount = computed(() => allConsultations.value.filter(c => c.status === POLL_STATUS_OPEN).length)
const closedCount = computed(() => allConsultations.value.filter(c => c.status === POLL_STATUS_CLOSED).length)
const myDrafts = computed(() =>
  allConsultations.value.filter(c => c.status === POLL_STATUS_DRAFT && c.created_by === authStore.user?.user_id),
)
const draftCount = computed(() =>
  canManage.value ? allConsultations.value.filter(c => c.status === POLL_STATUS_DRAFT).length : myDrafts.value.length,
)
// The draft tab is shown to managers/board members or anyone owning at least one draft.
const showDraftTab = computed(() => canManage.value || myDrafts.value.length > 0)
// A draft is editable/openable/deletable by a manager or by its creator.
function canManageDraft(c: Consultation): boolean {
  return canManage.value || c.created_by === authStore.user?.user_id
}
const consultationTotalVotes = computed(() => {
  const map: Record<string, number> = {}
  for (const c of allConsultations.value) {
    map[c.id] = c.choices.reduce((sum, choice) => sum + choice.votes, 0)
  }
  return map
})

const legalCardLinks = computed(() => [
  {
    label: t('consultations.legalCard.article17LinkLabel'),
    href: t('consultations.legalCard.article17LinkHref'),
  },
  {
    label: t('consultations.legalCard.article24LinkLabel'),
    href: t('consultations.legalCard.article24LinkHref'),
  },
])

function openCreateModal() {
  editingConsultation.value = null
  showCreateModal.value = true
}

function openEditModal(c: Consultation) {
  editingConsultation.value = c
  showCreateModal.value = true
}

async function openDraft(c: Consultation) {
  const ok = await confirm({
    title: t('consultations.confirmOpenTitle'),
    message: t('consultations.confirmOpenMessage'),
    variant: 'primary',
  })
  if (!ok) return
  try {
    await consultationsApi.openDraft(c.id)
    await reload()
  } catch (err) {
    pageError.value = toErrorMessage(err)
  }
}

async function remindNonRespondents(c: Consultation) {
  const count = nonRespondentsCount.value[c.id] ?? 0
  const ok = await confirm({
    title: t('consultations.confirmRemindTitle'),
    message: t('consultations.confirmRemindMessage', { count }),
    confirmLabel: t('common.remind'),
    variant: 'primary',
  })
  if (!ok) return
  remindingId.value = c.id
  try {
    await consultationsApi.remind(c.id)
    await reload()
  } catch (err) {
    pageError.value = toErrorMessage(err)
  } finally {
    remindingId.value = null
  }
}

async function submitVote(c: Consultation) {
  const iv = getInline(c)
  if (!iv.selections.length) return
  iv.submitting = true
  iv.error = ''
  try {
    await consultationsApi.respond(c.id, iv.selections, iv.comment || undefined)
    await reload()
  } catch (err) {
    iv.error = toErrorMessage(err)
    iv.submitting = false
  }
}

function openCloseModal(c: Consultation) {
  closingConsultation.value = c
}

function openDeleteModal(c: Consultation) {
  deletingConsultation.value = c
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight />
          <span>{{ t('nav.consultations') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canCreate"
          variant="primary"
          size="sm"
          :icon="CIconPlus"
          :title="t('consultations.create')"
          @click="openCreateModal"
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
      <template v-else>
        <header class="view-header">
          <CEyebrow>
            {{ t('page.eyebrows.consultations') }}
          </CEyebrow>
          <CHeading type="header">
            {{ t('nav.consultations') }}
          </CHeading>
          <CMetaItem variant="sub">
            {{ t('consultations.allCount', { count: allCount }) }}
          </CMetaItem>
        </header>

        <CAlert
          v-if="pageError"
          variant="error"
        >
          {{ pageError }}
        </CAlert>

        <CTabs
          v-model="statusFilter"
          variant="filter"
          :tabs="[
            { value: ALL_FILTER, label: t('consultations.filters.all'), count: allCount },
            { value: POLL_STATUS_OPEN, label: t('consultations.filters.open'), count: openCount },
            ...(showDraftTab ? [{ value: POLL_STATUS_DRAFT, label: t('consultations.filters.draft'), count: draftCount }] : []),
            { value: POLL_STATUS_CLOSED, label: t('consultations.filters.closed'), count: closedCount },
          ]"
        />
        <template v-if="!consultations.length">
          <CEmptyState
            :title="statusFilter === ALL_FILTER ? t('consultations.noAll') : statusFilter === POLL_STATUS_DRAFT ? t('consultations.noDraft') : statusFilter === POLL_STATUS_OPEN ? t('consultations.noOpen') : t('consultations.noClosed')"
            size="page"
          >
            <template 
              #actions
            >
              <CBtn
                v-if="canCreate"
                variant="primary"
                :icon="CIconPlus"
                :title="t('consultations.create')"
                @click="openCreateModal"
              />
            </template>
          </CEmptyState>
        </template>
        <template v-else>
          <div class="consultation-list">
            <CCard
              v-for="c in consultations"
              :key="c.id"
              :shadow="c.status !== POLL_STATUS_CLOSED"
              :class="{ 'outline outline-1 outline-outline-variant': c.status === POLL_STATUS_CLOSED }"
            >
              <CBadge
                :variant="c.status === POLL_STATUS_OPEN ? 'open' : c.status === POLL_STATUS_DRAFT ? 'pending' : 'closed'"
                class="status-badge"
                dot
              >
                {{ t(`consultations.statuses.${c.status}`) }}
              </CBadge>
  
              <CHeading 
                type="card"
                variant="title"
              >
                {{ c.title }}
              </CHeading>
  
              <p
                v-if="c.description"
              >
                {{ c.description }}
              </p>
  
              <!-- Choices + live result bars (CSS grid — 3 columns per choice row) -->
              <div class="choices-results">
                <div
                  v-for="choice in c.choices"
                  :key="choice.id"
                  class="choice-row"
                >
                  <!-- Column 1: checkbox (open) or plain label (closed/draft) -->
                  <CFormCheckbox
                    v-if="c.status === POLL_STATUS_OPEN"
                    v-model="getInline(c).selections"
                    :value="choice.id"
                    :label="choice.label"
                    :disabled="isVoteLocked(c)"
                    class="choice-label"
                    :class="{ 'choice-label--locked': isVoteLocked(c) }"
                  />
                  <span
                    v-else
                    class="choice-label"
                  >{{ choice.label }}</span>
  
                  <!-- Column 2: result bar -->
                  <div class="result-bar-bg">
                    <div
                      class="result-bar"
                      :class="{ 'result-bar--dim': !hasVoted(c) && c.status === POLL_STATUS_OPEN }"
                      :style="{ width: barWidth(c, choice.votes) + '%' }"
                    ></div>
                  </div>
  
                  <!-- Column 3: percentage -->
                  <span class="choice-percent font-mono text-xs text-end">
                    {{ hasVoted(c) || c.status === POLL_STATUS_CLOSED ? (() => { const total = getTotalVotes(c); return total > 0 ? Math.round(choice.votes * 100 / total) : 0 })() + '%' : '—' }}
                  </span>
                </div>
              </div>
  
              <!-- Response count -->
              <p
                v-if="hasVoted(c) || c.status === POLL_STATUS_CLOSED"
                class="body-xs response-count"
              >
                {{ t('consultations.responseCount', { count: c.total_responses }) }}
              </p>
  
              <!-- Inline vote form: comment + submit appear when ≥1 choice selected AND not locked -->
              <div
                v-if="c.status === POLL_STATUS_OPEN && getInline(c).selections.length > 0 && !isVoteLocked(c)"
                class="inline-vote-form"
              >
                <CFormTextarea
                  v-model="getInline(c).comment"
                  :rows="2"
                  :placeholder="t('consultations.respondModal.commentLabel')"
                />
                <div class="inline-vote-actions">
                  <p
                    v-if="getInline(c).error"
                    class="body-xs error-text"
                  >
                    {{ getInline(c).error }}
                  </p>
                  <CBtn
                    variant="primary"
                    size="sm"
                    :title="hasVoted(c) ? t('consultations.updateVote') : t('consultations.respondModal.submit')"
                    :loading="getInline(c).submitting"
                    @click="submitVote(c)"
                  />
                </div>
              </div>
  
              <!-- Locked state: show "Modifier" button when already voted -->
              <div
                v-if="c.status === POLL_STATUS_OPEN && isVoteLocked(c)"
                class="vote-locked-actions"
              >
                <CBtn
                  variant="ghost"
                  size="sm"
                  :title="t('consultations.modifyVote')"
                  @click="unlockVote(c)"
                />
              </div>
  
              <!-- Date hint -->
              <p class="body-xs footer-meta">
                {{ c.status === POLL_STATUS_DRAFT
                  ? t('consultations.draftedAt', { date: formatRelative(c.updated_at) })
                  : c.status === POLL_STATUS_OPEN
                    ? t('consultations.openedAt', { date: formatRelative(c.created_at) })
                    : t('consultations.closedAt', { date: formatDate(c.closed_at!) }) }}
              </p>

              <!-- Non-respondents info (open consultations, managers only) -->
              <p
                v-if="c.status === POLL_STATUS_OPEN && canManage"
                class="body-xs response-count remind-info"
              >
                {{ t('consultations.nonRespondents', { count: nonRespondentsCount[c.id] ?? 0 }) }}
                <template v-if="c.last_reminder_at">
                  · {{ t('consultations.lastReminder', { date: formatRelative(c.last_reminder_at) }) }}
                </template>
              </p>

              <!-- Draft actions: edit / open / delete -->
              <div
                v-if="c.status === POLL_STATUS_DRAFT && canManageDraft(c)"
                class="card-actions"
              >
                <CBtn
                  variant="ghost"
                  size="sm"
                  :icon="CIconEdit"
                  :title="t('common.edit')"
                  @click="openEditModal(c)"
                />
                <CBtn
                  variant="primary"
                  size="sm"
                  :icon="CIconCheckCircle"
                  :title="t('consultations.openAction')"
                  @click="openDraft(c)"
                />
                <CBtn
                  variant="danger"
                  size="sm"
                  :icon="CIconTrash"
                  :title="t('common.delete')"
                  @click="openDeleteModal(c)"
                />
              </div>
  
              <!-- Manager actions (open consultations): remind + close -->
              <div
                v-if="c.status === POLL_STATUS_OPEN && canManage"
                class="card-actions"
              >
                <CBtn
                  variant="ghost"
                  size="sm"
                  :icon="CIconBell"
                  :title="t('common.remind')"
                  :disabled="(nonRespondentsCount[c.id] ?? 0) === 0"
                  :loading="remindingId === c.id"
                  @click="remindNonRespondents(c)"
                />
                <CBtn
                  variant="primary"
                  size="sm"
                  :icon="CIconCheckCircle"
                  :title="t('consultations.close')"
                  @click="openCloseModal(c)"
                />
              </div>
  
              <!-- Manager delete action (closed consultations) -->
              <div
                v-if="c.status === POLL_STATUS_CLOSED && canManage"
                class="card-actions"
              >
                <CBtn
                  variant="danger"
                  size="sm"
                  :icon="CIconTrash"
                  :title="t('common.delete')"
                  @click="openDeleteModal(c)"
                />
              </div>
            </CCard>
          </div>
        </template>

        <CLegalCard>
          <CLegalNotice :links="legalCardLinks">
            <CLegalDescription
              :title="t('consultations.legalCard.title')"
              :summary="t('consultations.legalCard.summary')"
            />
          </CLegalNotice>
        </CLegalCard>
      </template>
    </div>

    <CreateConsultationModal
      v-model:open="showCreateModal"
      :consultation="editingConsultation"
      @created="reload"
      @updated="reload"
    />

    <CloseConsultationModal
      :open="!!closingConsultation"
      :consultation="closingConsultation"
      @update:open="v => !v && (closingConsultation = null)"
      @closed="reload"
    />

    <DeleteConsultationModal
      :open="!!deletingConsultation"
      :consultation="deletingConsultation"
      @update:open="v => !v && (deletingConsultation = null)"
      @deleted="reload"
    />
  </div>
</template>

<style scoped>
.consultation-list { display: flex; flex-direction: column; gap: var(--space-3); }

.status-badge { margin-bottom: var(--space-2); }
.footer-meta { font-size: var(--text-2xs); color: var(--color-on-surface-variant); margin-top: var(--space-3); }
.card-actions { display: flex; gap: 6px; margin-top: var(--space-3); }
.remind-info { margin-top: var(--space-2); }
.response-count { color: var(--color-on-surface-variant); margin-top: var(--space-1); }

/* ── Choices + result bars (CSS grid) ─────────────────────────── */
/* Desktop: 3-col grid — label | bar | result%
   .choice-row uses display:contents so children participate in the parent grid.
   Mobile: .choices-results becomes a flex column; each .choice-row is its own
   2-row mini-grid — label+result% on row 1, bar full-width on row 2. */
.choices-results {
  display: grid;
  grid-template-columns: fit-content(50%) 1fr var(--space-9);
  align-items: center;
  column-gap: var(--space-2);
  row-gap: var(--space-2);
  margin-bottom: var(--space-2);
}
.choice-row { 
  width: 100%;
  display: contents; 
}

@media (max-width: 768px) {
  .choices-results {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .choice-row {
    display: grid;
    grid-template-columns: 1fr auto;
    grid-template-rows: auto auto;
    column-gap: var(--space-2);
    row-gap: var(--space-1);
    align-items: center;
  }
  .choice-row .choice-label  { grid-column: 1; grid-row: 1; }
  .choice-row .choice-percent { grid-column: 2; grid-row: 1; }
  .choice-row .result-bar-bg  { grid-column: 1 / -1; grid-row: 2; }
}

.choice-label {
  font-size: var(--text-sm);
  overflow-wrap: break-word;
  word-break: break-word;
  white-space: normal;
  min-width: 0;
}

.choice-label--locked {
  opacity: 0.6;
}

.result-bar-bg {
  background: var(--color-surface-container);
  border-radius: var(--radius-full);
  height: 10px;
  overflow: hidden;
}
.result-bar {
  background: var(--color-primary);
  height: 100%;
  border-radius: var(--radius-full);
  transition: width 0.3s ease;
}
.result-bar--dim { background: var(--color-outline-variant); }

/* ── Inline vote form ──────────────────────────────────────────── */
.vote-locked-actions {
  margin-top: var(--space-2);
}
.inline-vote-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  margin-top: var(--space-3);
  padding-top: var(--space-3);
}
.inline-vote-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-2);
}
.error-text { color: var(--color-error); flex: 1; }

</style>
