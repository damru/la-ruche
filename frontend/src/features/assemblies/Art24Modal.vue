<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Resolution } from '@/types/domain'

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
const art24Feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const savingArt24 = ref(false)

watch(() => props.open, (open) => {
  if (!open) forceClose()
})

async function confirmArt24Resolution() {
  if (!props.resolution) return
  savingArt24.value = true
  art24Feedback.reset()
  try {
    await meetingsApi.createResolution(props.meetingId, {
      number: props.nextNumber,
      title: `${props.resolution.title} (Art. 24)`,
      description: props.resolution.description,
      vote_type: 'simple_majority',
    })
    art24Feedback.setSuccess(t('common.saved'))
    scheduleClose(async () => {
      emit('update:open', false)
      emit('created')
    })
  } catch (err) {
    art24Feedback.setError(toErrorMessage(err))
  } finally {
    savingArt24.value = false
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
        :status="art24Feedback.status.value"
        :message="art24Feedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :title="t('common.create')"
          :loading="savingArt24"
          @click="confirmArt24Resolution"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
