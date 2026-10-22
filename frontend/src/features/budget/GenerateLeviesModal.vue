<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormCheckbox from '@/components/atoms/CFormCheckbox.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCardNote from '@/components/molecules/CCardNote.vue'
import CIconMegaphone from '@/components/atoms/CIconMegaphone.svg'
import { budgetApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
  budgetId: string
  impactedLotsCount: number
  alreadyCalledDisplayAmount: number
  generatePreviewTotal: number
  selectedTotalForecast: number
  canGenerateLevies: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'generated': []
}>()

const { t } = useI18n()
const { formatCurrency } = useFormatters()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const loading = ref(false)
const form = ref({ due_date: '', include_works_fund: true, call_label: '' })

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
  const now = new Date()
  const nextQ = new Date(now.getFullYear(), Math.floor(now.getMonth() / 3) * 3 + 3, 1)
  form.value = {
    due_date: nextQ.toISOString().slice(0, 10),
    include_works_fund: true,
    call_label: '',
  }
})

const feedbackStatus = computed(() =>
  feedback.status.value !== 'idle'
    ? feedback.status.value
    : (!props.canGenerateLevies ? 'error' : 'idle'),
)

const feedbackMessage = computed(() =>
  feedback.status.value !== 'idle' ? feedback.message.value : '',
)

async function handleSubmit() {
  if (!form.value.due_date) return
  loading.value = true
  feedback.reset()
  try {
    await budgetApi.generateLevies(props.budgetId, {
      due_date: form.value.due_date,
      include_works_fund: form.value.include_works_fund,
      call_label: form.value.call_label.trim() || undefined,
    })
    emit('generated')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <CModal
    size="lg"
    :open="open"
    :title="t('budget.generateLeviesTitle')"
    @update:open="$emit('update:open', $event)"
  >
    <p class="text-xs text-on-surface-variant">
      {{ t('budget.generateLeviesSub') }}
    </p>
    <CForm
      id="budget-generate"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="form.due_date"
        :label="t('common.dueDate')"
        type="date"
        required
      />
      <div class="flex flex-col gap-1">
        <CFormCheckbox
          v-model="form.include_works_fund"
          :label="t('budget.includeWorksFund')"
        />
        <p class="text-xs text-on-surface-variant">
          {{ t('budget.worksFundLegal') }}
        </p>
      </div>
      <CFormInput
        v-model="form.call_label"
        :label="t('budget.callLabel')"
        :placeholder="t('budget.callLabelPlaceholder')"
      />
    </CForm>
    <CCardNote
      v-if="selectedTotalForecast > 0"
      class="flex-col"
    >
      <strong>{{ t('budget.generateSummaryLabel') }}</strong>
      <div class="grid gap-3 grid-cols-3">
        <div class="flex flex-col gap-1">
          <span class="text-2xs opacity-80">{{ t('budget.generateSummaryLotsLabel') }}</span>
          <strong>{{ impactedLotsCount }}</strong>
        </div>
        <div class="flex flex-col gap-1">
          <span class="text-2xs opacity-80">{{ t('common.alreadyCalled') }}</span>
          <strong class="font-mono tabular-nums">{{ formatCurrency(alreadyCalledDisplayAmount) }}</strong>
        </div>
        <div class="flex flex-col gap-1">
          <span class="text-2xs opacity-80">{{ t('budget.generateSummaryRemainingLabel') }}</span>
          <strong class="font-mono tabular-nums">{{ formatCurrency(generatePreviewTotal) }}</strong>
        </div>
      </div>
      <p v-if="generatePreviewTotal === 0">
        {{ t('budget.generateSummaryNothingToGenerate') }}
      </p>
    </CCardNote>
    <template #footer>
      <CFormActions
        :status="feedbackStatus"
        :message="feedbackMessage"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :icon="CIconMegaphone"
          :title="t('common.generate')"
          :disabled="!canGenerateLevies"
          :loading="loading"
          type="submit"
          form="budget-generate"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
