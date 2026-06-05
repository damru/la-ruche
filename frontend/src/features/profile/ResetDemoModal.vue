<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { adminApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'reset': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const loading = ref(false)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
})

async function confirm() {
  loading.value = true
  feedback.reset()
  try {
    await adminApi.resetDemo()
    feedback.setSuccess(t('settings.account.resetAppSuccess'))
    emit('reset')
    scheduleClose(() => { emit('update:open', false) })
  } catch {
    feedback.setError(t('settings.account.resetAppError'))
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('settings.account.resetAppConfirmTitle')"
    :closable="!loading"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('settings.account.resetAppConfirmMessage') }}
    </p>
    <template #footer>
      <CFormActions
        :status="feedback.status.value"
        :message="feedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          :disabled="loading"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="danger"
          :title="t('settings.account.resetAppConfirm')"
          :loading="loading"
          :disabled="loading"
          @click="confirm"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
