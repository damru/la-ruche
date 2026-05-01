<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { usePageLoader, toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useRole } from '@/composables/useRole'
import CBadge from '@/components/atoms/CBadge.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CTooltip from '@/components/atoms/CTooltip.vue'
import CPageError from '@/components/molecules/CPageError.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CCard from '@/components/molecules/CCard.vue'
import CModal from '@/components/molecules/CModal.vue'
import CTopbar from '@/components/organisms/CTopbar.vue'
import CIconAlertTriangle from '@/components/atoms/CIconAlertTriangle.svg'
import CIconClock from '@/components/atoms/CIconClock.svg'
import CIconFile from '@/components/atoms/CIconFile.svg'
import CIconHome from '@/components/atoms/CIconHome.svg'
import CIconShield from '@/components/atoms/CIconShield.svg'
import CIconUpload from '@/components/atoms/CIconUpload.svg'
import { buildingApi, claimsApi, documentsApi, unitsApi } from '@/api'
import { extractFileExtension } from '@/utils/files'
import type { Claim, ClaimStatus, ClaimType, Unit } from '@/types/domain'

const { t } = useI18n()
const { formatDate, formatRelative } = useFormatters()
const { hasAnyRole } = useRole()

const CLAIM_TYPES: ClaimType[] = ['current', 'natural_disaster', 'theft']
const CLAIM_STATUSES: ClaimStatus[] = ['open', 'in_progress', 'closed']

const buildingName = ref('')
const allClaims = ref<Claim[]>([])
const allUnits = ref<Unit[]>([])
const filter = ref<'all' | ClaimStatus>('all')
const canManage = computed(() => hasAnyRole(['manager', 'board_member']))

const showCreateModal = ref(false)
const createFeedback = useFormFeedback()
const savingClaim = ref(false)
const uploadingDocuments = ref(false)
const selectedDocuments = ref<File[]>([])
const documentInputRef = ref<HTMLInputElement | null>(null)
const documentsDragOver = ref(false)
const createForm = ref(defaultCreateForm())
const { schedule: scheduleCreateClose, forceExpire: forceCreateClose } = useAutoClose()
watch(showCreateModal, (open) => { if (!open) forceCreateClose() })

const claimToUpdate = ref<Claim | null>(null)
const updateFeedback = useFormFeedback()
const updatingClaim = ref(false)
const updateForm = ref(defaultUpdateForm())
const { schedule: scheduleUpdateClose, forceExpire: forceUpdateClose } = useAutoClose()
watch(claimToUpdate, (claim) => { if (!claim) forceUpdateClose() })

function defaultCreateForm() {
  return {
    title: '',
    description: '',
    claim_type: 'current' as ClaimType,
    declared_at: new Date().toISOString().slice(0, 10),
    published_at: '',
    unit_id: '',
  }
}

function defaultUpdateForm() {
  return {
    title: '',
    description: '',
    claim_type: 'current' as ClaimType,
    declared_at: '',
    published_at: '',
    status: 'open' as ClaimStatus,
    unit_id: '',
    transmitted_at: '',
  }
}

async function loadData() {
  const [buildingResponse, claimsResponse, unitsResponse] = await Promise.all([
    buildingApi.get(),
    claimsApi.list(),
    unitsApi.list(),
  ])

  buildingName.value = buildingResponse.data.name
  allClaims.value = claimsResponse.data
  allUnits.value = unitsResponse.data
}

const { loading, error, reload } = usePageLoader(loadData, t('common.loadError'))
onMounted(reload)

const unitsById = computed(() =>
  new Map(allUnits.value.map(unit => [unit.id, unit])),
)

const claims = computed(() =>
  (filter.value === 'all'
    ? allClaims.value
    : allClaims.value.filter(claim => claim.status === filter.value))
    .slice()
    .sort((left, right) => right.created_at.localeCompare(left.created_at)),
)

const activeClaimsCount = computed(() =>
  allClaims.value.filter(claim => claim.status !== 'closed').length,
)

function statusVariant(status: ClaimStatus): 'open' | 'pending' | 'closed' | 'default' {
  if (status === 'open') return 'pending'
  if (status === 'in_progress') return 'open'
  if (status === 'closed') return 'closed'
  return 'default'
}

function typeIcon(type: ClaimType) {
  const icons = {
    current: CIconShield,
    natural_disaster: CIconHome,
    theft: CIconAlertTriangle,
  }

  return icons[type] ?? CIconShield
}

function typeTone(type: ClaimType): string {
  const tones = {
    current: 'tone--default',
    natural_disaster: 'tone--warn',
    theft: 'tone--err',
  }

  return tones[type] ?? 'tone--default'
}

function unitLabel(unitId?: string): string {
  if (!unitId) return t('claims.commonAreas')
  return t('claims.unitLabel', { number: unitsById.value.get(unitId)?.unit_number ?? '—' })
}

function deadlineVariant(claim: Claim): 'valid' | 'pending' | 'err' | 'default' {
  if (claim.transmitted_at) return claim.is_transmitted_late ? 'err' : 'valid'
  if (claim.is_deadline_missed) return 'err'
  if (claim.is_deadline_approaching) return 'pending'
  return 'default'
}

function deadlineLabel(claim: Claim): string {
  if (claim.transmitted_at) {
    return t(
      claim.is_transmitted_late
        ? 'claims.deadline.transmittedLate'
        : 'claims.deadline.transmittedOnTime',
      {
        date: formatDate(claim.transmitted_at),
        deadline: formatDate(claim.legal_deadline_at),
      },
    )
  }

  if (claim.is_deadline_missed) {
    return t('claims.deadline.missed', { date: formatDate(claim.legal_deadline_at) })
  }

  if (claim.days_remaining === 0) {
    return t('claims.deadline.today', { date: formatDate(claim.legal_deadline_at) })
  }

  if (typeof claim.days_remaining === 'number') {
    return t('claims.deadline.remaining', {
      days: claim.days_remaining,
      date: formatDate(claim.legal_deadline_at),
    })
  }

  return t('claims.deadline.reference', { date: formatDate(claim.legal_deadline_at) })
}

function claimLegalTooltip(claim: Claim) {
  let tooltipKey: 'current' | 'theft' | 'naturalDisaster'
  switch (claim.claim_type) {
    case 'natural_disaster':
      tooltipKey = 'naturalDisaster'
      break
    case 'theft':
      tooltipKey = 'theft'
      break
    default:
      tooltipKey = 'current'
  }

  return {
    content: t(`claims.legalTooltips.${tooltipKey}.summary`),
    linkLabel: t(`claims.legalTooltips.${tooltipKey}.linkLabel`),
    linkHref: t(`claims.legalTooltips.${tooltipKey}.linkHref`),
  }
}

function openCreateModal() {
  createForm.value = defaultCreateForm()
  selectedDocuments.value = []
  documentsDragOver.value = false
  createFeedback.reset()
  showCreateModal.value = true
}

function openUpdateModal(claim: Claim) {
  claimToUpdate.value = claim
  updateForm.value = {
    title: claim.title,
    description: claim.description ?? '',
    claim_type: claim.claim_type,
    declared_at: claim.declared_at,
    published_at: claim.published_at ?? '',
    status: claim.status,
    unit_id: claim.unit_id ?? '',
    transmitted_at: claim.transmitted_at ?? '',
  }
  updateFeedback.reset()
}

function onDocumentChange(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files) {
    selectedDocuments.value = [...selectedDocuments.value, ...Array.from(input.files)]
  }
  if (documentInputRef.value) documentInputRef.value.value = ''
}

function onDocumentDrop(event: DragEvent) {
  documentsDragOver.value = false
  const files = Array.from(event.dataTransfer?.files ?? [])
  selectedDocuments.value = [...selectedDocuments.value, ...files]
}

function removeDocument(index: number) {
  selectedDocuments.value = selectedDocuments.value.filter((_, currentIndex) => currentIndex !== index)
}

async function uploadDocument(claimId: string, file: File): Promise<void> {
  const extension = extractFileExtension(file.name)
  const mimeType = file.type || 'application/octet-stream'
  const uploadUrlResponse = await documentsApi.getUploadUrl(extension, mimeType, file.size)
  const { url, key, bucket } = uploadUrlResponse.data
  await documentsApi.uploadToS3(file, url, mimeType)
  const documentResponse = await documentsApi.confirmUpload({
    key,
    bucket,
    original_name: file.name,
    mime_type: mimeType,
    size_bytes: file.size,
    visible_roles: ['manager', 'board_member', 'co_owner', 'tenant'],
  })
  await claimsApi.attachDocument(claimId, documentResponse.data.id)
}

async function submitCreate() {
  if (createFeedback.status.value === 'success') {
    forceCreateClose()
    return
  }

  savingClaim.value = true
  createFeedback.reset()

  try {
    const response = await claimsApi.create({
      title: createForm.value.title.trim(),
      description: createForm.value.description.trim() || undefined,
      claim_type: createForm.value.claim_type,
      declared_at: createForm.value.declared_at,
      published_at: createForm.value.claim_type === 'natural_disaster'
        ? (createForm.value.published_at || undefined)
        : undefined,
      unit_id: createForm.value.unit_id || undefined,
    })

    if (selectedDocuments.value.length > 0) {
      uploadingDocuments.value = true
      const failedDocuments: string[] = []

      for (const document of selectedDocuments.value) {
        try {
          await uploadDocument(response.data.id, document)
        } catch {
          failedDocuments.push(document.name)
        }
      }

      uploadingDocuments.value = false

      if (failedDocuments.length > 0) {
        createFeedback.setError(
          `${t('claims.documentUploadError')}: ${failedDocuments.join(', ')}`,
        )
        setTimeout(() => {
          showCreateModal.value = false
          reload()
        }, 2200)
        return
      }
    }

    createFeedback.setSuccess(t('claims.createSuccess'))
    scheduleCreateClose(() => {
      showCreateModal.value = false
      reload()
    })
  } catch (currentError: unknown) {
    createFeedback.setError(toErrorMessage(currentError, t('common.error')))
  } finally {
    savingClaim.value = false
    uploadingDocuments.value = false
  }
}

async function submitUpdate() {
  if (updateFeedback.status.value === 'success') {
    forceUpdateClose()
    return
  }

  if (!claimToUpdate.value) return

  updatingClaim.value = true
  updateFeedback.reset()

  try {
    await claimsApi.update(claimToUpdate.value.id, {
      title: updateForm.value.title.trim(),
      description: updateForm.value.description.trim() || null,
      claim_type: updateForm.value.claim_type,
      declared_at: updateForm.value.declared_at,
      published_at: updateForm.value.claim_type === 'natural_disaster'
        ? (updateForm.value.published_at || null)
        : null,
      status: updateForm.value.status,
      unit_id: updateForm.value.unit_id || null,
      transmitted_at: updateForm.value.transmitted_at || null,
    })

    updateFeedback.setSuccess(t('claims.updateSuccess'))
    scheduleUpdateClose(() => {
      claimToUpdate.value = null
      reload()
    })
  } catch (currentError: unknown) {
    updateFeedback.setError(toErrorMessage(currentError, t('common.error')))
  } finally {
    updatingClaim.value = false
  }
}
</script>

<template>
  <div class="view-page">
    <CTopbar>
      <template #left>
        <div class="topbar-breadcrumb">
          {{ buildingName || '…' }}
          <CIconChevronRight
            width="14"
            height="14"
          />
          <span>{{ t('claims.title') }}</span>
        </div>
      </template>
      <template #right>
        <CBtn
          variant="primary"
          size="sm"
          @click="openCreateModal"
        >
          <CIconPlus style="width:16px;height:16px" />
          <span class="btn-label">{{ t('claims.create') }}</span>
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
              {{ t('claims.title') }}
            </h1>
            <p
              class="body-md"
              style="color:var(--color-on-surface-variant)"
            >
              {{ t('claims.activeCount', activeClaimsCount) }}
            </p>
          </div>
        </div>

        <div class="tabs">
          <button
            v-for="option in [
              { key: 'all', label: t('common.all') },
              { key: 'open', label: t('claims.filters.open') },
              { key: 'in_progress', label: t('claims.filters.inProgress') },
              { key: 'closed', label: t('claims.filters.closed') },
            ]"
            :key="option.key"
            :class="['tab', { 'tab--active': filter === option.key }]"
            @click="filter = option.key as typeof filter"
          >
            {{ option.label }}
          </button>
        </div>

        <div class="claims-list">
          <CCard
            v-for="claim in claims"
            :key="claim.id"
            :shadow="claim.status !== 'closed'"
            :class="['claim-card', { 'card--terminated': claim.status === 'closed' }]"
          >
            <div class="claim-card__header">
              <div class="claim-card__identity">
                <div :class="['claim-card__icon', typeTone(claim.claim_type)]">
                  <component
                    :is="typeIcon(claim.claim_type)"
                    width="18"
                    height="18"
                  />
                </div>
                <div class="claim-card__title-wrap">
                  <div class="claim-card__badges">
                    <CBadge
                      :variant="statusVariant(claim.status)"
                      dot
                    >
                      {{ t(`claims.statuses.${claim.status}`) }}
                    </CBadge>
                    <CBadge variant="default">
                      {{ t(`claims.types.${claim.claim_type}`) }}
                    </CBadge>
                    <CBadge
                      :variant="deadlineVariant(claim)"
                      :dot="deadlineVariant(claim) !== 'default'"
                    >
                      {{ deadlineLabel(claim) }}
                    </CBadge>
                  </div>
                  <h3 class="title-md">
                    {{ claim.title }}
                  </h3>
                  <p class="body-sm claim-card__meta">
                    {{ unitLabel(claim.unit_id) }} · {{ t('claims.declaredOn', { date: formatDate(claim.declared_at) }) }}
                    <template v-if="claim.published_at">
                      · {{ t('claims.publishedOn', { date: formatDate(claim.published_at) }) }}
                    </template>
                  </p>
                </div>
              </div>

              <div class="claim-card__actions">
                <span class="claim-card__opened">
                  {{ t('claims.openedAt', { date: formatRelative(claim.created_at) }) }}
                </span>
                <CBtn
                  v-if="canManage"
                  variant="ghost"
                  size="sm"
                  @click="openUpdateModal(claim)"
                >
                  {{ t('claims.updateTracking') }}
                </CBtn>
              </div>
            </div>

            <p
              v-if="claim.description"
              class="body-sm claim-card__description"
            >
              {{ claim.description }}
            </p>

            <div class="claim-card__footer">
              <span class="claim-card__detail">
                <CIconClock
                  width="14"
                  height="14"
                />
                <span>{{ t('claims.deadline.reference', { date: formatDate(claim.legal_deadline_at) }) }}</span>
                <CTooltip v-bind="claimLegalTooltip(claim)" />
              </span>
              <span class="claim-card__detail">
                <CIconFile
                  width="14"
                  height="14"
                />
                {{ t('claims.documentsCount', claim.document_count) }}
              </span>
            </div>
          </CCard>

          <div
            v-if="claims.length === 0"
            class="empty-state"
          >
            {{ t('claims.empty') }}
          </div>
        </div>
      </template>
    </div>

    <CModal
      :open="showCreateModal"
      :title="t('claims.create')"
      size="lg"
      :closable="!savingClaim && !uploadingDocuments"
      @update:open="showCreateModal = $event"
    >
      <form
        class="form-grid"
        @submit.prevent="submitCreate"
      >
        <div class="form-field">
          <label class="form-label">{{ t('claims.fields.title') }}</label>
          <input
            v-model="createForm.title"
            type="text"
            class="form-input"
            required
          />
        </div>

        <div class="form-field">
          <label class="form-label">{{ t('claims.fields.claimType') }}</label>
          <select
            v-model="createForm.claim_type"
            class="form-input"
          >
            <option
              v-for="claimType in CLAIM_TYPES"
              :key="claimType"
              :value="claimType"
            >
              {{ t(`claims.types.${claimType}`) }}
            </option>
          </select>
        </div>

        <div class="form-row">
          <div class="form-field">
            <label class="form-label">{{ t('claims.fields.declaredAt') }}</label>
            <input
              v-model="createForm.declared_at"
              type="date"
              class="form-input"
              required
            />
          </div>

          <div
            v-if="createForm.claim_type === 'natural_disaster'"
            class="form-field"
          >
            <label class="form-label">{{ t('claims.fields.publishedAt') }}</label>
            <input
              v-model="createForm.published_at"
              type="date"
              class="form-input"
              required
            />
            <p class="field-hint">
              {{ t('claims.fields.publishedAtHint') }}
            </p>
          </div>
        </div>

        <div class="form-field">
          <label class="form-label">{{ t('claims.fields.unit') }}</label>
          <select
            v-model="createForm.unit_id"
            class="form-input"
          >
            <option value="">
              {{ t('claims.commonAreas') }}
            </option>
            <option
              v-for="unit in allUnits"
              :key="unit.id"
              :value="unit.id"
            >
              {{ t('claims.unitLabel', { number: unit.unit_number }) }}
            </option>
          </select>
        </div>

        <div class="form-field">
          <label class="form-label">{{ t('claims.fields.description') }}</label>
          <textarea
            v-model="createForm.description"
            class="form-input"
            rows="3"
          ></textarea>
        </div>

        <div class="form-field">
          <label class="form-label">{{ t('claims.fields.documents') }}</label>
          <input
            ref="documentInputRef"
            type="file"
            multiple
            style="display:none"
            @change="onDocumentChange"
          />
          <div
            :class="['document-dropzone', { 'document-dropzone--over': documentsDragOver }]"
            role="button"
            tabindex="0"
            @click="documentInputRef?.click()"
            @keydown.enter.prevent="documentInputRef?.click()"
            @keydown.space.prevent="documentInputRef?.click()"
            @dragenter.prevent="documentsDragOver = true"
            @dragover.prevent="documentsDragOver = true"
            @dragleave.prevent="documentsDragOver = false"
            @drop.prevent="onDocumentDrop"
          >
            <CIconUpload
              width="16"
              height="16"
            />
            <span>{{ t('claims.documentHint') }}</span>
          </div>

          <ul
            v-if="selectedDocuments.length > 0"
            class="document-list"
          >
            <li
              v-for="(document, index) in selectedDocuments"
              :key="`${document.name}-${index}`"
              class="document-list__item"
            >
              <span>{{ document.name }}</span>
              <button
                type="button"
                class="document-list__remove"
                @click="removeDocument(index)"
              >
                {{ t('claims.removeDocument') }}
              </button>
            </li>
          </ul>
        </div>
      </form>

      <template #footer>
        <CFormFeedback
          class="modal-footer-feedback"
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
          :loading="savingClaim || uploadingDocuments"
          :closing="createFeedback.status.value === 'success'"
          @click="submitCreate"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>

    <CModal
      :open="!!claimToUpdate"
      :title="t('claims.updateTracking')"
      size="md"
      @update:open="value => !value && (claimToUpdate = null)"
    >
      <form
        class="form-grid"
        @submit.prevent="submitUpdate"
      >
        <div class="form-field">
          <label class="form-label">{{ t('claims.fields.status') }}</label>
          <select
            v-model="updateForm.status"
            class="form-input"
          >
            <option
              v-for="status in CLAIM_STATUSES"
              :key="status"
              :value="status"
            >
              {{ t(`claims.statuses.${status}`) }}
            </option>
          </select>
        </div>

        <div class="form-field">
          <label class="form-label">{{ t('claims.fields.transmittedAt') }}</label>
          <input
            v-model="updateForm.transmitted_at"
            type="date"
            class="form-input"
          />
          <p class="field-hint">
            {{ t('claims.fields.transmittedAtHint') }}
          </p>
        </div>
      </form>

      <template #footer>
        <CFormFeedback
          class="modal-footer-feedback"
          :status="updateFeedback.status.value"
          :message="updateFeedback.message.value"
        />
        <CBtn
          variant="ghost"
          @click="claimToUpdate = null"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="updatingClaim"
          :closing="updateFeedback.status.value === 'success'"
          @click="submitUpdate"
        >
          {{ t('common.save') }}
        </CBtn>
      </template>
    </CModal>
  </div>
</template>

<style scoped>
.claims-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.claim-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.claim-card__header {
  display: flex;
  justify-content: space-between;
  gap: var(--space-4);
}

.claim-card__identity {
  display: flex;
  gap: var(--space-3);
  min-width: 0;
}

.claim-card__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  flex-shrink: 0;
  background: var(--tone-bg);
  color: var(--tone-fg);
}

.claim-card__title-wrap {
  min-width: 0;
}

.claim-card__badges {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 8px;
}

.claim-card__meta,
.claim-card__opened,
.claim-card__description,
.claim-card__detail {
  color: var(--color-on-surface-variant);
}

.claim-card__description {
  margin: 0;
}

.claim-card__actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: var(--space-2);
  flex-shrink: 0;
}

.claim-card__opened {
  font-size: var(--text-xs);
}

.claim-card__footer {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
}

.claim-card__detail {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: var(--text-sm);
}

.document-dropzone {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
  color: var(--color-on-surface-variant);
  cursor: pointer;
}

.document-dropzone--over {
  background: var(--color-secondary-container);
  color: var(--color-on-secondary-container);
}

.document-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: var(--space-3) 0 0;
  padding: 0;
  list-style: none;
}

.document-list__item {
  display: flex;
  justify-content: space-between;
  gap: var(--space-3);
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-container-low);
}

.document-list__remove {
  color: var(--color-error);
  font-size: var(--text-sm);
}

.modal-footer-feedback {
  margin-right: auto;
  min-width: 0;
}

.form-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.empty-state {
  padding: var(--space-8);
  text-align: center;
  color: var(--color-on-surface-variant);
}

@media (max-width: 720px) {
  .claim-card__header {
    flex-direction: column;
  }

  .claim-card__actions {
    align-items: flex-start;
  }

  .form-row {
    grid-template-columns: 1fr;
  }
}
</style>
