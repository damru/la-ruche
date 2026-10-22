<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CFormCheckbox from '@/components/atoms/CFormCheckbox.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CModal from '@/components/molecules/CModal.vue'
import { contractsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { ContractType, PaymentFrequency, VendorContract } from '@/types/domain'

// Amendment form state — mirrors the descriptive + frozen fields of a
// contract. Pattern B (ADR-009): submitting creates a NEW draft contract
// linked to the current one via predecessor_id; it does not mutate the
// active contract. The user must still activate the amendment for it to
// take effect (which then closes the predecessor).
interface AmendFormState {
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
  contract: VendorContract | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  amended: []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const fieldErrors = ref<Record<string, string>>({})

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

const defaultFormState = (): AmendFormState => ({
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

const form = ref<AmendFormState>(defaultFormState())

function hydrateFromContract(contract: VendorContract | null) {
  if (!contract) {
    form.value = defaultFormState()
    fieldErrors.value = {}
    feedback.reset()
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
    if (!open) {
      forceClose()
      return
    }
    hydrateFromContract(contract)
  },
  { immediate: true },
)

function normalizeAmountInput(value: string): string {
  return value.trim().replace(',', '.')
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

function closeModal() {
  emit('update:open', false)
}

async function submitAmendment() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.contract) return
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

    await contractsApi.createAmendment(props.contract.id, payload)
    feedback.setSuccess(t('contracts.amendSuccess'))
    emit('amended')
    scheduleClose(() => closeModal())
  } catch (error: unknown) {
    feedback.setError(toErrorMessage(error, t('contracts.amendError')))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    size="lg"
    :title="t('contracts.amendTitle', { name: contract?.vendor_name ?? '' })"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="amend-contract-form"
      class="amend-form"
      @submit="submitAmendment"
    >
      <p class="body-sm amend-form__hint">
        {{ t('contracts.amendHint') }}
      </p>

      <div class="amend-form__grid">
        <CFormInput
          v-model="form.vendor_name"
          :label="t('common.vendor')"
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
          class="amend-form__field--full"
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

      <div class="amend-form__renewal">
        <CFormCheckbox
          v-model="form.auto_renewal"
          class="amend-form__checkbox"
          :label="t('contracts.autoRenewal')"
        />
        <p class="text-xs text-on-surface-variant">
          {{ t('contracts.form.autoRenewalHint') }}
        </p>
      </div>

      <div class="amend-form__grid">
        <CFormInput
          v-model="form.notice_days"
          :label="t('contracts.noticePeriod')"
          :hint="t('contracts.form.noticeDaysHint')"
          :error="fieldErrors.notice_days"
          type="number"
          :disabled="!form.auto_renewal"
        />
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
          @click="closeModal"
        />
        <CBtn
          :title="t('contracts.amendSubmit')"
          type="submit"
          form="amend-contract-form"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
          :disabled="saving"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>
.amend-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.amend-form__hint {
  margin: 0;
  color: var(--color-on-surface-variant);
}

.amend-form__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-4);
}

.amend-form__field--full {
  grid-column: 1 / -1;
}

.amend-form__renewal {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-4);
  border-radius: var(--radius-xl);
  background: var(--color-surface-container-low);
}

.amend-form__checkbox {
  font-weight: var(--fw-medium);
}

@media (max-width: 720px) {
  .amend-form__grid {
    grid-template-columns: 1fr;
  }
}
</style>
