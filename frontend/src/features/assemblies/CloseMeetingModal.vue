<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
  meetingId: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'closed': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const closing = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); feedback.reset(); return }
  feedback.reset()
})

async function handleConfirm() {
  if (feedback.status.value === 'success') { forceClose(); return }
  closing.value = true
  feedback.reset()
  try {
    await meetingsApi.close(props.meetingId)
    feedback.setSuccess(t('meetings.closeMeetingSuccess'))
    emit('closed')
    scheduleClose(() => { emit('update:open', false) })
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
    :title="t('meetings.closeMeetingTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('meetings.closeMeetingSub') }}
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
          :title="t('budget.closeExerciseConfirm')"
          :loading="closing"
          :closing="feedback.status.value === 'success'"
          @click="handleConfirm"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
