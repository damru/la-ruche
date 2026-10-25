<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { unitsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { Unit, User } from '@/types/domain'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  unit: Unit | null
  residents: User[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'transferred': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const loading = ref(false)
const newOwnerId = ref('')
const transferDate = ref('')

const residentOptions = computed(() =>
  props.residents
    .filter(u => u.status !== 'pending_invite' && u.status !== 'archived')
    .map(u => ({ value: u.id, label: `${u.first_name} ${u.last_name}` }))
)

const isValid = computed(() => !!newOwnerId.value && !!transferDate.value)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  newOwnerId.value = ''
  transferDate.value = new Date().toISOString().slice(0, 10)
  feedback.reset()
})

async function handleSubmit() {
  if (!isValid.value || !props.unit) return
  loading.value = true
  feedback.reset()
  try {
    await unitsApi.transfer(props.unit.id, {
      user_id: newOwnerId.value,
      start_date: transferDate.value,
    })
    feedback.setSuccess(t('residents.transferSuccess'))
    emit('transferred')
    scheduleClose(() => { emit('update:open', false) })
  } catch {
    feedback.setError(t('common.error'))
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('residents.transferOwnership')"
    :closable="!loading"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="transfer-form"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="transferDate"
        type="date"
        :label="t('residents.transferDate')"
        required
        :disabled="loading"
      />
      <CFormSelect
        v-model="newOwnerId"
        :label="t('residents.transferNewOwner')"
        :options="residentOptions"
        :placeholder="t('residents.transferNoResidents')"
        required
        :disabled="loading"
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
          form="transfer-form"
          :title="t('residents.transferOwnership')"
          :loading="loading"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          :disabled="!isValid || loading"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
