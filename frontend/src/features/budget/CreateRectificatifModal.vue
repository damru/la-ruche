<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { budgetApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
  budgetId: string
  fiscalYear: number
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': [id: string]
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { forceExpire: forceClose } = useAutoClose()
const saving = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
})

async function handleSubmit() {
  saving.value = true
  feedback.reset()
  try {
    const res = await budgetApi.createRectificatif(props.budgetId)
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
    :title="t('budget.rectificatifTitle', { year: fiscalYear })"
    @update:open="$emit('update:open', $event)"
  >
    <CAlert
      variant="warning"
      :title="t('budget.approveVoteRequiredTitle')"
    >
      {{ t('budget.rectificatifVoteRequired') }}
    </CAlert>
    <p class="body-sm">
      {{ t('budget.rectificatifExplanation', { year: fiscalYear }) }}
    </p>
    <p class="body-sm">
      {{ t('budget.rectificatifWorksFundNotice') }}
    </p>
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
          :title="t('budget.rectificatifConfirm')"
          :loading="saving"
          @click="handleSubmit"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
