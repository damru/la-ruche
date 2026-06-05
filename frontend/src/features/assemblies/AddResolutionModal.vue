<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { VoteType } from '@/types/domain'

const props = defineProps<{
  open: boolean
  meetingId: string
  nextNumber: number
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const form = ref(defaultForm())

function defaultForm() {
  return {
    number: 1,
    title: '',
    description: '',
    vote_type: 'simple_majority' as VoteType,
  }
}

const voteTypeOptions = computed(() => [
  { value: 'simple_majority', label: t('meetings.vote.types.simple_majority') },
  { value: 'absolute_majority', label: t('meetings.vote.types.absolute_majority') },
  { value: 'double_majority', label: t('meetings.vote.types.double_majority') },
  { value: 'unanimity', label: t('meetings.vote.types.unanimity') },
])

function setResolutionNumber(v: string) {
  form.value.number = parseInt(v, 10) || 1
}

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = { ...defaultForm(), number: props.nextNumber }
  feedback.reset()
})

async function handleSubmit() {
  if (!form.value.title.trim()) return
  saving.value = true
  feedback.reset()
  try {
    await meetingsApi.createResolution(props.meetingId, {
      number: form.value.number,
      title: form.value.title,
      description: form.value.description || undefined,
      vote_type: form.value.vote_type,
    })
    feedback.setSuccess(t('common.saved'))
    scheduleClose(() => {
      emit('update:open', false)
      emit('created')
    })
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
    :title="t('meetings.newResolution')"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-resolution"
      @submit="handleSubmit"
    >
      <CFormInput
        :model-value="form.number"
        label="#"
        type="number"
        @update:model-value="setResolutionNumber"
      />
      <CFormInput
        v-model="form.title"
        :label="t('meetings.resolutionLabel')"
        :placeholder="t('meetings.resolutionLabelPlaceholder')"
      />
      <CFormTextarea
        v-model="form.description"
        :label="t('meetings.resolutionDescription')"
        :placeholder="t('meetings.resolutionDescriptionPlaceholder')"
      />
      <CFormSelect
        v-model="form.vote_type"
        :label="t('meetings.voteRule')"
        :options="voteTypeOptions"
      />
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
          form="meeting-resolution"
          :title="t('common.add')"
          :loading="saving"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
