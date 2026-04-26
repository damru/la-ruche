<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import { contractsApi, documentsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormatters } from '@/composables/useFormatters'
import { useFormFeedback } from '@/composables/useFormFeedback'
import type { Document, VendorContract } from '@/types/domain'
import { extractFileExtension } from '@/utils/files'

const props = defineProps<{
  open: boolean
  contract?: VendorContract | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  changed: []
}>()

const { t } = useI18n()
const { formatDate, formatFileSize } = useFormatters()
const feedback = useFormFeedback()

const documents = ref<Document[]>([])
const loading = ref(false)
const uploading = ref(false)
const deleting = ref(false)
const error = ref('')
const selectedFile = ref<File | null>(null)
const fileInputRef = ref<HTMLInputElement | null>(null)
const pendingDocument = ref<Document | null>(null)
const deleteDocumentOnDetach = ref(false)

watch(
  () => [props.open, props.contract?.id] as const,
  async ([open, contractId]) => {
    if (!open) {
      pendingDocument.value = null
      deleteDocumentOnDetach.value = false
      return
    }
    if (!contractId) return
    await loadDocuments()
  },
  { immediate: true },
)

async function loadDocuments() {
  if (!props.contract?.id) return
  loading.value = true
  error.value = ''
  feedback.reset()

  try {
    const response = await contractsApi.listDocuments(props.contract.id)
    documents.value = response.data
  } catch (currentError: unknown) {
    error.value = toErrorMessage(currentError, t('contracts.documents.loadError'))
  } finally {
    loading.value = false
  }
}

function openFilePicker() {
  fileInputRef.value?.click()
}

function onFileChange(event: Event) {
  const target = event.target as HTMLInputElement
  selectedFile.value = target.files?.[0] ?? null
}

async function uploadDocument() {
  if (!props.contract?.id || !selectedFile.value) return

  uploading.value = true
  error.value = ''
  feedback.reset()

  try {
    const file = selectedFile.value
    const extension = extractFileExtension(file.name)
    const uploadUrl = await documentsApi.getUploadUrl(extension, file.type, file.size)

    await documentsApi.uploadToS3(file, uploadUrl.data.url)

    const document = await documentsApi.confirmUpload({
      key: uploadUrl.data.key,
      bucket: uploadUrl.data.bucket,
      original_name: file.name,
      mime_type: file.type || 'application/octet-stream',
      size_bytes: file.size,
      visible_roles: ['manager', 'board_member'],
    })

    await contractsApi.attachDocument(props.contract.id, document.data.id)
    selectedFile.value = null
    if (fileInputRef.value) fileInputRef.value.value = ''
    feedback.setSuccess(t('contracts.documents.uploadSuccess'))
    await loadDocuments()
    emit('changed')
  } catch (currentError: unknown) {
    error.value = toErrorMessage(currentError, t('contracts.documents.uploadError'))
  } finally {
    uploading.value = false
  }
}

async function downloadDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch (currentError: unknown) {
    error.value = toErrorMessage(currentError, t('contracts.documents.downloadError'))
  }
}

const showDocumentActionModal = computed(() => pendingDocument.value !== null)

function closeDocumentActionModal() {
  if (deleting.value) return
  pendingDocument.value = null
  deleteDocumentOnDetach.value = false
}

function openDocumentActionModal(document: Document) {
  pendingDocument.value = document
  deleteDocumentOnDetach.value = false
}

async function detachDocument() {
  if (!props.contract?.id) return
  if (!pendingDocument.value) return
  const deleteDocument = deleteDocumentOnDetach.value

  deleting.value = true
  error.value = ''
  feedback.reset()

  try {
    if (deleteDocument) {
      await documentsApi.delete(pendingDocument.value.id)
      feedback.setSuccess(t('contracts.documents.deleteSuccess'))
    } else {
      await contractsApi.detachDocument(props.contract.id, pendingDocument.value.id)
      feedback.setSuccess(t('contracts.documents.detachSuccess'))
    }

    pendingDocument.value = null
    deleteDocumentOnDetach.value = false
    await loadDocuments()
    emit('changed')
  } catch (currentError: unknown) {
    error.value = toErrorMessage(
      currentError,
      deleteDocument
        ? t('contracts.documents.deleteError')
        : t('contracts.documents.detachError'),
    )
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    size="lg"
    :title="t('contracts.documents.title', { name: contract?.vendor_name ?? '' })"
    @update:open="emit('update:open', $event)"
  >
    <div class="documents-modal">
      <p class="documents-modal__hint">
        {{ contract?.contract_type === 'insurance' ? t('contracts.documents.insuranceHint') : t('contracts.documents.hint') }}
      </p>

      <div class="documents-modal__upload">
        <input
          ref="fileInputRef"
          type="file"
          class="documents-modal__file-input"
          @change="onFileChange"
        />
        <div class="documents-modal__upload-actions">
          <CBtn
            variant="secondary"
            @click="openFilePicker"
          >
            <CIconUpload />
            {{ t('contracts.documents.selectFile') }}
          </CBtn>
          <span class="documents-modal__file-name">
            {{ selectedFile?.name ?? t('contracts.documents.noFileSelected') }}
          </span>
        </div>
        <CBtn
          :disabled="!selectedFile || uploading"
          :loading="uploading"
          @click="uploadDocument"
        >
          {{ uploading ? t('common.uploadInProgress') : t('contracts.documents.attachUpload') }}
        </CBtn>
      </div>

      <CFormFeedback
        :status="feedback.status.value"
        :message="feedback.message.value"
      />

      <p
        v-if="error"
        class="documents-modal__error"
      >
        {{ error }}
      </p>

      <CPageLoader v-if="loading" />

      <template v-else>
        <div
          v-if="documents.length === 0"
          class="documents-modal__empty"
        >
          {{ t('contracts.documents.empty') }}
        </div>

        <div
          v-else
          class="documents-modal__list"
        >
          <div
            v-for="document in documents"
            :key="document.id"
            class="documents-modal__row"
          >
            <div class="documents-modal__meta">
              <div class="documents-modal__name">
                <CIconFile
                  class="documents-modal__file-icon"
                  aria-hidden="true"
                />
                <span>{{ document.original_name }}</span>
              </div>
              <div class="documents-modal__details">
                <span>{{ formatFileSize(document.size_bytes) }}</span>
                <span>·</span>
                <span>{{ formatDate(document.created_at) }}</span>
              </div>
            </div>
            <div class="documents-modal__row-actions">
              <CBtn
                variant="ghost"
                size="sm"
                @click="downloadDocument(document.id)"
              >
                <CIconDownload />
                {{ t('common.download') }}
              </CBtn>
              <CBtn
                variant="danger"
                size="sm"
                @click="openDocumentActionModal(document)"
              >
                <CIconTrash />
                {{ t('contracts.documents.detach') }}
              </CBtn>
            </div>
          </div>
        </div>
      </template>
    </div>

    <template #footer>
      <CBtn
        variant="ghost"
        @click="emit('update:open', false)"
      >
        {{ t('common.close') }}
      </CBtn>
    </template>
  </CModal>

  <CModal
    :open="showDocumentActionModal"
    size="sm"
    :stack="20"
    :closable="false"
    :title="t('contracts.documents.detachTitle')"
  >
    <div
      v-if="pendingDocument"
      class="documents-action-modal"
    >
      <p class="documents-action-modal__name">
        {{ pendingDocument.original_name }}
      </p>
      <label class="checkbox-label documents-action-modal__checkbox">
        <input
          v-model="deleteDocumentOnDetach"
          type="checkbox"
          :disabled="deleting"
        />
        {{ t('contracts.documents.deleteAlso') }}
      </label>
      <p
        v-if="deleteDocumentOnDetach"
        class="documents-action-modal__warning"
      >
        {{ t('contracts.documents.deleteWarning', { name: pendingDocument.original_name }) }}
      </p>
    </div>

    <template #footer>
      <CBtn
        variant="ghost"
        :disabled="deleting"
        @click.stop="closeDocumentActionModal()"
      >
        {{ t('common.cancel') }}
      </CBtn>
      <CBtn
        :variant="deleteDocumentOnDetach ? 'danger' : 'secondary'"
        :loading="deleting"
        @click.stop="detachDocument()"
      >
        {{ t('contracts.documents.detach') }}
      </CBtn>
    </template>
  </CModal>
</template>

<style scoped>
.documents-modal {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.documents-modal__hint {
  margin: 0;
  color: var(--color-on-surface-variant);
}

.documents-modal__upload {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
}

.documents-modal__upload-actions {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-wrap: wrap;
}

.documents-modal__file-input {
  display: none;
}

.documents-modal__file-name {
  color: var(--color-on-surface-variant);
}

.documents-modal__error {
  margin: 0;
  color: var(--color-error);
}

.documents-modal__empty {
  padding: var(--space-4);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
  color: var(--color-on-surface-variant);
}

.documents-modal__list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.documents-modal__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
}

.documents-modal__meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.documents-modal__name {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
  font-weight: var(--fw-medium);
}

.documents-modal__file-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  color: var(--color-primary);
}

.documents-modal__name span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.documents-modal__details {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--color-on-surface-variant);
  font-size: var(--text-sm);
}

.documents-modal__row-actions {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.documents-action-modal {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.documents-action-modal__name,
.documents-action-modal__warning {
  margin: 0;
}

.documents-action-modal__name {
  font-weight: var(--fw-medium);
  word-break: break-word;
}

.documents-action-modal__checkbox {
  color: var(--color-on-surface-variant);
}

.documents-action-modal__warning {
  padding: var(--space-3);
  border-radius: var(--radius-md);
  background: var(--color-error-container);
  color: var(--color-on-error-container);
}

@media (max-width: 720px) {
  .documents-modal__row {
    flex-direction: column;
    align-items: stretch;
  }

  .documents-modal__row-actions {
    justify-content: stretch;
  }
}
</style>
