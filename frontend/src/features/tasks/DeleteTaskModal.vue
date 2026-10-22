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

const props = defineProps<{
  open: boolean
  task: Task | null
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
  if (!open) { forceClose(); return }
  feedback.reset()
})

async function handleSubmit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.task) return
  deleting.value = true
  feedback.reset()
  try {
    await tasksApi.delete(props.task.id)
    feedback.setSuccess(t('tasks.deleteSuccess'))
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
    :title="t('tasks.deleteTask')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{
        task?.is_in_logbook
          ? t('tasks.confirmDeleteLogbookMessage', { title: task?.title ?? '' })
          : t('tasks.confirmDeleteMessage', { title: task?.title ?? '' })
      }}
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
          :title="t('common.deleteLabel')"
          :loading="deleting"
          :closing="feedback.status.value === 'success'"
          @click="handleSubmit"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
