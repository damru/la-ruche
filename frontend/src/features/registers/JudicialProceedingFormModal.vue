<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { judicialProceedingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { JudicialProceeding } from '@/types/domain'
import { useUiStore } from '@/stores/ui'
import {
  DEFAULT_JUDICIAL_PROCEEDING_STATUS,
  DEFAULT_JUDICIAL_PROCEEDING_TYPE,
  JUDICIAL_PROCEEDING_STATUSES,
  JUDICIAL_PROCEEDING_STATUS_CLOSED,
  JUDICIAL_PROCEEDING_TYPES,
} from '@/utils/constants'

const props = defineProps<{
  open: boolean
  proceeding: JudicialProceeding | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)

const defaultForm = () => ({
  proceeding_type: DEFAULT_JUDICIAL_PROCEEDING_TYPE,
  status: DEFAULT_JUDICIAL_PROCEEDING_STATUS,
  subject: '',
  claimant: '',
  defendant: '',
  jurisdiction: '',
  case_reference: '',
  opened_at: '',
  closed_at: '',
  outcome: '',
})
const form = ref(defaultForm())

const isEdit = computed(() => props.proceeding !== null)
const isClosed = computed(() => form.value.status === JUDICIAL_PROCEEDING_STATUS_CLOSED)

const typeOptions = computed(() =>
  JUDICIAL_PROCEEDING_TYPES.map(value => ({ value, label: t(`registers.judicial.types.${value}`) })),
)
const statusOptions = computed(() =>
  JUDICIAL_PROCEEDING_STATUSES.map(value => ({ value, label: t(`registers.judicial.statuses.${value}`) })),
)

watch(() => props.open, (open) => {
  if (!open) {
    forceClose()
    feedback.reset()
    return
  }
  feedback.reset()
  if (props.proceeding) {
    const p = props.proceeding
    form.value = {
      proceeding_type: p.proceeding_type,
      status: p.status,
      subject: p.subject,
      claimant: p.claimant,
      defendant: p.defendant,
      jurisdiction: p.jurisdiction ?? '',
      case_reference: p.case_reference ?? '',
      opened_at: p.opened_at,
      closed_at: p.closed_at ?? '',
      outcome: p.outcome ?? '',
    }
  } else {
    form.value = defaultForm()
  }
})

async function save() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!form.value.subject.trim() || !form.value.claimant.trim() || !form.value.defendant.trim()) {
    feedback.setError(t('registers.judicial.partiesRequired'))
    return
  }
  if (!form.value.opened_at) {
    feedback.setError(t('registers.judicial.openedAtRequired'))
    return
  }
  if (isEdit.value && isClosed.value && !form.value.closed_at) {
    feedback.setError(t('registers.judicial.closedAtRequired'))
    return
  }
  saving.value = true
  feedback.reset()
  try {
    if (props.proceeding) {
      await judicialProceedingsApi.update(props.proceeding.id, {
        proceeding_type: form.value.proceeding_type,
        status: form.value.status,
        subject: form.value.subject.trim(),
        claimant: form.value.claimant.trim(),
        defendant: form.value.defendant.trim(),
        jurisdiction: form.value.jurisdiction.trim() || null,
        case_reference: form.value.case_reference.trim() || null,
        opened_at: form.value.opened_at,
        closed_at: isClosed.value ? form.value.closed_at : null,
        outcome: form.value.outcome.trim() || null,
      })
    } else {
      await judicialProceedingsApi.create({
        proceeding_type: form.value.proceeding_type,
        subject: form.value.subject.trim(),
        claimant: form.value.claimant.trim(),
        defendant: form.value.defendant.trim(),
        jurisdiction: form.value.jurisdiction.trim() || undefined,
        case_reference: form.value.case_reference.trim() || undefined,
        opened_at: form.value.opened_at,
      })
    }
    feedback.setSuccess(t('common.saved'))
    emit('saved')
    scheduleClose(() => { emit('update:open', false) })
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
    :title="isEdit ? t('registers.judicial.edit') : t('registers.judicial.new')"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="judicial-form"
      @submit="save"
    >
      <CFormInput
        v-model="form.subject"
        :label="t('registers.judicial.subject')"
        class="col-span-full"
        required
      />
      <CFormSelect
        v-model="form.proceeding_type"
        :label="t('registers.judicial.type')"
        :options="typeOptions"
        required
      />
      <CFormSelect
        v-if="isEdit"
        v-model="form.status"
        :label="t('status')"
        :options="statusOptions"
        required
      />
      <CFormInput
        v-model="form.claimant"
        :label="t('registers.judicial.claimant')"
        required
      />
      <CFormInput
        v-model="form.defendant"
        :label="t('registers.judicial.defendant')"
        required
      />
      <CFormInput
        v-model="form.jurisdiction"
        :label="t('registers.judicial.jurisdiction')"
      />
      <CFormInput
        v-model="form.case_reference"
        :label="t('registers.judicial.caseReference')"
      />
      <CFormInput
        v-model="form.opened_at"
        :label="t('registers.judicial.openedAt')"
        type="date"
        required
      />
      <CFormInput
        v-if="isEdit && isClosed"
        v-model="form.closed_at"
        :label="t('registers.judicial.closedAt')"
        type="date"
        required
      />
      <CFormTextarea
        v-if="isEdit"
        v-model="form.outcome"
        :label="t('registers.judicial.outcome')"
        :rows="3"
        class="col-span-full"
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
          @click="emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          type="submit"
          form="judicial-form"
          :title="t('common.save')"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
