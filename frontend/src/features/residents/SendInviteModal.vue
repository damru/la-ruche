<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CCopyField from '@/components/molecules/CCopyField.vue'
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
  'invited': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { forceExpire: forceClose } = useAutoClose()
const loading = ref(false)
const inviteLink = ref('')

const form = reactive({ email: '' })

// Email is required only when user has none
const needsEmail = computed(() => !props.user?.email)

const isValid = computed(() =>
  !needsEmail.value || !!form.email.trim()
)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.email = ''
  inviteLink.value = ''
  copied.value = false
  feedback.reset()
})

async function handleSubmit() {
  if (!isValid.value || !props.user) return
  loading.value = true
  feedback.reset()
  try {
    const emailToSend = form.email.trim() || null
    const res = await usersApi.sendInvite(props.user.id, emailToSend)
    inviteLink.value = res.data.invite_link
    feedback.setSuccess(t('residents.inviteSuccess'))
    emit('invited')
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    const msg = (e as { response?: { data?: { message?: string } } })?.response?.data?.message ?? ''
    if (status === 409 && msg.includes('email_already_exists')) {
      feedback.setError(t('residents.emailAlreadyExists'))
    } else if (status === 422 && msg.includes('user_already_has_access')) {
      feedback.setError(t('residents.userAlreadyHasAccess'))
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
    :title="t('residents.sendInviteTitle')"
    :closable="!loading"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="send-invite-form"
      @submit="handleSubmit"
    >
      <p
        v-if="user"
        class="body-md"
      >
        {{ t('residents.sendInviteMessage', { name: `${user.first_name} ${user.last_name}` }) }}
      </p>

      <CFormInput
        v-if="user?.email && !needsEmail"
        :model-value="user.email"
        :label="t('residents.inviteEmail')"
        type="email"
        disabled
      />
      <CFormInput
        v-else
        v-model="form.email"
        type="email"
        :label="t('residents.inviteEmail')"
        placeholder="prenom.nom@exemple.fr"
        autocomplete="email"
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
          form="send-invite-form"
          :title="t('residents.inviteSendButton')"
          :loading="loading"
          :disabled="!isValid || loading"
        />
        <CBtn
          v-else
          variant="primary"
          :title="t('common.close')"
          @click="$emit('update:open', false)"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

