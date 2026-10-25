<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { usersApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { User } from '@/types/domain'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  user: User | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'revoked': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const revokeFeedback = useFormFeedback()
const revokingInvite = ref(false)
const { schedule: scheduleRevokeClose, forceExpire: forceRevokeClose } = useAutoClose()

watch(() => props.open, (open) => {
  if (!open) { forceRevokeClose(); return }
  revokeFeedback.reset()
})

async function confirmRevokeInvite() {
  if (revokeFeedback.status.value === 'success') { forceRevokeClose(); return }
  if (!props.user?.invite_token_id) return
  revokingInvite.value = true
  revokeFeedback.reset()
  try {
    await usersApi.revokeInvite(props.user.invite_token_id)
    revokeFeedback.setSuccess(t('residents.revokeSuccess'))
    emit('revoked')
    scheduleRevokeClose(() => { emit('update:open', false) })
  } catch {
    revokeFeedback.setError(t('common.error'))
  } finally {
    revokingInvite.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('residents.confirmRevokeTitle')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('residents.confirmRevokeMessage', { email: user?.email ?? '' }) }}
    </p>
    <template #footer>
      <CFormActions
        :status="revokeFeedback.status.value"
        :message="revokeFeedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="danger"
          :title="t('residents.confirmRevokeTitle')"
          :loading="revokingInvite"
          :closing="revokeFeedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          @click="confirmRevokeInvite"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
