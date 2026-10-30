<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconFile from '@/components/atoms/CIconFile.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
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
import { CONTRACT_TYPES, DEFAULT_CONTRACT_TYPE, PAYMENT_FREQUENCIES } from '@/utils/constants'
import { MANAGER_ROLES } from '@/utils/roles'
import { extractFileExtension } from '@/utils/files'
import { useUiStore } from '@/stores/ui'

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
const ui = useUiStore()
const { formatDate, formatFileSize } = useFormatters()
const feedback = useFormFeedback()
const saving = ref(false)
const fieldErrors = ref<Record<string, string>>({})
const loadingDocuments = ref(false)
const uploadingDocuments = ref(false)
const existingDocuments = ref<Document[]>([])
const selectedDocuments = ref<File[]>([])
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
  contract_type: DEFAULT_CONTRACT_TYPE,
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
// Pattern B (ADR-009): once a contract leaves `draft`, the 7 structuring
// fields are gelés (frozen) — amount/start_date/end_date/notice_days/
// contract_type/payment_frequency/auto_renewal. They render read-only here;
// the backend also rejects any attempt to change them via PATCH. Use
// "Créer un avenant" (AmendContractModal) to propose a new version instead.
const isFrozen = computed(() => !!props.contract && props.contract.status !== 'draft')

const contractTypeOptions = computed(() =>
  [
    ...CONTRACT_TYPES.map(v => ({ value: v, label: t(`contracts.types.${v}`) })),
  ].sort((left, right) => left.label.localeCompare(right.label, 'fr')),
)

const paymentFrequencyOptions = computed(() => [
  ...PAYMENT_FREQUENCIES.map(v => ({ value: v, label: t(`contracts.frequencies.${v}`) })),
])

function normalizeAmountInput(value: string): string {
  return value.trim().replace(',', '.')
}

function resetForm() {
  form.value = defaultFormState()
  fieldErrors.value = {}
  existingDocuments.value = []
  selectedDocuments.value = []
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
    visible_roles: [...MANAGER_ROLES],
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
    :title="isEditing ? t('contracts.form.editTitle') : t('contracts.create')"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="contract-form"
      class="contract-form"
      @submit="saveContract"
    >
      <div class="contract-form__grid">
        <CFormInput
          v-model="form.vendor_name"
          :label="t('common.contractor')"
          :error="fieldErrors.vendor_name"
          required
        />
        <CFormSelect
          v-model="form.contract_type"
          :label="t('contracts.form.contractType')"
          :options="contractTypeOptions"
          :disabled="isFrozen"
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
          :disabled="isFrozen"
        />
        <CFormInput
          v-model="form.end_date"
          :label="t('common.endDate')"
          :hint="t('contracts.form.endDateHint')"
          :error="fieldErrors.end_date"
          type="date"
          :disabled="isFrozen"
        />
        <CFormInput
          v-model="form.amount"
          :label="t('common.amount')"
          :hint="t('contracts.form.amountHint')"
          :error="fieldErrors.amount"
          type="text"
          placeholder="0,00"
          :disabled="isFrozen"
        />
        <CFormSelect
          v-model="form.payment_frequency"
          :label="t('contracts.form.paymentFrequency')"
          :placeholder="t('contracts.form.paymentFrequencyPlaceholder')"
          :options="paymentFrequencyOptions"
          :error="fieldErrors.payment_frequency"
          :disabled="isFrozen"
        />
      </div>

      <div class="contract-form__renewal">
        <CFormCheckbox
          v-model="form.auto_renewal"
          class="contract-form__checkbox"
          :label="t('contracts.autoRenewal')"
          :disabled="isFrozen"
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
          :disabled="!form.auto_renewal || isFrozen"
        />
      </div>

      <div class="contract-form__documents">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('contracts.documents.titleInline') }}</label>
        <CFileDropzone
          v-model:files="selectedDocuments"
          :hint="t('contracts.documents.hint')"
          :remove-label="t('common.delete')"
        />

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
    </CForm>

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
          type="submit"
          form="contract-form"
          :loading="saving || uploadingDocuments"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          :disabled="saving || uploadingDocuments"
        />
      </CFormActions>
    </template>
  </CModal>

  <CDocumentLinkActionModal
    :open="showDocumentActionModal"
    :title="t('contracts.documents.detachTitle')"
    :description="t('contracts.documents.detachMessage', { name: pendingDocument?.original_name ?? '', contract: props.contract?.vendor_name ?? '' })"
    :warning="t('common.documentDeleteWarning', { name: pendingDocument?.original_name ?? '' })"
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
    <p class="body-sm">
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
          :closing-duration-ms="ui.autoCloseDelayMs"
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
  gap: var(--space-1);
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
