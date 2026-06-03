<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormRow from '@/components/molecules/CFormRow.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { usersApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { Unit } from '@/types/domain'

const props = defineProps<{
  open: boolean
  units: Unit[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const loading = ref(false)

const form = reactive({ first_name: '', last_name: '', email: '', role: 'co_owner', unit_id: '' })

const ROLES = ['manager', 'board_member', 'co_owner', 'tenant'] as const

const roleOptions = computed(() =>
  ROLES.map(r => ({ value: r, label: t(`roles.${r}`) }))
)

const activeUnits = computed(() =>
  props.units.filter(u => u.active).map(u => ({ value: u.id, label: u.unit_number }))
)

const isTenant = computed(() => form.role === 'tenant')

const isValid = computed(() =>
  !!form.first_name.trim() &&
  !!form.last_name.trim() &&
  !!form.role &&
  (!isTenant.value || !!form.unit_id)
)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.first_name = ''
  form.last_name = ''
  form.email = ''
  form.role = 'co_owner'
  form.unit_id = ''
  feedback.reset()
})

watch(() => form.role, (role) => {
  if (role !== 'tenant') form.unit_id = ''
})

async function handleSubmit() {
  if (!isValid.value) return
  loading.value = true
  feedback.reset()
  try {
    await usersApi.createResident({
      first_name: form.first_name.trim(),
      last_name: form.last_name.trim(),
      email: form.email.trim() || null,
      role: form.role,
      unit_id: isTenant.value && form.unit_id ? form.unit_id : null,
    })
    feedback.setSuccess(t('residents.createResidentSuccess'))
    emit('created')
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
    :title="t('residents.createResidentTitle')"
    :closable="!loading"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="create-resident-form"
      @submit="handleSubmit"
    >
      <CFormRow :cols="2">
        <CFormInput
          v-model="form.first_name"
          :label="t('onboarding.firstName')"
          placeholder="Marie"
          autocomplete="given-name"
          required
          :disabled="loading"
        />
        <CFormInput
          v-model="form.last_name"
          :label="t('onboarding.lastName')"
          placeholder="Dupont"
          autocomplete="family-name"
          required
          :disabled="loading"
        />
      </CFormRow>
      <CFormSelect
        v-model="form.role"
        :label="t('residents.inviteRole')"
        :options="roleOptions"
        required
        :disabled="loading"
      />
      <CFormSelect
        v-if="isTenant"
        v-model="form.unit_id"
        :label="t('residents.inviteUnit')"
        :options="activeUnits"
        :hint="t('residents.inviteUnitHint')"
        required
        :disabled="loading"
      />
      <CFormInput
        v-model="form.email"
        type="email"
        :label="t('residents.emailOptional')"
        placeholder="prenom.nom@exemple.fr"
        autocomplete="email"
        :hint="t('residents.emailOptionalHint')"
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
          form="create-resident-form"
          :title="t('residents.createResidentButton')"
          :loading="loading"
          :closing="feedback.status.value === 'success'"
          :disabled="!isValid || loading"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
