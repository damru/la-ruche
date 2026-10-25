<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { tasksApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Task } from '@/types/domain'
import { TASK_STATUS_AWAITING_VALIDATION, TASK_STATUS_CLOSED } from '@/utils/constants'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  task: Task | null
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
const closeAmount = ref('')

const modalTitle = computed(() =>
  props.task?.status === TASK_STATUS_AWAITING_VALIDATION
    ? t('tasks.confirmCloseValidationTitle')
    : t('tasks.closeTask'),
)

const confirmButtonLabel = computed(() =>
  props.task?.status === TASK_STATUS_AWAITING_VALIDATION
    ? t('tasks.validateTask')
    : t('tasks.closeTask'),
)

const bodyMessage = computed(() =>
  props.task?.status === TASK_STATUS_AWAITING_VALIDATION
    ? t('tasks.confirmCloseValidationMessage', { title: props.task?.title ?? '' })
    : t('tasks.confirmCloseMessage', { title: props.task?.title ?? '' }),
)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  closeAmount.value = props.task?.completion_cost != null ? String(props.task.completion_cost) : ''
  feedback.reset()
})

async function handleSubmit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.task) return
  closing.value = true
  feedback.reset()
  try {
    await tasksApi.update(props.task.id, {
      status: TASK_STATUS_CLOSED,
      completion_cost: closeAmount.value ? Number(closeAmount.value) : null,
    })
    feedback.setSuccess(t('tasks.lifecycle.closed'))
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
    :title="modalTitle"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ bodyMessage }}
    </p>
    <CForm
      id="task-close"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="closeAmount"
        :label="t('tasks.completionCostLabel')"
        type="number"
        :min="0"
        :step="0.01"
        :hint="task?.status === TASK_STATUS_AWAITING_VALIDATION && task?.completion_cost != null
          ? t('tasks.completionCostVendorHint')
          : t('tasks.completionCostHint')"
        class="mt-4"
      />
    </CForm>
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
          :title="confirmButtonLabel"
          :loading="closing"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          type="submit"
          form="task-close"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
