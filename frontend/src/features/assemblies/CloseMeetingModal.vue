<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
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
const closing = ref(false)

watch(() => props.open, (open) => {
  if (!open) feedback.reset()
})

async function handleConfirm() {
  closing.value = true
  feedback.reset()
  try {
    await meetingsApi.close(props.meetingId)
    emit('update:open', false)
    emit('closed')
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
          :title="t('meetings.closeMeetingConfirm')"
          :loading="closing"
          @click="handleConfirm"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
