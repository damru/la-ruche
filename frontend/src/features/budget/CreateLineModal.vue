<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { budgetApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'

const props = defineProps<{
  open: boolean
  budgetId: string
  keyOptions: Array<{ value: string; label: string }>
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const form = reactive({ label: '', key_id: '', budgeted_amount: '' })

const isValid = computed(() =>
  !!form.label.trim() && !!form.key_id && parseFloat(form.budgeted_amount) > 0
)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.label = ''
  form.key_id = props.keyOptions[0]?.value ?? ''
  form.budgeted_amount = ''
  feedback.reset()
})

async function handleSubmit() {
  if (!isValid.value) {
    feedback.setError(t('common.required'))
    return
  }
  saving.value = true
  feedback.reset()
  try {
    await budgetApi.createLine(props.budgetId, {
      label: form.label.trim(),
      key_id: form.key_id,
      budgeted_amount: parseFloat(form.budgeted_amount),
    })
    emit('created')
    emit('update:open', false)
  } catch {
    feedback.setError(t('common.error'))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('budget.addLine')"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="budget-line-create"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="form.label"
        :label="t('budget.lineLabel')"
        required
      />
      <CFormSelect
        v-model="form.key_id"
        :label="t('budget.lineKey')"
        :options="keyOptions"
        required
      />
      <CFormInput
        v-model="form.budgeted_amount"
        :label="t('budget.lineBudgetedAmount')"
        type="number"
        :min="0"
        step="0.01"
        required
      />
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
          :loading="saving"
          type="submit"
          form="budget-line-create"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
