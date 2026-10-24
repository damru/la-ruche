<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Resolution } from '@/types/domain'
import { VOTE_TYPE_ABSOLUTE_MAJORITY } from '@/utils/constants'

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
const art25Feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const savingArt25 = ref(false)

watch(() => props.open, (open) => {
  if (!open) forceClose()
})

async function confirmArt25Resolution() {
  if (!props.resolution) return
  savingArt25.value = true
  art25Feedback.reset()
  try {
    await meetingsApi.createResolution(props.meetingId, {
      number: props.nextNumber,
      title: `${props.resolution.title} (Art. 25)`,
      description: props.resolution.description,
      vote_type: VOTE_TYPE_ABSOLUTE_MAJORITY,
    })
    art25Feedback.setSuccess(t('common.saved'))
    scheduleClose(async () => {
      emit('update:open', false)
      emit('created')
    })
  } catch (err) {
    art25Feedback.setError(toErrorMessage(err))
  } finally {
    savingArt25.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('meetings.createArt25Title')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('meetings.createArt25Message', { title: resolution?.title ?? '' }) }}
    </p>
    <template #footer>
      <CFormActions
        :status="art25Feedback.status.value"
        :message="art25Feedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :title="t('common.create')"
          :loading="savingArt25"
          @click="confirmArt25Resolution"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
