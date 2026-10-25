<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { tasksApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Task } from '@/types/domain'
import { TASK_STATUS_OPEN } from '@/utils/constants'

const props = defineProps<{
  open: boolean
  task: Task | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'reopened': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const reopening = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
})

async function handleSubmit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.task) return
  reopening.value = true
  feedback.reset()
  try {
    await tasksApi.update(props.task.id, { status: TASK_STATUS_OPEN })
    feedback.setSuccess(t('tasks.reopenSuccess'))
    emit('reopened')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    reopening.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('tasks.reopenTask')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('tasks.confirmReopenMessage', { title: task?.title ?? '' }) }}
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
          :title="t('tasks.reopenTask')"
          :loading="reopening"
          :closing="feedback.status.value === 'success'"
          @click="handleSubmit"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
