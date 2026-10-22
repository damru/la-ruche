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
  documentId: string | null
  documentName: string | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'deleted': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const deleting = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); feedback.reset(); return }
  feedback.reset()
})

async function handleConfirm() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.documentId) return
  deleting.value = true
  feedback.reset()
  try {
    await meetingsApi.detachDocument(props.meetingId, props.documentId)
    feedback.setSuccess(t('meetings.documents.detachSuccess'))
    emit('deleted')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('meetings.documents.detachTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('meetings.documents.detachMessage') }}
      <template v-if="documentName">
        <br />
        <span class="font-medium">{{ documentName }}</span>
      </template>
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
          :title="t('common.remove')"
          :loading="deleting"
          :closing="feedback.status.value === 'success'"
          @click="handleConfirm"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
