<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { budgetApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
  fiscalYear: number
  currentRate: number
  minimumRate: number
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const form = ref({ contribution_rate_pct: '' })

const rateFormatter = new Intl.NumberFormat('fr-FR', { minimumFractionDigits: 1, maximumFractionDigits: 1 })

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = { contribution_rate_pct: props.currentRate.toFixed(2) }
  feedback.reset()
})

async function handleSubmit() {
  const contributionRatePct = Number.parseFloat(form.value.contribution_rate_pct)
  if (!Number.isFinite(contributionRatePct) || contributionRatePct < 0 || contributionRatePct > 100) {
    feedback.setError(t('errors.budget.works_fund_rate_invalid'))
    return
  }
  saving.value = true
  feedback.reset()
  try {
    await budgetApi.updateWorksFund(props.fiscalYear, { contribution_rate_pct: contributionRatePct })
    emit('saved')
    emit('update:open', false)
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('budget.worksFundConfigureTitle')"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="works-fund-form"
      @submit="handleSubmit"
    >
      <p class="text-xs text-on-surface-variant">
        {{ t('budget.worksFundConfigureHint') }}
      </p>
      <CFormInput
        v-model="form.contribution_rate_pct"
        :label="t('budget.worksFundRate')"
        type="number"
        :min="0"
        :max="100"
        step="0.01"
        required
      />
      <p class="text-xs text-on-surface-variant">
        {{ t('budget.worksFundRateHint', { minimum: rateFormatter.format(minimumRate) }) }}
      </p>
    </CForm>
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
          variant="primary"
          type="submit"
          form="works-fund-form"
          :title="t('common.save')"
          :loading="saving"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
