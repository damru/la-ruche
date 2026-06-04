<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { allocationKeysApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { AllocationKey } from '@/types/domain'

const props = defineProps<{
  open: boolean
  keyToDelete: AllocationKey | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'deleted': [id: string]
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const deleting = ref(false)

watch(() => props.open, (open) => {
  if (!open) {
    forceClose()
    feedback.reset()
  }
})

async function confirm() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.keyToDelete) return
  const keyId = props.keyToDelete.id
  deleting.value = true
  feedback.reset()
  try {
    await allocationKeysApi.delete(keyId)
    feedback.setSuccess(t('settings.keys.deleteSuccess'))
    emit('deleted', keyId)
    scheduleClose(() => { emit('update:open', false) })
  } catch (e: unknown) {
    feedback.setError((e as { response?: { data?: { message?: string } } })?.response?.data?.message ?? t('common.error'))
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('settings.keys.confirmDeleteTitle')"
    size="sm"
    @update:open="emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('settings.keys.confirmDeleteMessage', { name: keyToDelete?.name ?? '' }) }}
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
          :title="t('common.deleteLabel')"
          :loading="deleting"
          :closing="feedback.status.value === 'success'"
          @click="confirm"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
