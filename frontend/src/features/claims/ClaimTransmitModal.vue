<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { claimsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { Claim } from '@/types/domain'
import { CLAIM_STATUS_IN_PROGRESS } from '@/utils/constants'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  claim: Claim | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'transmitted': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()

type ClaimUpdatePayload = {
  title: string
  description?: string | null
  claim_type: string
  declared_at: string
  published_at?: string | null
  status: string
  unit_id?: string | null
  transmitted_at?: string | null
  reimbursement_date?: string | null
  reimbursement_amount?: string | null
}

function todayIso(): string {
  return new Date().toISOString().slice(0, 10)
}

function buildClaimUpdatePayload(claim: Claim, overrides: Partial<ClaimUpdatePayload>): ClaimUpdatePayload {
  return {
    title: claim.title,
    description: claim.description ?? null,
    claim_type: claim.claim_type,
    declared_at: claim.declared_at,
    published_at: claim.published_at ?? null,
    status: claim.status,
    unit_id: claim.unit_id ?? null,
    transmitted_at: claim.transmitted_at ?? null,
    reimbursement_date: claim.reimbursement_date ?? null,
    reimbursement_amount: claim.reimbursement_amount != null ? String(claim.reimbursement_amount) : null,
    ...overrides,
  }
}

const transmittingClaim = ref(false)
const form = ref({ transmitted_at: todayIso() })

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  if (!props.claim) return
  form.value = { transmitted_at: props.claim.transmitted_at ?? todayIso() }
  feedback.reset()
})

async function handleSubmit() {
  if (feedback.status.value === 'success') {
    forceClose()
    return
  }

  if (!props.claim) return
  if (!form.value.transmitted_at) {
    feedback.setError(t('claims.transmittedAtRequired'))
    return
  }

  transmittingClaim.value = true
  feedback.reset()

  try {
    await claimsApi.update(
      props.claim.id,
      buildClaimUpdatePayload(props.claim, {
        status: CLAIM_STATUS_IN_PROGRESS,
        transmitted_at: form.value.transmitted_at,
      }),
    )

    feedback.setSuccess(t('claims.transmitSuccess'))
    emit('transmitted')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err: unknown) {
    feedback.setError(toErrorMessage(err, t('common.error')))
  } finally {
    transmittingClaim.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('claims.transmitTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="claim-transmit"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="form.transmitted_at"
        :label="t('claims.fields.transmittedAt')"
        type="date"
        required
        :hint="t('claims.fields.transmittedAtHint')"
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
          :loading="transmittingClaim"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          type="submit"
          form="claim-transmit"
        >
          {{ t('claims.transmit') }}
        </CBtn>
      </CFormActions>
    </template>
  </CModal>
</template>
