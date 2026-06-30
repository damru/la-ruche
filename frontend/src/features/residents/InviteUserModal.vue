<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCopyField from '@/components/molecules/CCopyField.vue'
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
  'invited': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const loading = ref(false)
const inviteLink = ref('')

const form = reactive({ email: '', role: 'co_owner', unit_id: '' })

const INVITE_ROLES = ['manager', 'board_member', 'co_owner', 'tenant'] as const

const roleOptions = computed(() =>
  INVITE_ROLES.map(r => ({ value: r, label: t(`roles.${r}`) }))
)

const activeUnits = computed(() =>
  props.units.filter(u => u.active).map(u => ({ value: u.id, label: u.unit_number }))
)

const isTenant = computed(() => form.role === 'tenant')
const isValid = computed(() =>
  !!form.email.trim() && !!form.role && (!isTenant.value || !!form.unit_id)
)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.email = ''
  form.role = 'co_owner'
  form.unit_id = ''
  inviteLink.value = ''
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
    const res = await usersApi.invite(
      form.email.trim(),
      form.role,
      isTenant.value ? form.unit_id : undefined,
    )
    inviteLink.value = res.data.invite_link
    feedback.setSuccess(t('residents.inviteSuccess'))
    emit('invited')
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
    :title="t('residents.inviteTitle')"
    :closable="!loading"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="invite-form"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="form.email"
        type="email"
        :label="t('residents.inviteEmail')"
        placeholder="prenom.nom@exemple.fr"
        autocomplete="email"
        required
        :disabled="loading || !!inviteLink"
      />
      <CFormSelect
        v-model="form.role"
        :label="t('residents.inviteRole')"
        :options="roleOptions"
        required
        :disabled="loading || !!inviteLink"
      />
      <CFormSelect
        v-if="isTenant"
        v-model="form.unit_id"
        :label="t('residents.inviteUnit')"
        :options="activeUnits"
        :hint="t('residents.inviteUnitHint')"
        required
        :disabled="loading || !!inviteLink"
      />

      <!-- Generated invite link -->
      <CCopyField
        v-if="inviteLink"
        :value="inviteLink"
        :label="t('residents.inviteLinkTitle')"
        :hint="t('residents.inviteLinkHint')"
        :copy-title="t('residents.inviteCopyLink')"
        :copied-title="t('residents.inviteCopied')"
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
          v-if="!inviteLink"
          variant="primary"
          type="submit"
          form="invite-form"
          :title="t('residents.inviteSendButton')"
          :loading="loading"
          :disabled="!isValid || loading"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

