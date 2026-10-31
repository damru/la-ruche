<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { tasksApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Task } from '@/types/domain'
import { useUiStore } from '@/stores/ui'
import { usePermissions } from '@/composables/usePermissions'

const props = defineProps<{
  open: boolean
  task: Task | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'deleted': []
  'cancel-requested': [task: Task]
}>()

const deleteBlocked = computed(
  () => Boolean(props.task?.is_in_logbook || props.task?.has_vendor_submission),
)

const { t } = useI18n()
const ui = useUiStore()
const { canAdministerTask } = usePermissions()
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
          : task?.has_vendor_submission
            ? t('tasks.deleteBlockedSubmissionsMessage', { title: task?.title ?? '' })
            : t('tasks.confirmDeleteMessage', { title: task?.title ?? '' })
      }}
    </p>
    <p
      v-if="deleteBlocked"
      class="body-sm"
    >
      {{ t('tasks.deleteBlockedHint') }}
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
          v-if="!deleteBlocked"
          variant="danger"
          :title="t('common.deleteLabel')"
          :loading="deleting"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          @click="handleSubmit"
        />
        <CBtn
          v-else-if="canAdministerTask"
          variant="primary"
          :title="t('tasks.cancelTask')"
          @click="task && $emit('cancel-requested', task)"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
