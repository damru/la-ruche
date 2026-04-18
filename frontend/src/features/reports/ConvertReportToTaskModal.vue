<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { reportsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Report, TaskPriority } from '@/types/domain'

const DEFAULT_CONVERT_PRIORITY: TaskPriority = 'normal'

const props = defineProps<{
  open: boolean
  report: Report | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'converted': [taskId: string]
}>()

const { t } = useI18n()
const router = useRouter()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const converting = ref(false)
const convertPriority = ref<TaskPriority>(DEFAULT_CONVERT_PRIORITY)

const taskPriorityOptions = computed(() =>
  (['low', 'normal', 'high', 'urgent'] as TaskPriority[]).map(priority => ({
    value: priority,
    label: t(`tasks.priorities.${priority}`),
  })),
)

watch(() => props.open, (open) => {
  if (open) {
    convertPriority.value = DEFAULT_CONVERT_PRIORITY
    feedback.reset()
  } else {
    forceClose()
  }
})

async function submit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.report) return
  converting.value = true
  feedback.reset()
  try {
    const res = await reportsApi.convertToTask(props.report.id, {
      priority: convertPriority.value,
    })
    feedback.setSuccess(t('reports.convertSuccess'))
    emit('converted', res.data.task_id)
    scheduleClose(() => {
      emit('update:open', false)
      void router.push({ name: 'task-detail', params: { id: res.data.task_id } })
    })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    converting.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('reports.confirmConvertTitle')"
    size="sm"
    @update:open="emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('reports.confirmConvertMessage', { title: report?.title ?? '' }) }}
    </p>
    <CForm
      id="report-convert"
      @submit="submit"
    >
      <CFormSelect
        v-model="convertPriority"
        :label="t('tasks.priorityLabel')"
        :options="taskPriorityOptions"
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
          @click="emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :title="t('reports.convertToTask')"
          :loading="converting"
          :closing="feedback.status.value === 'success'"
          type="submit"
          form="report-convert"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
