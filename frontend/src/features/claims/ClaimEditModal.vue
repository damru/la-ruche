<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import { claimsApi, documentsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useFormatters } from '@/composables/useFormatters'
import { extractFileExtension } from '@/utils/files'
import type { Claim, ClaimType, Document, Unit } from '@/types/domain'

const props = defineProps<{
  open: boolean
  claim: Claim | null
  units: Unit[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'updated': []
  'delete-requested': []
  'document-count-changed': [claimId: string, delta: number]
}>()

const { t } = useI18n()
const { formatDate, formatFileSize } = useFormatters()
const feedback = useFormFeedback()
const documentActionFeedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()

const CLAIM_TYPES: ClaimType[] = ['current', 'natural_disaster', 'theft']

type ClaimEditorForm = {
  title: string
  description: string
  claim_type: ClaimType
  declared_at: string
  published_at: string
  unit_id: string
}

type ClaimUpdatePayload = {
  title: string
  description?: string | null
  claim_type: string
  declared_at: string
  published_at?: string | null
  status: string
  unit_id?: string | null
  transmitted_at?: string | null
  reimbursement_date?: string | null
  reimbursement_amount?: string | null
}

function defaultForm(): ClaimEditorForm {
  return {
    title: '',
    description: '',
    claim_type: 'current',
    declared_at: new Date().toISOString().slice(0, 10),
    published_at: '',
    unit_id: '',
  }
}

function formFromClaim(claim: Claim): ClaimEditorForm {
  return {
    title: claim.title,
    description: claim.description ?? '',
    claim_type: claim.claim_type,
    declared_at: claim.declared_at,
    published_at: claim.published_at ?? '',
    unit_id: claim.unit_id ?? '',
  }
}

const editingClaim = ref(false)
const uploadingEditDocuments = ref(false)
const form = ref<ClaimEditorForm>(defaultForm())
const editDocuments = ref<Document[]>([])
const loadingEditDocuments = ref(false)
const selectedEditDocuments = ref<File[]>([])
const editDocumentInputRef = ref<HTMLInputElement | null>(null)
const editDocumentsDragOver = ref(false)
const detachingDocumentId = ref<string | null>(null)
const pendingDocumentAction = ref<Document | null>(null)

const claimTypeOptions = computed(() =>
  CLAIM_TYPES.map(type => ({ value: type, label: t(`claims.types.${type}`) })),
)

const unitOptions = computed(() => [
  { value: '', label: t('claims.commonAreas') },
  ...props.units.map(unit => ({
    value: unit.id,
    label: t('claims.unitLabel', { number: unit.unit_number }),
  })),
])

watch(() => props.open, async (open) => {
  if (!open) { forceClose(); return }
  if (!props.claim) return
  form.value = formFromClaim(props.claim)
  selectedEditDocuments.value = []
  editDocumentsDragOver.value = false
  editDocuments.value = []
  feedback.reset()
  await loadEditDocuments(props.claim.id)
})

function normalizePublishedAt(claimType: ClaimType, publishedAt: string): string | null {
  if (claimType !== 'natural_disaster') return null
  return publishedAt || null
}

function buildClaimUpdatePayload(claim: Claim, overrides: Partial<ClaimUpdatePayload>): ClaimUpdatePayload {
  return {
    title: claim.title,
    description: claim.description ?? null,
    claim_type: claim.claim_type,
    declared_at: claim.declared_at,
    published_at: claim.published_at ?? null,
    status: claim.status,
    unit_id: claim.unit_id ?? null,
    transmitted_at: claim.transmitted_at ?? null,
    reimbursement_date: claim.reimbursement_date ?? null,
    reimbursement_amount: claim.reimbursement_amount != null ? String(claim.reimbursement_amount) : null,
    ...overrides,
  }
}

async function loadEditDocuments(claimId: string) {
  loadingEditDocuments.value = true
  try {
    const response = await claimsApi.listDocuments(claimId)
    editDocuments.value = response.data
  } catch (err: unknown) {
    feedback.setError(toErrorMessage(err, t('claims.documentsLoadError')))
  } finally {
    loadingEditDocuments.value = false
  }
}

function onEditDocumentChange(event: Event) {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files ?? [])
  if (files.length > 0) selectedEditDocuments.value = [...selectedEditDocuments.value, ...files]
  if (editDocumentInputRef.value) editDocumentInputRef.value.value = ''
}

function onEditDocumentDrop(event: DragEvent) {
  editDocumentsDragOver.value = false
  const files = Array.from(event.dataTransfer?.files ?? [])
  if (files.length > 0) selectedEditDocuments.value = [...selectedEditDocuments.value, ...files]
}

function removeEditDocument(index: number) {
  selectedEditDocuments.value = selectedEditDocuments.value.filter((_, i) => i !== index)
}

async function downloadClaimDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch (err: unknown) {
    feedback.setError(toErrorMessage(err, t('common.error')))
  }
}

function openDocumentActionModal(document: Document) {
  pendingDocumentAction.value = document
  documentActionFeedback.reset()
}

function closeDocumentActionModal() {
  if (detachingDocumentId.value) return
  pendingDocumentAction.value = null
  documentActionFeedback.reset()
}

async function handleDocumentAction(deleteDocument: boolean) {
  if (!props.claim || !pendingDocumentAction.value) return

  detachingDocumentId.value = pendingDocumentAction.value.id

  try {
    if (deleteDocument) {
      await documentsApi.delete(pendingDocumentAction.value.id)
    } else {
      await claimsApi.detachDocument(props.claim.id, pendingDocumentAction.value.id)
    }
    editDocuments.value = editDocuments.value.filter(d => d.id !== pendingDocumentAction.value?.id)
    emit('document-count-changed', props.claim.id, -1)
    pendingDocumentAction.value = null
  } catch (err: unknown) {
    documentActionFeedback.setError(
      toErrorMessage(err, deleteDocument ? t('claims.documentsDeleteError') : t('claims.documentsDetachError')),
    )
  } finally {
    detachingDocumentId.value = null
  }
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

async function attachSelectedDocuments(claimId: string, documents: File[]): Promise<string[]> {
  const failed: string[] = []
  for (const document of documents) {
    try {
      await uploadDocument(claimId, document)
    } catch {
      failed.push(document.name)
    }
  }
  return failed
}

async function handleSubmit() {
  if (feedback.status.value === 'success') {
    forceClose()
    return
  }

  if (!props.claim) return

  editingClaim.value = true
  feedback.reset()

  try {
    await claimsApi.update(
      props.claim.id,
      buildClaimUpdatePayload(props.claim, {
        title: form.value.title.trim(),
        description: form.value.description.trim() || null,
        claim_type: form.value.claim_type,
        declared_at: form.value.declared_at,
        published_at: normalizePublishedAt(form.value.claim_type, form.value.published_at),
        unit_id: form.value.unit_id || null,
      }),
    )

    if (selectedEditDocuments.value.length > 0) {
      uploadingEditDocuments.value = true
      const failed = await attachSelectedDocuments(props.claim.id, selectedEditDocuments.value)
      uploadingEditDocuments.value = false

      if (failed.length > 0) {
        feedback.setError(`${t('claims.documentUploadError')}: ${failed.join(', ')}`)
        setTimeout(() => {
          emit('updated')
          emit('update:open', false)
        }, 2200)
        return
      }
    }

    feedback.setSuccess(t('claims.updateSuccess'))
    emit('updated')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err: unknown) {
    feedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    editingClaim.value = false
    uploadingEditDocuments.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('claims.editTitle')"
    size="lg"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="claim-edit"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="form.title"
        :label="t('claims.fields.title')"
        required
      />
      <CFormSelect
        v-model="form.claim_type"
        :label="t('claims.fields.claimType')"
        :options="claimTypeOptions"
      />

      <div class="form-row">
        <CFormInput
          v-model="form.declared_at"
          :label="t('claims.fields.declaredAt')"
          type="date"
          required
        />
        <CFormInput
          v-if="form.claim_type === 'natural_disaster'"
          v-model="form.published_at"
          :label="t('claims.fields.publishedAt')"
          type="date"
          required
          :hint="t('claims.fields.publishedAtHint')"
        />
      </div>

      <CFormSelect
        v-model="form.unit_id"
        :label="t('claims.fields.unit')"
        :options="unitOptions"
      />
      <CFormTextarea
        v-model="form.description"
        :label="t('claims.fields.description')"
        :rows="3"
      />

      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('claims.fields.documents') }}</label>
        <input
          ref="editDocumentInputRef"
          type="file"
          multiple
          style="display:none"
          @change="onEditDocumentChange"
        />
        <div
          :class="['dropzone', { 'dropzone--over': editDocumentsDragOver }]"
          role="button"
          tabindex="0"
          @click="editDocumentInputRef?.click()"
          @keydown.enter.prevent="editDocumentInputRef?.click()"
          @keydown.space.prevent="editDocumentInputRef?.click()"
          @dragenter.prevent="editDocumentsDragOver = true"
          @dragover.prevent="editDocumentsDragOver = true"
          @dragleave.prevent="editDocumentsDragOver = false"
          @drop.prevent="onEditDocumentDrop"
        >
          <CIconUpload class="dropzone__icon" />
          <span class="body-xs dropzone__hint">{{ t('claims.editDocumentHint') }}</span>
        </div>

        <div
          v-if="selectedEditDocuments.length > 0"
          class="claim-preview-strip"
        >
          <div
            v-for="(document, index) in selectedEditDocuments"
            :key="`${document.name}-${index}`"
            class="claim-preview-item"
          >
            <span class="claim-preview-name">{{ document.name }}</span>
            <button
              type="button"
              class="claim-remove"
              :title="t('claims.removeDocument')"
              :aria-label="t('claims.removeDocument')"
              @click="removeEditDocument(index)"
            >
              ×
            </button>
          </div>
        </div>

        <div class="claim-documents-section">
          <p class="text-xs text-on-surface-variant claim-documents-section__hint">
            {{ t('claims.fields.attachedDocuments') }}
          </p>
          <CPageLoader v-if="loadingEditDocuments" />
          <div
            v-else-if="editDocuments.length === 0"
            class="claim-documents-empty"
          >
            {{ t('claims.noAttachedDocuments') }}
          </div>
          <div
            v-else
            class="claim-documents-list"
          >
            <div
              v-for="document in editDocuments"
              :key="document.id"
              class="claim-documents-row"
            >
              <div class="claim-documents-meta">
                <div class="claim-documents-name">
                  <CIconFile class="icon-sm" />
                  <span>{{ document.original_name }}</span>
                </div>
                <div class="claim-documents-details">
                  <span>{{ formatFileSize(document.size_bytes) }}</span>
                  <span>·</span>
                  <span>{{ formatDate(document.created_at) }}</span>
                </div>
              </div>
              <div class="claim-documents-actions">
                <CBtn
                  variant="ghost"
                  size="sm"
                  :title="t('common.download')"
                  @click="downloadClaimDocument(document.id)"
                />
                <CBtn
                  variant="danger"
                  size="sm"
                  :title="t('claims.detachDocument')"
                  :loading="detachingDocumentId === document.id"
                  @click="openDocumentActionModal(document)"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </CForm>

    <template #footer>
      <CFormActions
        :status="feedback.status.value"
        :message="feedback.message.value"
      >
        <CBtn
          v-if="claim"
          variant="danger"
          :icon="CIconTrash"
          :title="t('common.delete')"
          class="mr-auto"
          @click="$emit('delete-requested')"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="editingClaim || uploadingEditDocuments"
          :closing="feedback.status.value === 'success'"
          type="submit"
          form="claim-edit"
        />
      </CFormActions>
    </template>
  </CModal>

  <CDocumentLinkActionModal
    :open="!!pendingDocumentAction"
    :title="t('claims.documentsDetachTitle')"
    :description="t('claims.documentsDetachMessage', { name: pendingDocumentAction?.original_name ?? '', title: claim?.title ?? '' })"
    :warning="t('claims.documentsDeleteWarning', { name: pendingDocumentAction?.original_name ?? '' })"
    :loading="!!detachingDocumentId"
    :status="documentActionFeedback.status.value"
    :message="documentActionFeedback.message.value"
    @update:open="value => !value && closeDocumentActionModal()"
    @unlink="handleDocumentAction(false)"
    @unlink-and-delete="handleDocumentAction(true)"
  />
</template>

<style scoped>
.form-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

@media (max-width: 720px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .claim-documents-row {
    flex-direction: column;
    align-items: flex-start;
  }
}

.claim-preview-strip {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: var(--space-2);
}

.claim-preview-item {
  position: relative;
  display: inline-flex;
  align-items: center;
  min-height: 44px;
  max-width: 100%;
  padding: 0 var(--space-4) 0 var(--space-3);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-outline-variant);
  background: var(--color-surface-container-low);
}

.claim-preview-name {
  max-width: 220px;
  overflow: clip;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-on-surface-variant);
  font-size: var(--text-sm);
}

.claim-remove {
  position: absolute;
  top: -6px;
  right: -6px;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--color-error);
  color: #fff;
  border: none;
  cursor: pointer;
  font-size: var(--text-md);
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  min-height: unset;
  min-width: unset;
}

.claim-remove:hover {
  background: var(--color-error-dark, var(--color-error));
  opacity: 0.85;
}

.claim-documents-section {
  margin-top: var(--space-3);
}

.claim-documents-section__hint {
  margin-bottom: var(--space-2);
}

.claim-documents-empty {
  padding: var(--space-3);
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-low);
  color: var(--color-on-surface-variant);
}

.claim-documents-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.claim-documents-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-low);
}

.claim-documents-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.claim-documents-name {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
  font-weight: var(--fw-medium);
}

.claim-documents-name span {
  overflow: clip;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.claim-documents-details {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

.claim-documents-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}
</style>
