<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useAuthStore } from '@/stores/auth'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CLegalDescription from '@/components/atoms/CLegalDescription.vue'
import CLegalNotice from '@/components/molecules/CLegalNotice.vue'
import CLegalCard from '@/components/molecules/CLegalCard.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/organisms/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CTabs from '@/components/atoms/CTabs.vue'
import { buildingApi, consultationsApi } from '@/api'
import type { Consultation } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()
const authStore = useAuthStore()

// Managers and board members can create/close consultations.
const canManage = computed(() => authStore.hasAnyRole(['manager', 'board_member']))

const buildingName = ref('')
const allConsultations = ref<Consultation[]>([])
const statusFilter = ref<'open' | 'closed'>('open')
const pageError = ref('')

// ── Create modal ───────────────────────────────────────────────────
const showCreateModal = ref(false)
const createFeedback = useFormFeedback()
const saving = ref(false)
const { schedule: scheduleCreateClose, forceExpire: forceCreateClose } = useAutoClose()
watch(showCreateModal, (open) => { if (!open) forceCreateClose() })
const defaultCreateForm = () => ({
  title: '',
  description: '',
  choices: ['', ''],
})
const createForm = ref(defaultCreateForm())

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
  if (!(hasVoted(c) || c.status === 'closed')) return 0
  const totalVotes = getTotalVotes(c)
  if (totalVotes === 0) return 0
  return Math.round(votes * 100 / totalVotes)
}

function unlockVote(c: Consultation) {
  getInline(c).unlocked = true
}

// ── Close modal ────────────────────────────────────────────────────
const closingConsultation = ref<Consultation | null>(null)
const closeFeedback = useFormFeedback()
const closing = ref(false)
const { schedule: scheduleCloseConsultation, forceExpire: forceCloseConsultation } = useAutoClose()
watch(closingConsultation, (c) => { if (!c) forceCloseConsultation() })

// ── Delete modal ───────────────────────────────────────────────────
const deletingConsultation = ref<Consultation | null>(null)
const deleteFeedback = useFormFeedback()
const deleting = ref(false)
const { schedule: scheduleDeleteConsultation, forceExpire: forceDeleteConsultation } = useAutoClose()
watch(deletingConsultation, (c) => { if (!c) forceDeleteConsultation() })

async function loadData() {
  const [buildingRes, listRes] = await Promise.all([
    buildingApi.get(),
    consultationsApi.list(),
  ])
  buildingName.value = buildingRes.data.name
  allConsultations.value = listRes.data.consultations ?? []

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
  allConsultations.value.filter(c =>
    statusFilter.value === 'open' ? c.status === 'open' : c.status === 'closed',
  ).slice().sort((left, right) => right.updated_at.localeCompare(left.updated_at)),
)
const openCount = computed(() => allConsultations.value.filter(c => c.status === 'open').length)
const closedCount = computed(() => allConsultations.value.filter(c => c.status === 'closed').length)
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
  createForm.value = defaultCreateForm()
  createFeedback.reset()
  showCreateModal.value = true
}

function addChoice() { createForm.value.choices.push('') }
function removeChoice(idx: number) {
  if (createForm.value.choices.length > 2) createForm.value.choices.splice(idx, 1)
}

async function saveConsultation() {
  if (createFeedback.status.value === 'success') { forceCreateClose(); return }
  if (!createForm.value.title.trim() || createForm.value.choices.filter(c => c.trim()).length < 2) return
  saving.value = true
  createFeedback.reset()
  try {
    await consultationsApi.create({
      title: createForm.value.title,
      description: createForm.value.description || undefined,
      visible_to_roles: ['manager', 'board_member', 'co_owner', 'tenant'],
      choices: createForm.value.choices.filter(c => c.trim()),
    })
    createFeedback.setSuccess(t('common.saved'))
    scheduleCreateClose(() => { showCreateModal.value = false; reload() })
  } catch (err) {
    createFeedback.setError(toErrorMessage(err))
  } finally {
    saving.value = false
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
  closeFeedback.reset()
}

async function confirmClose() {
  if (closeFeedback.status.value === 'success') { forceCloseConsultation(); return }
  if (!closingConsultation.value) return
  closing.value = true
  closeFeedback.reset()
  try {
    await consultationsApi.close(closingConsultation.value.id)
    closeFeedback.setSuccess(t('consultations.closeSuccess'))
    scheduleCloseConsultation(async () => { closingConsultation.value = null; await reload() })
  } catch (err) {
    closeFeedback.setError(toErrorMessage(err))
  } finally {
    closing.value = false
  }
}

function openDeleteModal(c: Consultation) {
  deletingConsultation.value = c
  deleteFeedback.reset()
}

async function confirmDelete() {
  if (deleteFeedback.status.value === 'success') { forceDeleteConsultation(); return }
  if (!deletingConsultation.value) return
  deleting.value = true
  deleteFeedback.reset()
  try {
    await consultationsApi.delete(deletingConsultation.value.id)
    deleteFeedback.setSuccess(t('consultations.deleteSuccess'))
    scheduleDeleteConsultation(async () => { deletingConsultation.value = null; await reload() })
  } catch (err) {
    deleteFeedback.setError(toErrorMessage(err))
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight />
          <span>{{ t('consultations.title') }}</span>
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
          <span class="btn-label">{{ t('consultations.create') }}</span>
        </CBtn>
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
            {{ t('consultations.title') }}
          </CHeading>
          <p
            class="body-md"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('consultations.openCount', openCount) }}
          </p>
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
            { value: 'open', label: t('consultations.filters.open'), count: openCount },
            { value: 'closed', label: t('consultations.filters.closed'), count: closedCount },
          ]"
        />
        <template v-if="!consultations.length">
          <CEmptyState
            :title="statusFilter === 'open' ? t('consultations.noOpen') : t('consultations.noClosed')"
            size="page"
          >
            <template 
              #actions
            >
              <CBtn
                v-if="canManage"
                variant="primary"
                @click="openCreateModal"
              >
                <CIconPlus />
                {{ t('consultations.create') }}
              </CBtn>
            </template>
          </CEmptyState>
        </template>
        <template v-else>
          <div class="consultation-list">
            <CCard
              v-for="c in consultations"
              :key="c.id"
              :shadow="c.status !== 'closed'"
              :class="{ 'outline outline-1 outline-outline-variant': c.status === 'closed' }"
            >
              <CBadge
                :variant="c.status === 'open' ? 'open' : 'closed'"
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
                  <!-- Column 1: checkbox (open) or plain label (closed) -->
                  <CFormCheckbox
                    v-if="c.status === 'open'"
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
                      :class="{ 'result-bar--dim': !hasVoted(c) && c.status === 'open' }"
                      :style="{ width: barWidth(c, choice.votes) + '%' }"
                    ></div>
                  </div>
  
                  <!-- Column 3: percentage -->
                  <span class="choice-percent font-mono text-xs text-end">
                    {{ hasVoted(c) || c.status === 'closed' ? (() => { const total = getTotalVotes(c); return total > 0 ? Math.round(choice.votes * 100 / total) : 0 })() + '%' : '—' }}
                  </span>
                </div>
              </div>
  
              <!-- Response count -->
              <p
                v-if="hasVoted(c) || c.status === 'closed'"
                class="body-xs response-count"
              >
                {{ t('consultations.responseCount', { count: c.total_responses }) }}
              </p>
  
              <!-- Inline vote form: comment + submit appear when ≥1 choice selected AND not locked -->
              <div
                v-if="c.status === 'open' && getInline(c).selections.length > 0 && !isVoteLocked(c)"
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
                    :loading="getInline(c).submitting"
                    @click="submitVote(c)"
                  >
                    {{ hasVoted(c) ? t('consultations.updateVote') : t('consultations.respondModal.submit') }}
                  </CBtn>
                </div>
              </div>
  
              <!-- Locked state: show "Modifier" button when already voted -->
              <div
                v-if="c.status === 'open' && isVoteLocked(c)"
                class="vote-locked-actions"
              >
                <CBtn
                  variant="ghost"
                  size="sm"
                  @click="unlockVote(c)"
                >
                  {{ t('consultations.modifyVote') }}
                </CBtn>
              </div>
  
              <!-- Date hint -->
              <p class="body-xs footer-meta">
                {{ c.status === 'open'
                  ? t('consultations.openedAt', { date: formatRelative(c.created_at) })
                  : t('consultations.closedAt', { date: formatDate(c.closed_at!) }) }}
              </p>
  
              <!-- Manager close action (open consultations) -->
              <div
                v-if="c.status === 'open' && canManage"
                class="card-actions"
              >
                <CBtn
                  variant="primary"
                  size="sm"
                  @click="openCloseModal(c)"
                >
                  <CIconCheckCircle />
                  {{ t('consultations.close') }}
                </CBtn>
              </div>
  
              <!-- Manager delete action (closed consultations) -->
              <div
                v-if="c.status === 'closed' && canManage"
                class="card-actions"
              >
                <CBtn
                  variant="danger"
                  size="sm"
                  @click="openDeleteModal(c)"
                >
                  <CIconTrash />
                  {{ t('common.delete') }}
                </CBtn>
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

    <!-- Create modal -->
    <CModal
      :open="showCreateModal"
      :title="t('consultations.create')"
      size="lg"
      @update:open="showCreateModal = $event"
    >
      <form
        class="flex flex-col gap-4"
        @submit.prevent="saveConsultation"
      >
        <CFormInput
          v-model="createForm.title"
          :label="t('consultations.fields.title')"
          required
        />
        <CFormTextarea
          v-model="createForm.description"
          :label="t('consultations.fields.description')"
          :rows="2"
        />
        <div class="flex flex-col gap-1">
          <label class="text-sm font-medium text-on-surface-variant">{{ t('consultations.fields.choices') }} *</label>
          <div
            v-for="(_, idx) in createForm.choices"
            :key="idx"
            class="choice-input-row"
          >
            <CFormInput
              v-model="createForm.choices[idx]"
              :placeholder="`Option ${idx + 1}`"
              class="grow"
            />
            <button
              v-if="createForm.choices.length > 2"
              type="button"
              class="icon-btn icon-btn--danger"
              @click="removeChoice(idx)"
            >
              <CIconX
                
                class="icon-xs"
              />
            </button>
          </div>
          <CBtn
            variant="ghost"
            size="sm"
            type="button"
            @click="addChoice"
          >
            <CIconPlus
              
              class="icon-xs"
            /> {{ t('consultations.fields.addChoice') }}
          </CBtn>
        </div>
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
          @click="saveConsultation"
        >
          {{ t('common.create') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Close consultation modal -->
    <CModal
      :open="!!closingConsultation"
      :title="t('consultations.confirmCloseTitle')"
      size="sm"
      @update:open="v => !v && (closingConsultation = null)"
    >
      <p class="body-xs">
        {{ t('consultations.confirmCloseMessage') }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="closeFeedback.status.value"
          :message="closeFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="closingConsultation = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="closing"
          :closing="closeFeedback.status.value === 'success'"
          @click="confirmClose"
        >
          {{ t('consultations.closeAction') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Delete consultation modal -->
    <CModal
      :open="!!deletingConsultation"
      :title="t('consultations.confirmDeleteTitle')"
      size="sm"
      @update:open="v => !v && (deletingConsultation = null)"
    >
      <p class="body-xs">
        {{ t('consultations.confirmDeleteMessage', { title: deletingConsultation?.title ?? '' }) }}
      </p>
      <template #footer>
        <CFormFeedback
          :status="deleteFeedback.status.value"
          :message="deleteFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="deletingConsultation = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="deleting"
          :closing="deleteFeedback.status.value === 'success'"
          @click="confirmDelete"
        >
          {{ t('common.delete') }}
        </CBtn>
      </template>
    </CModal>
  </div>
</template>

<style scoped>
.consultation-list { display: flex; flex-direction: column; gap: 12px; }

.status-badge { margin-bottom: var(--space-2); }
.footer-meta { font-size: var(--text-2xs); color: var(--color-on-surface-variant); margin-top: var(--space-3); }
.card-actions { display: flex; gap: 6px; margin-top: var(--space-3); }
.response-count { color: var(--color-on-surface-variant); margin-top: var(--space-1); }

/* ── Choices + result bars (CSS grid) ─────────────────────────── */
/* Desktop: 3-col grid — label | bar | result%
   .choice-row uses display:contents so children participate in the parent grid.
   Mobile: .choices-results becomes a flex column; each .choice-row is its own
   2-row mini-grid — label+result% on row 1, bar full-width on row 2. */
.choices-results {
  display: grid;
  grid-template-columns: fit-content(50%) 1fr 36px;
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

.choice-label.checkbox-label {
  display: inline-flex;
  align-items: flex-start;
  gap: var(--space-2);
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
  border-top: 1px solid var(--color-outline-variant);
}
.inline-vote-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-2);
}
.error-text { color: var(--color-error); flex: 1; }

/* ── Create-form styles ────────────────────────────────────────── */
.choice-input-row { display: flex; align-items: center; gap: 8px; }
.choice-input-row .form-input { flex: 1; }

.dim { color: var(--color-on-surface-variant); }

</style>
