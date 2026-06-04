<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
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
  'closed': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { forceExpire: forceClose } = useAutoClose()
const closing = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
})

async function handleClose() {
  closing.value = true
  feedback.reset()
  try {
    await budgetApi.close(props.budgetId)
    emit('closed')
    emit('update:open', false)
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    closing.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('budget.closeExerciseTitle')"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('budget.closeExerciseConfirmation', { year: fiscalYear }) }}
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
          variant="danger"
          :title="t('budget.closeExerciseConfirm')"
          :loading="closing"
          @click="handleClose"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
