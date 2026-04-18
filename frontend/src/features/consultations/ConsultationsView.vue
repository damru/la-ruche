<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useAuthStore } from '@/stores/auth'
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
import CreateConsultationModal from './CreateConsultationModal.vue'
import CloseConsultationModal from './CloseConsultationModal.vue'
import DeleteConsultationModal from './DeleteConsultationModal.vue'
import { consultationsApi } from '@/api'
import { useBuildingStore } from '@/stores/building'
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

// ── Delete modal ───────────────────────────────────────────────────
const deletingConsultation = ref<Consultation | null>(null)

async function loadData() {
  const [building, listRes] = await Promise.all([
    useBuildingStore().fetch(),
    consultationsApi.list(),
  ])
  buildingName.value = building.name
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
  showCreateModal.value = true
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
          <span>{{ t('consultations.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          v-if="canManage"
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
            {{ t('consultations.title') }}
          </CHeading>
          <CMetaItem variant="sub">
            {{ t('consultations.openCount', { count: openCount }) }}
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
                    :title="hasVoted(c) ? t('consultations.updateVote') : t('consultations.respondModal.submit')"
                    :loading="getInline(c).submitting"
                    @click="submitVote(c)"
                  />
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
                  :title="t('consultations.modifyVote')"
                  @click="unlockVote(c)"
                />
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
                  :icon="CIconCheckCircle"
                  :title="t('consultations.close')"
                  @click="openCloseModal(c)"
                />
              </div>
  
              <!-- Manager delete action (closed consultations) -->
              <div
                v-if="c.status === 'closed' && canManage"
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
      @created="reload"
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
