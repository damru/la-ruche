<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { claimsApi, documentsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { extractFileExtension } from '@/utils/files'
import type { ClaimType, Unit } from '@/types/domain'

const props = defineProps<{
  open: boolean
  units: Unit[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
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

function todayIso(): string {
  return new Date().toISOString().slice(0, 10)
}

function defaultForm(): ClaimEditorForm {
  return {
    title: '',
    description: '',
    claim_type: 'current',
    declared_at: todayIso(),
    published_at: '',
    unit_id: '',
  }
}

const saving = ref(false)
const uploading = ref(false)
const form = ref<ClaimEditorForm>(defaultForm())
const selectedDocuments = ref<File[]>([])
const documentInputRef = ref<HTMLInputElement | null>(null)
const dragOver = ref(false)

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

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = defaultForm()
  selectedDocuments.value = []
  dragOver.value = false
  feedback.reset()
})

function normalizePublishedAt(claimType: ClaimType, publishedAt: string): string | null {
  if (claimType !== 'natural_disaster') return null
  return publishedAt || null
}

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

  saving.value = true
  feedback.reset()

  try {
    const response = await claimsApi.create({
      title: form.value.title.trim(),
      description: form.value.description.trim() || undefined,
      claim_type: form.value.claim_type,
      declared_at: form.value.declared_at,
      published_at: normalizePublishedAt(form.value.claim_type, form.value.published_at) ?? undefined,
      unit_id: form.value.unit_id || undefined,
    })

    if (selectedDocuments.value.length > 0) {
      uploading.value = true
      const failed = await attachSelectedDocuments(response.data.id, selectedDocuments.value)
      uploading.value = false

      if (failed.length > 0) {
        feedback.setError(`${t('claims.documentUploadError')}: ${failed.join(', ')}`)
        setTimeout(() => {
          emit('created')
          emit('update:open', false)
        }, 2200)
        return
      }
    }

    feedback.setSuccess(t('claims.createSuccess'))
    emit('created')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err: unknown) {
    feedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    saving.value = false
    uploading.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('claims.create')"
    size="lg"
    :closable="!saving && !uploading"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="claim-create"
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
          <span class="body-xs dropzone__hint">{{ t('claims.documentHint') }}</span>
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
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="saving || uploading"
          :closing="feedback.status.value === 'success'"
          type="submit"
          form="claim-create"
        />
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
