<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { budgetApi, meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { useFormatters } from '@/composables/useFormatters'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Meeting } from '@/types/domain'
import { MEETING_STATUS_CLOSED } from '@/utils/constants'

const props = defineProps<{
  open: boolean
  budgetId: string
  fiscalYear: number
  /** True when approving a rectificatif: approval also closes the amended budget. */
  isRectificatif?: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'approved': []
}>()

const { t } = useI18n()
const { formatDate } = useFormatters()
const feedback = useFormFeedback()
const { forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const meetings = ref<Meeting[]>([])
const form = reactive({ approval_meeting_id: '' })

// Only a held (closed) AG can be the legal basis of an approval — art. 24
// loi 1965. The backend enforces this too; filtering here avoids offering a
// choice that would be rejected.
const meetingOptions = computed(() =>
  meetings.value
    .filter(m => m.status === MEETING_STATUS_CLOSED)
    .map(m => ({ value: m.id, label: `${formatDate(m.meeting_date)} — ${t(`meetings.typeLabel.${m.meeting_type}`)}` })),
)

const isValid = computed(() => !!form.approval_meeting_id)

watch(() => props.open, async (open) => {
  if (!open) { forceClose(); return }
  form.approval_meeting_id = ''
  feedback.reset()
  try {
    const res = await meetingsApi.list()
    meetings.value = res.data
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  }
})

async function handleSubmit() {
  if (!isValid.value) {
    feedback.setError(t('common.required'))
    return
  }
  saving.value = true
  feedback.reset()
  try {
    await budgetApi.approve(props.budgetId, { approval_meeting_id: form.approval_meeting_id })
    emit('approved')
    emit('update:open', false)
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('budget.approveTitle', { year: fiscalYear })"
    @update:open="$emit('update:open', $event)"
  >
    <CAlert
      variant="warning"
      :title="t('budget.approveVoteRequiredTitle')"
    >
      {{ t('budget.approveVoteRequired') }}
    </CAlert>
    <p
      v-if="isRectificatif"
      class="body-sm"
    >
      {{ t('budget.approveRectificatifClosesParent') }}
    </p>
    <p class="body-sm">
      {{ t('budget.approveFreezeWarning') }}
    </p>
    <CForm
      id="budget-approve"
      @submit="handleSubmit"
    >
      <CFormSelect
        v-model="form.approval_meeting_id"
        :label="t('budget.approvalMeeting')"
        :options="meetingOptions"
        required
      />
      <p
        v-if="meetingOptions.length === 0"
        class="field-hint"
      >
        {{ t('budget.approveNoClosedMeeting') }}
      </p>
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
          :title="t('budget.approveConfirm')"
          :loading="saving"
          type="submit"
          form="budget-approve"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
