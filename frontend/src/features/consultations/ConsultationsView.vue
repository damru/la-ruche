<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback, SUCCESS_AUTO_CLEAR_MS } from '@/composables/useFormFeedback'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCard from '@/components/molecules/CCard.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import { buildingApi, consultationsApi } from '@/api'
import type { Consultation, ConsultationResults } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()

const buildingName = ref('')
const allConsultations = ref<Consultation[]>([])
const statusFilter = ref<'open' | 'closed'>('open')
const resultsCache = ref<Record<string, ConsultationResults>>({})
const pageError = ref('')

// ── Create modal ───────────────────────────────────────────────────
const showCreateModal = ref(false)
const createFeedback = useFormFeedback()
const saving = ref(false)
const defaultCreateForm = () => ({
  title: '',
  description: '',
  choices: ['', ''],
})
const createForm = ref(defaultCreateForm())

// ── Respond modal ──────────────────────────────────────────────────
const respondingTo = ref<Consultation | null>(null)
const respondFeedback = useFormFeedback()
const responding = ref(false)
const selectedChoice = ref('')
const respondComment = ref('')

// ── Close modal ────────────────────────────────────────────────────
const closingConsultation = ref<Consultation | null>(null)
const closeFeedback = useFormFeedback()
const closing = ref(false)

async function loadData() {
  const [buildingRes, listRes] = await Promise.all([
    buildingApi.get(),
    consultationsApi.list(),
  ])
  buildingName.value = buildingRes.data.name
  allConsultations.value = listRes.data.consultations ?? []

  // Pre-fetch results for closed consultations
  await Promise.all(
    allConsultations.value
      .filter(c => c.status === 'closed' && !resultsCache.value[c.id])
      .map(async (c) => {
        const res = await consultationsApi.results(c.id)
        resultsCache.value[c.id] = res.data.results
      }),
  )
}

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
onMounted(reload)

const consultations = computed(() =>
  allConsultations.value.filter(c =>
    statusFilter.value === 'open' ? c.status === 'open' : c.status === 'closed',
  ),
)
const openCount = computed(() => allConsultations.value.filter(c => c.status === 'open').length)

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
    setTimeout(() => { showCreateModal.value = false; reload() }, SUCCESS_AUTO_CLEAR_MS)
  } catch (err) {
    createFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    saving.value = false
  }
}

function openRespondModal(c: Consultation) {
  respondingTo.value = c
  respondFeedback.reset()
  selectedChoice.value = ''
  respondComment.value = ''
}

async function submitResponse() {
  if (!respondingTo.value || !selectedChoice.value) return
  responding.value = true
  respondFeedback.reset()
  try {
    await consultationsApi.respond(respondingTo.value.id, selectedChoice.value, respondComment.value || undefined)
    respondFeedback.setSuccess(t('common.saved'))
    setTimeout(() => { respondingTo.value = null; reload() }, SUCCESS_AUTO_CLEAR_MS)
  } catch (err) {
    respondFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    responding.value = false
  }
}

function openCloseModal(c: Consultation) {
  closingConsultation.value = c
  closeFeedback.reset()
}

async function confirmClose() {
  if (!closingConsultation.value) return
  closing.value = true
  closeFeedback.reset()
  try {
    await consultationsApi.close(closingConsultation.value.id)
    closeFeedback.setSuccess(t('common.saved'))
    setTimeout(async () => { closingConsultation.value = null; await reload() }, SUCCESS_AUTO_CLEAR_MS)
  } catch (err) {
    closeFeedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    closing.value = false
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName }}
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('consultations.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          @click="openCreateModal"
        >
          <CIconPlus style="width:16px;height:16px" />
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
        <div class="view-header">
          <div>
            <h1 class="display-md">
              {{ t('consultations.title') }}
            </h1>
            <p
              class="body-md"
              style="color:var(--color-on-surface-variant)"
            >
              {{ t('consultations.openCount', openCount) }}
            </p>
          </div>
        </div>

        <CAlert
          v-if="pageError"
          variant="error"
        >
          {{ pageError }}
        </CAlert>

        <div class="tabs">
          <button
            v-for="item in [
              { key: 'open', label: t('consultations.filters.open') },
              { key: 'closed', label: t('consultations.filters.closed') },
            ]"
            :key="item.key"
            :class="['tab', { 'tab--active': statusFilter === item.key }]"
            @click="statusFilter = item.key as typeof statusFilter"
          >
            {{ item.label }}
          </button>
        </div>

        <div class="consultation-list">
          <CCard
            v-for="c in consultations"
            :key="c.id"
          >
            <!-- Status badge top-left -->
            <CBadge
              :variant="c.status === 'open' ? 'open' : 'closed'"
              class="status-badge"
            >
              {{ t(`consultations.statuses.${c.status}`) }}
            </CBadge>

            <!-- Title prominent, under status -->
            <h3 class="consultation-title title-md">
              {{ c.title }}
            </h3>

            <p
              v-if="c.description"
              class="body-sm desc"
            >
              {{ c.description }}
            </p>

            <!-- Open choices pills -->
            <div
              v-if="c.status === 'open' && c.choices"
              class="open-choices"
            >
              <span
                v-for="ch in c.choices"
                :key="ch.id"
                class="choice-pill"
              >{{ ch.label }}</span>
            </div>

            <!-- Closed results -->
            <div
              v-if="c.status === 'closed' && resultsCache[c.id]"
              class="results"
            >
              <div
                v-for="choice in resultsCache[c.id].choices"
                :key="choice.id"
                class="result-row"
              >
                <span class="result-label">{{ choice.label }}</span>
                <div class="result-bar-bg">
                  <div
                    class="result-bar"
                    :style="{ width: choice.pct + '%' }"
                  ></div>
                </div>
                <span class="result-pct">{{ choice.pct }}%</span>
              </div>
              <p
                class="body-xs"
                style="color:var(--color-on-surface-variant);margin-top:4px"
              >
                {{ t('consultations.responseCount', { count: resultsCache[c.id].total }) }}
              </p>
            </div>

            <!-- Date hint -->
            <p class="body-xs footer-meta">
              {{ c.status === 'open'
                ? t('consultations.openedAt', { date: formatRelative(c.created_at) })
                : t('consultations.closedAt', { date: formatDate(c.closed_at!) }) }}
            </p>

            <!-- Actions at bottom -->
            <div
              v-if="c.status === 'open'"
              class="card-actions"
            >
              <CBtn
                variant="primary"
                size="sm"
                @click="openRespondModal(c)"
              >
                {{ t('consultations.respond') }}
              </CBtn>
              <CBtn
                variant="ghost"
                size="sm"
                @click="openCloseModal(c)"
              >
                {{ t('consultations.close') }}
              </CBtn>
            </div>
          </CCard>

          <div
            v-if="!consultations.length"
            class="empty-state"
          >
            {{ statusFilter === 'open' ? t('consultations.noOpen') : t('consultations.noClosed') }}
          </div>
        </div>

        <!-- Legal info — bottom of page, CCard pattern (cf. Registry) -->
        <CCard>
          <p class="body-sm dim">
            ⚖️ <strong>{{ t('consultations.legalNote') }}</strong> — {{ t('consultations.informalNote') }}
            <br /><span class="body-xs">{{ t('consultations.informalNoteDetail') }}</span>
          </p>
        </CCard>
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
        class="form-grid"
        @submit.prevent="saveConsultation"
      >
        <div class="form-field">
          <label class="form-label">{{ t('consultations.fields.title') }} *</label>
          <input
            v-model="createForm.title"
            type="text"
            class="form-input"
            required
          />
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('consultations.fields.description') }}</label>
          <textarea
            v-model="createForm.description"
            class="form-input"
            rows="2"
          ></textarea>
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('consultations.fields.choices') }} *</label>
          <div
            v-for="(_, idx) in createForm.choices"
            :key="idx"
            class="choice-input-row"
          >
            <input
              v-model="createForm.choices[idx]"
              type="text"
              class="form-input"
              :placeholder="`Option ${idx + 1}`"
            />
            <button
              v-if="createForm.choices.length > 2"
              type="button"
              class="icon-btn icon-btn--danger"
              @click="removeChoice(idx)"
            >
              <CIconX
                width="14"
                height="14"
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
              width="14"
              height="14"
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
          @click="saveConsultation"
        >
          {{ t('common.create') }}
        </CBtn>
      </template>
    </CModal>

    <!-- Respond modal -->
    <CModal
      :open="!!respondingTo"
      :title="t('consultations.respondModal.title')"
      size="md"
      @update:open="v => !v && (respondingTo = null)"
    >
      <div
        v-if="respondingTo"
        class="form-grid"
      >
        <p
          class="body-md"
          style="font-weight:var(--fw-medium)"
        >
          {{ respondingTo.title }}
        </p>
        <div class="form-field">
          <label class="form-label">{{ t('consultations.respondModal.chooseOption') }} *</label>
          <div class="choice-options">
            <label
              v-for="choice in respondingTo.choices"
              :key="choice.id"
              class="checkbox-label"
            >
              <input
                v-model="selectedChoice"
                type="radio"
                :value="choice.id"
                name="choice"
              />
              {{ choice.label }}
            </label>
          </div>
        </div>
        <div class="form-field">
          <label class="form-label">{{ t('consultations.respondModal.commentLabel') }}</label>
          <textarea
            v-model="respondComment"
            class="form-input"
            rows="2"
          ></textarea>
        </div>
      </div>
      <template #footer>
        <CFormFeedback
          :status="respondFeedback.status.value"
          :message="respondFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="respondingTo = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="responding"
          :disabled="!selectedChoice"
          @click="submitResponse"
        >
          {{ t('consultations.respondModal.submit') }}
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
      <p class="body-sm">
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
          @click="confirmClose"
        >
          {{ t('consultations.closeAction') }}
        </CBtn>
      </template>
    </CModal>
  </div>
</template>

<style scoped>
.consultation-list { display: flex; flex-direction: column; gap: 12px; }

.status-badge { margin-bottom: var(--space-2); }
.consultation-title { margin-bottom: var(--space-2); }
.desc { color: var(--color-on-surface-variant); margin-bottom: var(--space-3); }
.footer-meta { font-size: var(--text-2xs); color: var(--color-on-surface-variant); margin-top: var(--space-3); }
.card-actions { display: flex; gap: 6px; margin-top: var(--space-3); }

.results { display: flex; flex-direction: column; gap: 8px; margin-bottom: var(--space-2); }
/* Label min space, bar max space, pct fixed */
.result-row { display: grid; grid-template-columns: auto 1fr 36px; gap: 8px; align-items: center; min-width: 0; }
.result-label { font-size: var(--text-xs); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 120px; }
.result-bar-bg { background: var(--color-surface-container); border-radius: var(--radius-full); height: 10px; overflow: hidden; }
.result-bar { background: var(--color-primary); height: 100%; border-radius: var(--radius-full); }
.result-pct { font-size: var(--text-xs); font-weight: var(--fw-semibold); text-align: right; }

.open-choices { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: var(--space-2); }
.choice-pill { padding: 3px 10px; background: var(--color-surface-container); border-radius: var(--radius-full); font-size: var(--text-xs); }

.choice-options { display: flex; flex-direction: column; gap: 8px; }
.choice-input-row { display: flex; align-items: center; gap: 8px; }
.choice-input-row .form-input { flex: 1; }

.form-grid { display: flex; flex-direction: column; gap: var(--space-4); }
.form-field { display: flex; flex-direction: column; gap: var(--space-2); }
.form-label { font-size: var(--text-sm); font-weight: var(--fw-medium); color: var(--color-on-surface-variant); }
.form-input { padding: var(--space-2) var(--space-3); border: 1px solid var(--color-outline); border-radius: var(--radius-md); font-size: var(--text-sm); background: var(--color-surface); color: var(--color-on-surface); }
.form-input:focus { outline: none; border-color: var(--color-primary); }
.empty-state { padding: var(--space-8); text-align: center; font-size: var(--text-md); color: var(--color-on-surface-variant); }
.dim { color: var(--color-on-surface-variant); }
</style>
