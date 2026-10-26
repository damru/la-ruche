<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { claimsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { Claim } from '@/types/domain'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  claim: Claim | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'declared': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()

function todayIso(): string {
  return new Date().toISOString().slice(0, 10)
}

const declaring = ref(false)
const form = ref({ declared_at: todayIso() })

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = { declared_at: todayIso() }
  feedback.reset()
})

async function handleSubmit() {
  if (feedback.status.value === 'success') {
    forceClose()
    return
  }

  if (!props.claim) return
  if (!form.value.declared_at) {
    feedback.setError(t('common.required'))
    return
  }

  declaring.value = true
  feedback.reset()

  try {
    await claimsApi.declare(props.claim.id, { declared_at: form.value.declared_at })
    feedback.setSuccess(t('claims.declareSuccess'))
    emit('declared')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err: unknown) {
    feedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    declaring.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('claims.declareTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="claim-declare"
      @submit="handleSubmit"
    >
      <p class="body-sm text-on-surface-variant m-0">
        {{ t('claims.declareConfirm') }}
      </p>
      <CFormInput
        v-model="form.declared_at"
        :label="t('claims.declaredAtField')"
        type="date"
        required
      />
    </CForm>

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
          variant="primary"
          :loading="declaring"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          type="submit"
          form="claim-declare"
        >
          {{ t('claims.declare') }}
        </CBtn>
      </CFormActions>
    </template>
  </CModal>
</template>
