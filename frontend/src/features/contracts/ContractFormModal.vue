<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconFile from '@/components/atoms/CIconFile.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CIconUpload from '@/components/atoms/CIconUpload.svg'
import CFormCheckbox from '@/components/atoms/CFormCheckbox.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CModal from '@/components/molecules/CModal.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CDocumentLinkActionModal from '@/components/organisms/CDocumentLinkActionModal.vue'
import { contractsApi, documentsApi } from '@/api'
import { useFormatters } from '@/composables/useFormatters'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { ContractType, Document, PaymentFrequency, VendorContract } from '@/types/domain'
import { extractFileExtension } from '@/utils/files'

interface ContractFormState {
  vendor_name: string
  vendor_email: string
  vendor_phone: string
  subject: string
  contract_type: ContractType
  start_date: string
  end_date: string
  amount: string
  payment_frequency: '' | PaymentFrequency
  auto_renewal: boolean
  notice_days: string
}

const props = defineProps<{
  open: boolean
  contract?: VendorContract | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  saved: []
}>()

const { t } = useI18n()
const { formatDate, formatFileSize } = useFormatters()
const feedback = useFormFeedback()
const saving = ref(false)
const fieldErrors = ref<Record<string, string>>({})
const loadingDocuments = ref(false)
const uploadingDocuments = ref(false)
const existingDocuments = ref<Document[]>([])
const selectedDocuments = ref<File[]>([])
const documentInputRef = ref<HTMLInputElement | null>(null)
const documentsDragOver = ref(false)
const documentActionFeedback = useFormFeedback()
const documentActionLoading = ref(false)
const pendingDocument = ref<Document | null>(null)
const deletingContract = ref(false)
const deleteContractFeedback = useFormFeedback()
const showDeleteModal = ref(false)
const { schedule: scheduleFormClose, forceExpire: forceFormClose } = useAutoClose()
const { schedule: scheduleDeleteClose, forceExpire: forceDeleteClose } = useAutoClose()
// Cancel any pending auto-close timer if the parent dismisses the modal early
watch(() => props.open, (open) => { if (!open) forceFormClose() })
watch(showDeleteModal, (open) => { if (!open) forceDeleteClose() })

const defaultFormState = (): ContractFormState => ({
  vendor_name: '',
  vendor_email: '',
  vendor_phone: '',
  subject: '',
  contract_type: 'service',
  start_date: '',
  end_date: '',
  amount: '',
  payment_frequency: '',
  auto_renewal: false,
  notice_days: '',
})

const form = ref<ContractFormState>(defaultFormState())

const isEditing = computed(() => !!props.contract?.id)
const showDocumentActionModal = computed(() => pendingDocument.value !== null)

const contractTypeOptions = computed(() =>
  [
    { value: 'maintenance', label: t('contracts.types.maintenance') },
    { value: 'insurance', label: t('contracts.types.insurance') },
    { value: 'service', label: t('contracts.types.service') },
    { value: 'energy', label: t('contracts.types.energy') },
    { value: 'water', label: t('contracts.types.water') },
    { value: 'cleaning', label: t('contracts.types.cleaning') },
    { value: 'green_spaces', label: t('contracts.types.green_spaces') },
    { value: 'other', label: t('contracts.types.other') },
  ].sort((left, right) => left.label.localeCompare(right.label, 'fr')),
)

const paymentFrequencyOptions = computed(() => [
  { value: 'monthly', label: t('contracts.frequencies.monthly') },
  { value: 'quarterly', label: t('contracts.frequencies.quarterly') },
  { value: 'biannual', label: t('contracts.frequencies.biannual') },
  { value: 'annual', label: t('contracts.frequencies.annual') },
])

function normalizeAmountInput(value: string): string {
  return value.trim().replace(',', '.')
}

function resetForm() {
  form.value = defaultFormState()
  fieldErrors.value = {}
  existingDocuments.value = []
  selectedDocuments.value = []
  documentsDragOver.value = false
  feedback.reset()
  documentActionFeedback.reset()
  deleteContractFeedback.reset()
  showDeleteModal.value = false
}

function hydrateForm(contract?: VendorContract | null) {
  if (!contract) {
    resetForm()
    return
  }

  form.value = {
    vendor_name: contract.vendor_name,
    vendor_email: contract.vendor_email ?? '',
    vendor_phone: contract.vendor_phone ?? '',
    subject: contract.subject,
    contract_type: contract.contract_type,
    start_date: contract.start_date,
    end_date: contract.end_date ?? '',
    amount: contract.amount != null ? String(contract.amount) : '',
    payment_frequency: contract.payment_frequency ?? '',
    auto_renewal: contract.auto_renewal,
    notice_days: contract.notice_days != null ? String(contract.notice_days) : '',
  }
  fieldErrors.value = {}
  selectedDocuments.value = []
  documentsDragOver.value = false
  feedback.reset()
}

watch(
  () => [props.open, props.contract] as const,
  ([open, contract]) => {
    if (!open) return
    hydrateForm(contract)
    if (contract?.id) {
      void loadDocuments(contract.id)
    }
  },
  { immediate: true },
)

function closeModal() {
  emit('update:open', false)
}

function openDeleteModal() {
  if (!props.contract?.id) return
  deleteContractFeedback.reset()
  showDeleteModal.value = true
}

async function deleteContract() {
  if (deleteContractFeedback.status.value === 'success') {
    forceDeleteClose()
    return
  }
  if (!props.contract?.id) return

  deletingContract.value = true
  deleteContractFeedback.reset()

  try {
    await contractsApi.delete(props.contract.id)
    deleteContractFeedback.setSuccess(t('contracts.deleteSuccess'))
    emit('saved')
    scheduleDeleteClose(() => {
      showDeleteModal.value = false
      closeModal()
    })
  } catch (error: unknown) {
    deleteContractFeedback.setError(toErrorMessage(error, t('contracts.deleteError')))
  } finally {
    deletingContract.value = false
  }
}

async function loadDocuments(contractId: string) {
  loadingDocuments.value = true
  documentActionFeedback.reset()

  try {
    const response = await contractsApi.listDocuments(contractId)
    existingDocuments.value = response.data
  } catch (error: unknown) {
    feedback.setError(toErrorMessage(error, t('contracts.documents.loadError')))
  } finally {
    loadingDocuments.value = false
  }
}

function openDocumentPicker() {
  if (!documentInputRef.value) return
  documentInputRef.value.click()
}

function appendDocuments(files: File[]) {
  if (!files.length) return
  selectedDocuments.value = [...selectedDocuments.value, ...files]
}

function onDocumentChange(event: Event) {
  const input = event.target as HTMLInputElement
  appendDocuments(Array.from(input.files ?? []))
  if (documentInputRef.value) documentInputRef.value.value = ''
}

function onDocumentDrop(event: DragEvent) {
  documentsDragOver.value = false
  appendDocuments(Array.from(event.dataTransfer?.files ?? []))
}

function removeSelectedDocument(index: number) {
  selectedDocuments.value = selectedDocuments.value.filter((_, currentIndex) => currentIndex !== index)
}

async function uploadDocument(contractId: string, file: File) {
  const extension = extractFileExtension(file.name)
  const mimeType = file.type || 'application/octet-stream'
  const uploadUrl = await documentsApi.getUploadUrl(extension, mimeType, file.size)

  await documentsApi.uploadToS3(file, uploadUrl.data.url, mimeType)

  const document = await documentsApi.confirmUpload({
    key: uploadUrl.data.key,
    bucket: uploadUrl.data.bucket,
    original_name: file.name,
    mime_type: mimeType,
    size_bytes: file.size,
    visible_roles: ['manager', 'board_member'],
  })

  await contractsApi.attachDocument(contractId, document.data.id)
}

async function attachSelectedDocuments(contractId: string) {
  const failedDocuments: string[] = []

  for (const document of selectedDocuments.value) {
    try {
      await uploadDocument(contractId, document)
    } catch {
      failedDocuments.push(document.name)
    }
  }

  return failedDocuments
}

async function downloadDocument(documentId: string) {
  try {
    const response = await documentsApi.getDownloadUrl(documentId)
    window.open(response.data.url, '_blank', 'noopener')
  } catch (error: unknown) {
    feedback.setError(toErrorMessage(error, t('contracts.documents.downloadError')))
  }
}

function openDocumentActionModal(document: Document) {
  pendingDocument.value = document
  documentActionFeedback.reset()
}

function closeDocumentActionModal() {
  if (documentActionLoading.value) return
  pendingDocument.value = null
  documentActionFeedback.reset()
}

async function detachDocument(deleteDocument: boolean) {
  if (!props.contract?.id || !pendingDocument.value) return

  documentActionLoading.value = true
  documentActionFeedback.reset()

  try {
    if (deleteDocument) {
      await documentsApi.delete(pendingDocument.value.id)
    } else {
      await contractsApi.detachDocument(props.contract.id, pendingDocument.value.id)
    }

    existingDocuments.value = existingDocuments.value.filter(document => document.id !== pendingDocument.value?.id)
    pendingDocument.value = null
  } catch (error: unknown) {
    documentActionFeedback.setError(
      toErrorMessage(
        error,
        deleteDocument ? t('contracts.documents.deleteError') : t('contracts.documents.detachError'),
      ),
    )
  } finally {
    documentActionLoading.value = false
  }
}

function validate(): boolean {
  const errors: Record<string, string> = {}
  const normalizedAmount = normalizeAmountInput(form.value.amount)

  if (!form.value.vendor_name.trim()) errors.vendor_name = t('common.required')
  if (!form.value.subject.trim()) errors.subject = t('common.required')
  if (!form.value.start_date) errors.start_date = t('common.required')

  if (form.value.end_date && form.value.start_date && form.value.end_date < form.value.start_date) {
    errors.end_date = t('contracts.form.invalidDateRange')
  }

  if (normalizedAmount) {
    const parsedAmount = Number.parseFloat(normalizedAmount)
    if (Number.isNaN(parsedAmount) || parsedAmount <= 0) {
      errors.amount = t('contracts.form.amountInvalid')
    }
    if (!form.value.payment_frequency) {
      errors.payment_frequency = t('contracts.form.frequencyRequired')
    }
  }

  if (form.value.auto_renewal) {
    const parsedNoticeDays = Number.parseInt(form.value.notice_days, 10)
    if (!form.value.notice_days.trim() || Number.isNaN(parsedNoticeDays) || parsedNoticeDays <= 0) {
      errors.notice_days = t('contracts.form.noticeRequired')
    }
  }

  fieldErrors.value = errors
  return Object.keys(errors).length === 0
}

async function saveContract() {
  if (feedback.status.value === 'success') { forceFormClose(); return }
  if (!validate()) return

  saving.value = true
  feedback.reset()

  try {
    const amount = form.value.amount.trim()
      ? Number.parseFloat(normalizeAmountInput(form.value.amount))
      : undefined
    const noticeDays = form.value.notice_days.trim()
      ? Number.parseInt(form.value.notice_days, 10)
      : undefined

    const payload = {
      vendor_name: form.value.vendor_name.trim(),
      vendor_email: form.value.vendor_email.trim() || undefined,
      vendor_phone: form.value.vendor_phone.trim() || undefined,
      subject: form.value.subject.trim(),
      contract_type: form.value.contract_type,
      start_date: form.value.start_date,
      end_date: form.value.end_date || undefined,
      amount,
      payment_frequency: form.value.payment_frequency || undefined,
      auto_renewal: form.value.auto_renewal,
      notice_days: form.value.auto_renewal ? noticeDays : undefined,
    }

    const response = props.contract?.id
      ? await contractsApi.update(props.contract.id, payload)
      : await contractsApi.create(payload)

    if (selectedDocuments.value.length > 0) {
      uploadingDocuments.value = true
      const failedDocuments = await attachSelectedDocuments(response.data.id)
      uploadingDocuments.value = false

      if (failedDocuments.length > 0) {
        feedback.setError(`${t('contracts.documents.uploadError')}: ${failedDocuments.join(', ')}`)
        await loadDocuments(response.data.id)
        return
      }
    }

    feedback.setSuccess(t('common.saved'))
    emit('saved')
    scheduleFormClose(() => closeModal())
  } catch (error: unknown) {
    feedback.setError(toErrorMessage(error, t('contracts.form.saveError')))
  } finally {
    saving.value = false
    uploadingDocuments.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    size="lg"
    :title="isEditing ? t('contracts.form.editTitle') : t('contracts.form.createTitle')"
    @update:open="emit('update:open', $event)"
  >
    <div class="contract-form">
      <div class="contract-form__grid">
        <CFormInput
          v-model="form.vendor_name"
          :label="t('contracts.form.vendorName')"
          :error="fieldErrors.vendor_name"
          required
        />
        <CFormSelect
          v-model="form.contract_type"
          :label="t('contracts.form.contractType')"
          :options="contractTypeOptions"
        />
        <CFormInput
          v-model="form.subject"
          :label="t('contracts.form.subject')"
          :error="fieldErrors.subject"
          required
          class="contract-form__field--full"
        />
        <CFormInput
          v-model="form.vendor_email"
          :label="t('contracts.form.vendorEmail')"
          type="email"
        />
        <CFormInput
          v-model="form.vendor_phone"
          :label="t('contracts.form.vendorPhone')"
        />
        <CFormInput
          v-model="form.start_date"
          :label="t('common.startDate')"
          :error="fieldErrors.start_date"
          type="date"
          required
        />
        <CFormInput
          v-model="form.end_date"
          :label="t('common.endDate')"
          :hint="t('contracts.form.endDateHint')"
          :error="fieldErrors.end_date"
          type="date"
        />
        <CFormInput
          v-model="form.amount"
          :label="t('common.amount')"
          :hint="t('contracts.form.amountHint')"
          :error="fieldErrors.amount"
          type="text"
          placeholder="0,00"
        />
        <CFormSelect
          v-model="form.payment_frequency"
          :label="t('contracts.form.paymentFrequency')"
          :placeholder="t('contracts.form.paymentFrequencyPlaceholder')"
          :options="paymentFrequencyOptions"
          :error="fieldErrors.payment_frequency"
        />
      </div>

      <div class="contract-form__renewal">
        <CFormCheckbox
          v-model="form.auto_renewal"
          class="contract-form__checkbox"
          :label="t('contracts.autoRenewal')"
        />
        <p class="text-xs text-on-surface-variant">
          {{ t('contracts.form.autoRenewalHint') }}
        </p>
      </div>

      <div class="contract-form__grid">
        <CFormInput
          v-model="form.notice_days"
          :label="t('contracts.noticePeriod')"
          :hint="t('contracts.form.noticeDaysHint')"
          :error="fieldErrors.notice_days"
          type="number"
          :disabled="!form.auto_renewal"
        />
      </div>

      <div class="contract-form__documents">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('contracts.documents.titleInline') }}</label>
        <input
          ref="documentInputRef"
          type="file"
          class="contract-form__file-input"
          multiple
          @change="onDocumentChange"
        />
        <div
          :class="['dropzone', { 'dropzone--over': documentsDragOver }]"
          role="button"
          tabindex="0"
          @click="openDocumentPicker"
          @keydown.enter.prevent="openDocumentPicker"
          @keydown.space.prevent="openDocumentPicker"
          @dragenter.prevent="documentsDragOver = true"
          @dragover.prevent="documentsDragOver = true"
          @dragleave.prevent="documentsDragOver = false"
          @drop.prevent="onDocumentDrop"
        >
          <CIconUpload class="dropzone__icon" />
          <span class="body-xs dropzone__hint">{{ t('contracts.documents.hint') }}</span>
        </div>

        <div
          v-if="selectedDocuments.length"
          class="contract-form__selected-documents"
        >
          <div
            v-for="(document, index) in selectedDocuments"
            :key="`${document.name}-${index}`"
            class="contract-form__selected-document"
          >
            <span class="contract-form__selected-name">{{ document.name }}</span>
            <button
              type="button"
              class="contract-form__remove-document"
              :title="t('common.delete')"
              :aria-label="t('common.delete')"
              @click="removeSelectedDocument(index)"
            >
              ×
            </button>
          </div>
        </div>

        <div
          v-if="isEditing"
          class="contract-form__existing-documents"
        >
          <p class="text-xs text-on-surface-variant contract-form__documents-hint">
            {{ t('contracts.documents.attached') }}
          </p>
          <CPageLoader v-if="loadingDocuments" />
          <div
            v-else-if="existingDocuments.length === 0"
            class="contract-form__documents-empty"
          >
            {{ t('contracts.documents.empty') }}
          </div>
          <div
            v-else
            class="contract-form__documents-list"
          >
            <div
              v-for="document in existingDocuments"
              :key="document.id"
              class="contract-form__documents-row"
            >
              <div class="contract-form__documents-meta">
                <div class="contract-form__documents-name">
                  <CIconFile
                    class="icon-sm"
                  />
                  <span>{{ document.original_name }}</span>
                </div>
                <div class="contract-form__documents-details">
                  <span>{{ formatFileSize(document.size_bytes) }}</span>
                  <span>·</span>
                  <span>{{ formatDate(document.created_at) }}</span>
                </div>
              </div>
              <div class="contract-form__documents-actions">
                <CBtn
                  variant="ghost"
                  size="sm"
                  :icon="CIconDownload"
                  :title="t('common.download')"
                  @click="downloadDocument(document.id)"
                />
                <CBtn
                  variant="danger"
                  size="sm"
                  :icon="CIconTrash"
                  :title="t('contracts.documents.detach')"
                  @click="openDocumentActionModal(document)"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <CFormActions
        :status="feedback.status.value"
        :message="feedback.message.value"
      >
        <CBtn
          v-if="isEditing"
          variant="danger"
          :icon="CIconTrash"
          :title="t('common.delete')"
          :loading="deletingContract"
          class="mr-auto"
          @click="openDeleteModal"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="closeModal"
        />
        <CBtn
          :title="t('common.save')"
          :loading="saving || uploadingDocuments"
          :closing="feedback.status.value === 'success'"
          :disabled="saving || uploadingDocuments"
          @click="saveContract"
        />
      </CFormActions>
    </template>
  </CModal>

  <CDocumentLinkActionModal
    :open="showDocumentActionModal"
    :title="t('contracts.documents.detachTitle')"
    :description="t('contracts.documents.detachMessage', { name: pendingDocument?.original_name ?? '', contract: props.contract?.vendor_name ?? '' })"
    :warning="t('contracts.documents.deleteWarning', { name: pendingDocument?.original_name ?? '' })"
    :loading="documentActionLoading"
    :status="documentActionFeedback.status.value"
    :message="documentActionFeedback.message.value"
    @update:open="value => !value && closeDocumentActionModal()"
    @unlink="detachDocument(false)"
    @unlink-and-delete="detachDocument(true)"
  />

  <CModal
    :open="showDeleteModal"
    :title="t('contracts.deleteTitle')"
    size="sm"
    @update:open="showDeleteModal = $event"
  >
    <p class="body-xs">
      {{ t('contracts.deleteMessage', { name: props.contract?.vendor_name ?? '' }) }}
    </p>
    <template #footer>
      <CFormActions
        :status="deleteContractFeedback.status.value"
        :message="deleteContractFeedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          :disabled="deletingContract"
          @click="showDeleteModal = false"
        />
        <CBtn
          variant="danger"
          :title="t('common.deleteLabel')"
          :loading="deletingContract"
          :closing="deleteContractFeedback.status.value === 'success'"
          @click="deleteContract"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>
.contract-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.contract-form__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-4);
}

.contract-form__field--full {
  grid-column: 1 / -1;
}

.contract-form__renewal {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-4);
  border-radius: var(--radius-xl);
  background: var(--color-surface-container-low);
}

.contract-form__checkbox {
  font-weight: var(--fw-medium);
}

.contract-form__documents {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.contract-form__file-input {
  display: none;
}

.contract-form__selected-documents {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.contract-form__selected-document {
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

.contract-form__selected-name {
  max-width: 220px;
  overflow: clip;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-on-surface-variant);
  font-size: var(--text-sm);
}

.contract-form__remove-document {
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
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: unset;
  min-height: unset;
  padding: 0;
}

.contract-form__existing-documents {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.contract-form__documents-hint {
  margin: 0;
}

.contract-form__documents-empty {
  padding: var(--space-3);
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-low);
  color: var(--color-on-surface-variant);
}

.contract-form__documents-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.contract-form__documents-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-low);
}

.contract-form__documents-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.contract-form__documents-name {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  font-weight: var(--fw-medium);
  min-width: 0;
}

.contract-form__documents-name span {
  overflow: clip;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.contract-form__documents-details {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  color: var(--color-on-surface-variant);
  font-size: var(--text-xs);
}

.contract-form__documents-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

@media (max-width: 720px) {
  .contract-form__grid {
    grid-template-columns: 1fr;
  }

  .contract-form__documents-row {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
