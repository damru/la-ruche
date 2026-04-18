<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { budgetApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': [id: string]
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const form = ref({ fiscal_year: new Date().getFullYear() + 1 })

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = { fiscal_year: new Date().getFullYear() + 1 }
  feedback.reset()
})

async function handleSubmit() {
  saving.value = true
  feedback.reset()
  try {
    const res = await budgetApi.create({ fiscal_year: Number(form.value.fiscal_year) })
    emit('created', res.data.id)
    emit('update:open', false)
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('budget.newFiscalYear')"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="budget-create"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="form.fiscal_year"
        :label="t('budget.fiscalYearLabel')"
        type="number"
        :min="2020"
        :max="2099"
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
          :title="t('common.create')"
          :loading="saving"
          type="submit"
          form="budget-create"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
