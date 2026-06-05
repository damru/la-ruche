<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { contractorsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Contractor } from '@/types/domain'

const props = defineProps<{
  open: boolean
  contractor: Contractor | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'deleted': []
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
  if (!props.contractor) return
  deleting.value = true
  feedback.reset()
  try {
    await contractorsApi.delete(props.contractor.id)
    feedback.setSuccess(t('contractors.deleteSuccess'))
    emit('deleted')
    scheduleClose(() => { emit('update:open', false) })
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
    :title="t('contractors.deleteTitle')"
    @update:open="emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('contractors.deleteConfirm', { name: contractor?.name }) }}
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
          :title="t('common.delete')"
          :loading="deleting"
          :closing="feedback.status.value === 'success'"
          @click="confirm"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
