<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { contractsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { VendorContract } from '@/types/domain'

const props = defineProps<{
  open: boolean
  contract: VendorContract | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'terminated': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const terminating = ref(false)

watch(() => props.open, (open) => {
  if (!open) {
    forceClose()
    feedback.reset()
  }
})

async function confirm() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.contract) return
  terminating.value = true
  feedback.reset()
  try {
    await contractsApi.terminate(props.contract.id)
    feedback.setSuccess(t('contracts.terminateSuccess'))
    emit('terminated')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err) {
    feedback.setError(toErrorMessage(err, t('contracts.terminateError')))
  } finally {
    terminating.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('contracts.terminateTitle')"
    size="sm"
    @update:open="emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('contracts.terminateMessage', { name: contract?.vendor_name ?? '' }) }}
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
          :title="t('contracts.terminate')"
          :loading="terminating"
          :closing="feedback.status.value === 'success'"
          @click="confirm"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
