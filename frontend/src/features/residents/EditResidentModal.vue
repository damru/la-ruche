<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormRow from '@/components/molecules/CFormRow.vue'
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
  'saved': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const loading = ref(false)

const form = reactive({ first_name: '', last_name: '', email: '', phone: '' })

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.first_name = props.user?.first_name ?? ''
  form.last_name = props.user?.last_name ?? ''
  form.email = props.user?.email ?? ''
  form.phone = props.user?.phone ?? ''
  feedback.reset()
})

async function handleSubmit() {
  if (!props.user) return
  loading.value = true
  feedback.reset()
  try {
    await usersApi.updateResident(props.user.id, {
      first_name: form.first_name.trim(),
      last_name: form.last_name.trim(),
      email: form.email.trim() || null,
      phone: form.phone.trim() || null,
    })
    feedback.setSuccess(t('residents.editResidentSuccess'))
    emit('saved')
    scheduleClose(() => { emit('update:open', false) })
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    const msg = (e as { response?: { data?: { message?: string } } })?.response?.data?.message ?? ''
    if (status === 409 && msg.includes('email_already_exists')) {
      feedback.setError(t('residents.emailAlreadyExists'))
    } else {
      feedback.setError(t('common.error'))
    }
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('common.editProfile')"
    :closable="!loading"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="edit-resident-form"
      @submit="handleSubmit"
    >
      <CFormRow :cols="2">
        <CFormInput
          v-model="form.first_name"
          :label="t('common.firstName')"
          autocomplete="given-name"
          required
          :disabled="loading"
        />
        <CFormInput
          v-model="form.last_name"
          :label="t('onboarding.lastName')"
          autocomplete="family-name"
          required
          :disabled="loading"
        />
      </CFormRow>
      <CFormInput
        v-model="form.email"
        type="email"
        :label="t('common.email')"
        autocomplete="email"
        :disabled="loading"
      />
      <CFormInput
        v-model="form.phone"
        type="tel"
        :label="t('common.phone')"
        autocomplete="tel"
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
          form="edit-resident-form"
          :title="t('common.save')"
          :loading="loading"
          :closing="feedback.status.value === 'success'"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
