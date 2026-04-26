<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormFeedback from '@/components/atoms/CFormFeedback.vue'
import CInput from '@/components/atoms/CInput.vue'
import CSelect from '@/components/atoms/CSelect.vue'
import CModal from '@/components/molecules/CModal.vue'
import { contractsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback, SUCCESS_AUTO_CLEAR_MS } from '@/composables/useFormFeedback'
import type { ContractType, PaymentFrequency, VendorContract } from '@/types/domain'

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
const feedback = useFormFeedback()
const saving = ref(false)
const fieldErrors = ref<Record<string, string>>({})

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
  feedback.reset()
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
  feedback.reset()
}

watch(
  () => [props.open, props.contract] as const,
  ([open, contract]) => {
    if (!open) return
    hydrateForm(contract)
  },
  { immediate: true },
)

function closeModal() {
  emit('update:open', false)
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

    if (props.contract?.id) {
      await contractsApi.update(props.contract.id, payload)
    } else {
      await contractsApi.create(payload)
    }

    feedback.setSuccess(t('common.saved'))
    emit('saved')
    setTimeout(() => closeModal(), SUCCESS_AUTO_CLEAR_MS)
  } catch (error: unknown) {
    feedback.setError(toErrorMessage(error, t('contracts.form.saveError')))
  } finally {
    saving.value = false
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
        <CInput
          v-model="form.vendor_name"
          :label="t('contracts.form.vendorName')"
          :error="fieldErrors.vendor_name"
          required
        />
        <CSelect
          v-model="form.contract_type"
          :label="t('contracts.form.contractType')"
          :options="contractTypeOptions"
        />
        <CInput
          v-model="form.subject"
          :label="t('contracts.form.subject')"
          :error="fieldErrors.subject"
          required
          class="contract-form__field--full"
        />
        <CInput
          v-model="form.vendor_email"
          :label="t('contracts.form.vendorEmail')"
          type="email"
        />
        <CInput
          v-model="form.vendor_phone"
          :label="t('contracts.form.vendorPhone')"
        />
        <CInput
          v-model="form.start_date"
          :label="t('common.startDate')"
          :error="fieldErrors.start_date"
          type="date"
          required
        />
        <CInput
          v-model="form.end_date"
          :label="t('common.endDate')"
          :hint="t('contracts.form.endDateHint')"
          :error="fieldErrors.end_date"
          type="date"
        />
        <CInput
          v-model="form.amount"
          :label="t('common.amount')"
          :hint="t('contracts.form.amountHint')"
          :error="fieldErrors.amount"
          type="text"
          placeholder="0,00"
        />
        <CSelect
          v-model="form.payment_frequency"
          :label="t('contracts.form.paymentFrequency')"
          :placeholder="t('contracts.form.paymentFrequencyPlaceholder')"
          :options="paymentFrequencyOptions"
          :error="fieldErrors.payment_frequency"
        />
      </div>

      <div class="contract-form__renewal">
        <label class="checkbox-label contract-form__checkbox">
          <input
            v-model="form.auto_renewal"
            type="checkbox"
          />
          <span>
            {{ t('contracts.autoRenewal') }}
          </span>
        </label>
        <p class="field-hint">
          {{ t('contracts.form.autoRenewalHint') }}
        </p>
      </div>

      <div class="contract-form__grid">
        <CInput
          v-model="form.notice_days"
          :label="t('contracts.noticePeriod')"
          :hint="t('contracts.form.noticeDaysHint')"
          :error="fieldErrors.notice_days"
          type="number"
          :disabled="!form.auto_renewal"
        />
      </div>
    </div>

    <template #footer>
      <CFormFeedback
        :status="feedback.status.value"
        :message="feedback.message.value"
        style="flex:1"
      />
      <CBtn
        variant="ghost"
        @click="closeModal"
      >
        {{ t('common.cancel') }}
      </CBtn>
      <CBtn
        :loading="saving"
        :disabled="saving"
        @click="saveContract"
      >
        {{ saving ? t('common.saving') : t('common.save') }}
      </CBtn>
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
  border-radius: var(--radius-md);
  background: var(--color-surface-container-low);
}

.contract-form__checkbox {
  font-weight: var(--fw-medium);
}

@media (max-width: 720px) {
  .contract-form__grid {
    grid-template-columns: 1fr;
  }
}
</style>
