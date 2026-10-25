<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormChipSelect from '@/components/atoms/CFormChipSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { contractorsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Contractor } from '@/types/domain'
import { CONTRACT_TYPES } from '@/utils/constants'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  contractor: Contractor | null
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
  name: '',
  email: '',
  phone: '',
  domains: [] as string[],
  notes: '',
})
const form = ref(defaultForm())

const domainOptions = computed(() =>
  [...CONTRACT_TYPES]
    .map(value => ({ value, label: t(`contractors.domains.${value}`) }))
    .sort((a, b) => a.label.localeCompare(b.label)),
)

watch(() => props.open, (open) => {
  if (!open) {
    forceClose()
    feedback.reset()
    return
  }
  feedback.reset()
  if (props.contractor) {
    form.value = {
      name: props.contractor.name,
      email: props.contractor.email ?? '',
      phone: props.contractor.phone ?? '',
      domains: [...props.contractor.domains],
      notes: props.contractor.notes ?? '',
    }
  } else {
    form.value = defaultForm()
  }
})

async function save() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!form.value.name.trim()) {
    feedback.setError(t('contractors.nameRequired'))
    return
  }
  saving.value = true
  feedback.reset()
  try {
    const payload = {
      name: form.value.name.trim(),
      email: form.value.email.trim() || undefined,
      phone: form.value.phone.trim() || undefined,
      domains: form.value.domains,
      notes: form.value.notes.trim() || undefined,
    }
    if (props.contractor) {
      await contractorsApi.update(props.contractor.id, payload)
    } else {
      await contractorsApi.create(payload)
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
    :title="contractor ? t('contractors.editContractor') : t('contractors.newContractor')"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="contractor-form"
      @submit="save"
    >
      <CFormInput
        v-model="form.name"
        :label="t('lastName')"
        :placeholder="t('contractors.namePlaceholder')"
        required
      />
      <CFormInput
        v-model="form.email"
        :label="t('contractors.email')"
        type="email"
        :placeholder="t('contractors.emailPlaceholder')"
      />
      <CFormInput
        v-model="form.phone"
        :label="t('common.phone')"
        type="tel"
        :placeholder="t('common.phonePlaceholder')"
      />
      <CFormChipSelect
        v-model="form.domains"
        :label="t('contractors.domainsLabel')"
        :options="domainOptions"
        class="flex flex-col gap-1 col-span-full"
      />
      <CFormTextarea
        v-model="form.notes"
        :label="t('common.notes')"
        :rows="3"
        :placeholder="t('contractors.notesPlaceholder')"
        class="flex flex-col gap-1 col-span-full"
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
          form="contractor-form"
          :title="t('common.save')"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
