<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { claimsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { Claim } from '@/types/domain'

const props = defineProps<{
  open: boolean
  claim: Claim | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'deleted': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()

const deletingClaim = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
})

async function handleDelete() {
  if (feedback.status.value === 'success') {
    forceClose()
    return
  }

  if (!props.claim) return

  deletingClaim.value = true
  feedback.reset()

  try {
    await claimsApi.delete(props.claim.id)
    feedback.setSuccess(t('claims.deleteSuccess'))
    emit('deleted')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err: unknown) {
    feedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    deletingClaim.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('claims.confirmDeleteTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('claims.confirmDeleteMessage', { title: claim?.title ?? '' }) }}
    </p>

    <template #footer>
      <CFormActions
        :status="feedback.status.value"
        :message="feedback.message.value"
      >
        <CBtn
          variant="ghost"
          @click="$emit('update:open', false)"
        >
          {{ t('common.cancel') }}
        </CBtn>
        <CBtn
          variant="danger"
          :loading="deletingClaim"
          :closing="feedback.status.value === 'success'"
          @click="handleDelete"
        >
          {{ t('common.deleteLabel') }}
        </CBtn>
      </CFormActions>
    </template>
  </CModal>
</template>
