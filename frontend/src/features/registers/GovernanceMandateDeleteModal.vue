<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { governanceApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { GovernanceMandate } from '@/types/domain'

const props = defineProps<{
  open: boolean
  mandate: GovernanceMandate | null
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
  if (!open) { forceClose(); feedback.reset(); return }
  feedback.reset()
})

async function handleDelete() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.mandate) return
  deleting.value = true
  feedback.reset()
  try {
    await governanceApi.delete(props.mandate.id)
    feedback.setSuccess(t('registers.governance.deleteSuccess'))
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
    :title="t('registers.governance.deleteTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('registers.governance.deleteMessage', { name: mandate?.holder_name ?? '' }) }}
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
          :title="t('common.deleteLabel')"
          :loading="deleting"
          :closing="feedback.status.value === 'success'"
          @click="handleDelete"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
