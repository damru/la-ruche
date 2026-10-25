<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { reportsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Report } from '@/types/domain'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  report: Report | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'deleted': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const deleting = ref(false)

watch(() => props.open, (open) => {
  if (open) {
    feedback.reset()
  } else {
    forceClose()
  }
})

async function submit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.report) return
  deleting.value = true
  feedback.reset()
  try {
    await reportsApi.delete(props.report.id)
    feedback.setSuccess(t('reports.deleteSuccess'))
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
    :title="t('reports.confirmDeleteTitle')"
    size="sm"
    @update:open="emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('reports.confirmDeleteMessage', { title: report?.title ?? '' }) }}
    </p>
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
          variant="danger"
          :title="t('common.delete')"
          :loading="deleting"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          @click="submit"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
