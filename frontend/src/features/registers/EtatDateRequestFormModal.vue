<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { etatDateApi, unitsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Unit } from '@/types/domain'
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
const units = ref<Unit[]>([])

const defaultForm = () => ({
  unit_id: '',
  requested_at: new Date().toISOString().slice(0, 10),
  requester_name: '',
})
const form = ref(defaultForm())

const unitOptions = computed(() =>
  units.value.map(u => ({ value: u.id, label: u.unit_number })),
)

onMounted(async () => {
  const res = await unitsApi.list()
  units.value = res.data
})

watch(() => props.open, (open) => {
  if (!open) {
    forceClose()
    feedback.reset()
    return
  }
  feedback.reset()
  form.value = defaultForm()
})

async function save() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!form.value.unit_id) {
    feedback.setError(t('registers.etatDate.unitRequired'))
    return
  }
  if (!form.value.requester_name.trim()) {
    feedback.setError(t('registers.etatDate.requesterRequired'))
    return
  }
  saving.value = true
  feedback.reset()
  try {
    await etatDateApi.create(form.value.unit_id, {
      requested_at: form.value.requested_at,
      requester_name: form.value.requester_name.trim(),
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
    :title="t('registers.etatDate.new')"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="etat-date-form"
      @submit="save"
    >
      <CFormSelect
        v-model="form.unit_id"
        :label="t('common.lot')"
        :options="unitOptions"
        class="col-span-full"
        required
      />
      <CFormInput
        v-model="form.requester_name"
        :label="t('registers.etatDate.requesterName')"
        class="col-span-full"
        required
      />
      <CFormInput
        v-model="form.requested_at"
        :label="t('registers.etatDate.requestedAt')"
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
          :title="t('common.cancel')"
          @click="emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          type="submit"
          form="etat-date-form"
          :title="t('common.save')"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
