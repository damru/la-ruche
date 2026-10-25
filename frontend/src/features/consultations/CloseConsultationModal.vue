<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { consultationsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Consultation } from '@/types/domain'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  consultation: Consultation | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'closed': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const closing = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
})

async function handleConfirm() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.consultation) return
  closing.value = true
  feedback.reset()
  try {
    await consultationsApi.close(props.consultation.id)
    feedback.setSuccess(t('consultations.closeSuccess'))
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
    :title="t('consultations.confirmCloseTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('consultations.confirmCloseMessage') }}
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
          :title="t('consultations.closeAction')"
          :loading="closing"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          @click="handleConfirm"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
