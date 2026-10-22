<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
  meetingId: string
  meetingLabel: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'deleted': []
}>()

const { t } = useI18n()
const router = useRouter()
const feedback = useFormFeedback()
const deleting = ref(false)

watch(() => props.open, (open) => {
  if (!open) feedback.reset()
})

async function handleConfirm() {
  deleting.value = true
  feedback.reset()
  try {
    await meetingsApi.delete(props.meetingId)
    emit('update:open', false)
    emit('deleted')
    router.push({ name: 'assembly-list' })
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
    :title="t('meetings.deleteMeetingTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('meetings.deleteMeetingMessage') }}
      <br />
      <span class="font-medium">{{ meetingLabel }}</span>
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
