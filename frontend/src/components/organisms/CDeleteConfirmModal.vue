<script setup lang="ts">
// Organism — CDeleteConfirmModal
// Generic destructive-action confirmation modal following DDR-006:
// success feedback + 3 s auto-close + error message from backend code.

import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
  title: string
  message: string
  successMessage: string
  onDelete: () => Promise<void>
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

async function handleDelete() {
  if (feedback.status.value === 'success') { forceClose(); return }
  deleting.value = true
  feedback.reset()
  try {
    await props.onDelete()
    feedback.setSuccess(props.successMessage)
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
    :title="title"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ message }}
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
          :title="t('common.delete')"
          :loading="deleting"
          :closing="feedback.status.value === 'success'"
          @click="handleDelete"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
