<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormRow from '@/components/molecules/CFormRow.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { maintenanceApi } from '@/api'
import { MAINTENANCE_CATEGORIES, DEFAULT_MAINTENANCE_CATEGORY } from '@/utils/constants'
import type { MaintenanceCategory } from '@/types/domain'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
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
  title: '',
  description: '',
  intervention_date: new Date().toISOString().slice(0, 10),
  cost: '',
  vendor_name: '',
  category: DEFAULT_MAINTENANCE_CATEGORY as MaintenanceCategory,
})

const categoryOptions = computed(() =>
  MAINTENANCE_CATEGORIES.map(value => ({ value, label: t(`maintenance.categories.${value}`) })),
)
const form = ref(defaultForm())

watch(() => props.open, (open) => {
  if (open) {
    form.value = defaultForm()
    feedback.reset()
  } else {
    forceClose()
  }
})

async function save() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!form.value.title.trim()) return
  saving.value = true
  feedback.reset()
  try {
    await maintenanceApi.create({
      title: form.value.title,
      description: form.value.description || undefined,
      intervention_date: form.value.intervention_date,
      cost: form.value.cost ? Number(form.value.cost) : undefined,
      vendor_name: form.value.vendor_name || undefined,
      category: form.value.category,
    })
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
    :title="t('maintenance.addEntry')"
    size="md"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="maintenance-entry"
      @submit="save"
    >
      <CFormInput
        v-model="form.title"
        :label="t('common.title')"
        required
      />
      <CFormTextarea
        v-model="form.description"
        :label="t('common.description')"
        :rows="2"
      />
      <CFormInput
        v-model="form.intervention_date"
        :label="t('maintenance.fields.interventionDate')"
        type="date"
        required
      />
      <CFormSelect
        v-model="form.category"
        :label="t('maintenance.fields.category')"
        :options="categoryOptions"
      />
      <CFormRow :cols="2">
        <CFormInput
          v-model="form.cost"
          :label="t('maintenance.fields.cost')"
          type="number"
          :step="0.01"
          :min="0"
        />
        <CFormInput
          v-model="form.vendor_name"
          :label="t('common.contractor')"
        />
      </CFormRow>
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
          :title="t('common.save')"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          type="submit"
          form="maintenance-entry"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
