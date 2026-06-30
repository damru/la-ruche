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
import { governanceApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { GovernanceMandate } from '@/types/domain'

const ROLES = ['syndic', 'council_president', 'council_member'] as const

const props = defineProps<{
  open: boolean
  mandate: GovernanceMandate | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)

const defaultForm = () => ({
  role: 'syndic' as string,
  holder_name: '',
  start_date: '',
  end_date: '',
  notes: '',
})
const form = ref(defaultForm())

const isEdit = computed(() => props.mandate !== null)

const roleOptions = computed(() =>
  ROLES.map(value => ({ value, label: t(`registers.governance.roles.${value}`) })),
)

watch(() => props.open, (open) => {
  if (!open) {
    forceClose()
    feedback.reset()
    return
  }
  feedback.reset()
  if (props.mandate) {
    const m = props.mandate
    form.value = {
      role: m.role,
      holder_name: m.holder_name,
      start_date: m.start_date,
      end_date: m.end_date ?? '',
      notes: m.notes ?? '',
    }
  } else {
    form.value = defaultForm()
  }
})

async function save() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!form.value.holder_name.trim()) {
    feedback.setError(t('registers.governance.holderRequired'))
    return
  }
  if (!form.value.start_date) {
    feedback.setError(t('registers.governance.startDateRequired'))
    return
  }
  saving.value = true
  feedback.reset()
  try {
    const payload = {
      role: form.value.role,
      holder_name: form.value.holder_name.trim(),
      start_date: form.value.start_date,
      end_date: form.value.end_date || null,
      notes: form.value.notes.trim() || null,
    }
    if (props.mandate) {
      await governanceApi.update(props.mandate.id, payload)
    } else {
      await governanceApi.create(payload)
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
    :title="isEdit ? t('registers.governance.edit') : t('registers.governance.new')"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="governance-form"
      @submit="save"
    >
      <CFormInput
        v-model="form.holder_name"
        :label="t('registers.governance.holder')"
        class="col-span-full"
        required
      />
      <CFormSelect
        v-model="form.role"
        :label="t('registers.governance.role')"
        :options="roleOptions"
        required
      />
      <CFormInput
        v-model="form.start_date"
        :label="t('registers.governance.startDate')"
        type="date"
        required
      />
      <CFormInput
        v-model="form.end_date"
        :label="t('registers.governance.endDate')"
        type="date"
        :hint="t('registers.governance.endDateHint')"
      />
      <CFormTextarea
        v-model="form.notes"
        :label="t('registers.governance.notes')"
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
          form="governance-form"
          :title="t('common.save')"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
