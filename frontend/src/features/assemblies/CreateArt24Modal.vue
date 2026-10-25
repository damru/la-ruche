<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Resolution } from '@/types/domain'
import { VOTE_TYPE_SIMPLE_MAJORITY } from '@/utils/constants'

const props = defineProps<{
  open: boolean
  meetingId: string
  resolution: Resolution | null
  nextNumber: number
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
})

async function handleConfirm() {
  if (!props.resolution) return
  saving.value = true
  feedback.reset()
  try {
    await meetingsApi.createResolution(props.meetingId, {
      number: props.nextNumber,
      title: `${props.resolution.title} (Art. 24)`,
      description: props.resolution.description,
      vote_type: VOTE_TYPE_SIMPLE_MAJORITY,
    })
    feedback.setSuccess(t('common.saved'))
    scheduleClose(() => {
      emit('update:open', false)
      emit('created')
    })
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
    :title="t('meetings.createArt24Title')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('meetings.createArt24Message', { title: resolution?.title ?? '' }) }}
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
          :title="t('common.create')"
          :loading="saving"
          @click="handleConfirm"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
