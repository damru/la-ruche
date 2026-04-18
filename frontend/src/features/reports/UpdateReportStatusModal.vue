<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { reportsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Report, ReportStatus } from '@/types/domain'

const REPORT_STATUSES: ReportStatus[] = ['open', 'in_progress', 'closed']

const props = defineProps<{
  open: boolean
  report: Report | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'updated': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const updating = ref(false)
const statusForm = ref<ReportStatus>('in_progress')

const reportStatusOptions = computed(() =>
  REPORT_STATUSES.map(s => ({ value: s, label: t(`reports.statuses.${s}`) })),
)

watch(() => props.open, (open) => {
  if (open && props.report) {
    const s = props.report.status
    statusForm.value = s === 'closed' ? 'open' : (s === 'open' ? 'in_progress' : 'closed')
    feedback.reset()
  } else if (!open) {
    forceClose()
  }
})

async function submit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.report) return
  updating.value = true
  feedback.reset()
  try {
    await reportsApi.updateStatus(props.report.id, statusForm.value)
    feedback.setSuccess(t('reports.updateSuccess'))
    emit('updated')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    updating.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('reports.confirmStatusTitle')"
    size="sm"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="report-update-status"
      @submit="submit"
    >
      <CFormSelect
        v-model="statusForm"
        :label="t('reports.statusLabel')"
        :options="reportStatusOptions"
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
          :title="t('common.save')"
          :loading="updating"
          :closing="feedback.status.value === 'success'"
          type="submit"
          form="report-update-status"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
