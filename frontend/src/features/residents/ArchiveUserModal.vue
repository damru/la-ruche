<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { usersApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { User } from '@/types/domain'

const props = defineProps<{
  open: boolean
  user: User | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'archived': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const loading = ref(false)
const departureDate = ref('')

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  departureDate.value = new Date().toISOString().slice(0, 10)
  feedback.reset()
})

async function confirmArchive() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.user) return
  loading.value = true
  feedback.reset()
  try {
    await usersApi.deactivate(props.user.id, departureDate.value || undefined)
    feedback.setSuccess(t('residents.archiveUserSuccess'))
    emit('archived')
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
    :title="t('residents.archiveUserTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('residents.archiveUserMessage', { name: `${user?.first_name ?? ''} ${user?.last_name ?? ''}`.trim() }) }}
    </p>
    <CFormInput
      v-model="departureDate"
      type="date"
      :label="t('residents.archiveDepartureDate')"
      :disabled="loading"
    />
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
          variant="danger"
          :title="t('residents.archiveUser')"
          :loading="loading"
          :closing="feedback.status.value === 'success'"
          @click="confirmArchive"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
