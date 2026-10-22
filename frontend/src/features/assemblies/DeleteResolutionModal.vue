<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Resolution } from '@/types/domain'

const props = defineProps<{
  open: boolean
  meetingId: string
  resolution: Resolution | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'deleted': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const deleting = ref(false)

watch(() => props.open, (open) => {
  if (!open) feedback.reset()
})

async function handleConfirm() {
  if (!props.resolution) return
  deleting.value = true
  feedback.reset()
  try {
    await meetingsApi.deleteResolution(props.meetingId, props.resolution.id)
    emit('update:open', false)
    emit('deleted')
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
    :title="t('meetings.deleteResolutionTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('meetings.deleteResolutionMessage') }}
      <template v-if="resolution?.title">
        <br />
        <span class="font-medium">{{ resolution.title }}</span>
      </template>
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
          :title="t('common.delete')"
          :loading="deleting"
          @click="handleConfirm"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
