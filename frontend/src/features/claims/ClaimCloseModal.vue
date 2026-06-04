<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { claimsApi, documentsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { extractFileExtension } from '@/utils/files'
import type { Claim } from '@/types/domain'

const props = defineProps<{
  open: boolean
  claim: Claim | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'closed': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()

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

function todayIso(): string {
  return new Date().toISOString().slice(0, 10)
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

function parseOptionalAmount(value: string): string | null {
  const normalized = value.trim().replace(',', '.')
  if (!normalized) return null
  const amount = Number(normalized)
  return Number.isFinite(amount) && amount > 0 ? normalized : null
}

const closingClaim = ref(false)
const uploading = ref(false)
const form = ref({ transmitted_at: todayIso(), reimbursement_date: '', reimbursement_amount: '' })
const selectedDocuments = ref<File[]>([])
const documentInputRef = ref<HTMLInputElement | null>(null)
const dragOver = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  if (!props.claim) return
  form.value = {
    transmitted_at: props.claim.transmitted_at ?? todayIso(),
    reimbursement_date: props.claim.reimbursement_date ?? '',
    reimbursement_amount: props.claim.reimbursement_amount != null ? String(props.claim.reimbursement_amount) : '',
  }
  selectedDocuments.value = []
  dragOver.value = false
  feedback.reset()
})

function onDocumentChange(event: Event) {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files ?? [])
  if (files.length > 0) selectedDocuments.value = [...selectedDocuments.value, ...files]
  if (documentInputRef.value) documentInputRef.value.value = ''
}

function onDocumentDrop(event: DragEvent) {
  dragOver.value = false
  const files = Array.from(event.dataTransfer?.files ?? [])
  if (files.length > 0) selectedDocuments.value = [...selectedDocuments.value, ...files]
}

function removeDocument(index: number) {
  selectedDocuments.value = selectedDocuments.value.filter((_, i) => i !== index)
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
  if (!form.value.transmitted_at) {
    feedback.setError(t('claims.transmittedAtRequired'))
    return
  }

  const reimbursementAmountInput = form.value.reimbursement_amount.trim()
  const reimbursementAmount = parseOptionalAmount(reimbursementAmountInput)
  if (reimbursementAmountInput && reimbursementAmount == null) {
    feedback.setError(t('claims.invalidReimbursementAmount'))
    return
  }

  closingClaim.value = true
  feedback.reset()

  try {
    await claimsApi.update(
      props.claim.id,
      buildClaimUpdatePayload(props.claim, {
        status: 'closed',
        transmitted_at: form.value.transmitted_at,
        reimbursement_date: form.value.reimbursement_date || null,
        reimbursement_amount: reimbursementAmount,
      }),
    )

    if (selectedDocuments.value.length > 0) {
      uploading.value = true
      const failed = await attachSelectedDocuments(props.claim.id, selectedDocuments.value)
      uploading.value = false

      if (failed.length > 0) {
        feedback.setError(`${t('claims.closeDocumentUploadError')}: ${failed.join(', ')}`)
        setTimeout(() => {
          emit('closed')
          emit('update:open', false)
        }, 2200)
        return
      }
    }

    feedback.setSuccess(t('claims.closeSuccess'))
    emit('closed')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err: unknown) {
    feedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    closingClaim.value = false
    uploading.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('claims.closeTitle')"
    size="lg"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="claim-close"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="form.transmitted_at"
        :label="t('claims.fields.transmittedAt')"
        type="date"
        required
        :hint="t('claims.fields.transmittedAtHint')"
      />

      <div class="form-row">
        <CFormInput
          v-model="form.reimbursement_date"
          :label="t('claims.fields.reimbursementDate')"
          type="date"
        />
        <CFormInput
          v-model="form.reimbursement_amount"
          :label="t('claims.fields.reimbursementAmount')"
          type="number"
          :min="0.01"
          :step="0.01"
          inputmode="decimal"
          :hint="t('claims.fields.reimbursementAmountHint')"
        />
      </div>

      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('claims.fields.closeDocuments') }}</label>
        <input
          ref="documentInputRef"
          type="file"
          multiple
          style="display:none"
          @change="onDocumentChange"
        />
        <div
          :class="['dropzone', { 'dropzone--over': dragOver }]"
          role="button"
          tabindex="0"
          @click="documentInputRef?.click()"
          @keydown.enter.prevent="documentInputRef?.click()"
          @keydown.space.prevent="documentInputRef?.click()"
          @dragenter.prevent="dragOver = true"
          @dragover.prevent="dragOver = true"
          @dragleave.prevent="dragOver = false"
          @drop.prevent="onDocumentDrop"
        >
          <CIconUpload class="dropzone__icon" />
          <span class="body-xs dropzone__hint">{{ t('claims.closeDocumentHint') }}</span>
        </div>

        <div
          v-if="selectedDocuments.length > 0"
          class="claim-preview-strip"
        >
          <div
            v-for="(document, index) in selectedDocuments"
            :key="`${document.name}-${index}`"
            class="claim-preview-item"
          >
            <span class="claim-preview-name">{{ document.name }}</span>
            <button
              type="button"
              class="claim-remove"
              :title="t('claims.removeDocument')"
              :aria-label="t('claims.removeDocument')"
              @click="removeDocument(index)"
            >
              ×
            </button>
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
          variant="ghost"
          @click="$emit('update:open', false)"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="primary"
          :loading="closingClaim || uploading"
          :closing="feedback.status.value === 'success'"
          type="submit"
          form="claim-close"
        >
          {{ t('claims.close') }}
        </CBtn>
      </CFormActions>
    </template>
  </CModal>
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
</style>
