<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Meeting } from '@/types/domain'

const props = defineProps<{
  open: boolean
  meetingId: string
  meeting: Meeting
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
}>()

const { t } = useI18n()
const signaturesFeedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const savingSignatures = ref(false)
const signaturesForm = ref({
  president_signed: false,
  secretary_signed: false,
  scrutineers_signed: [] as string[],
})

function initForm() {
  signaturesForm.value = {
    president_signed: !!props.meeting.minutes_president_signed_at,
    secretary_signed: !!props.meeting.minutes_secretary_signed_at,
    scrutineers_signed: (props.meeting.minutes_scrutineers_signed ?? []).map(s => s.name),
  }
}

watch(() => props.open, (open) => {
  if (open) initForm()
  else forceClose()
})

async function saveSignatures() {
  savingSignatures.value = true
  signaturesFeedback.reset()
  try {
    await meetingsApi.signMinutes(props.meetingId, signaturesForm.value)
    signaturesFeedback.setSuccess(t('common.saved'))
    scheduleClose(async () => {
      emit('update:open', false)
      emit('saved')
    })
  } catch (err) {
    signaturesFeedback.setError(toErrorMessage(err))
  } finally {
    savingSignatures.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('meetings.signatures.modal_title')"
    size="md"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-signatures"
      @submit="saveSignatures"
    >
      <p class="body-sm">
        {{ t('meetings.signatures.modal_lead') }}<sup><a
          href="https://www.legifrance.gouv.fr/loda/article_lc/LEGIARTI000042078689/"
          target="_blank"
          rel="noopener noreferrer"
          class="legal-ref"
        >art.&nbsp;17–17-1</a></sup>
      </p>
      <CFormCheckbox v-model="signaturesForm.president_signed">
        {{ t('meetings.session_roles.president') }}
        <template v-if="meeting.president_name">
          — {{ meeting.president_name }}
        </template>
      </CFormCheckbox>
      <CFormCheckbox v-model="signaturesForm.secretary_signed">
        {{ t('meetings.session_roles.secretary') }}
        <template v-if="meeting.secretary_name">
          — {{ meeting.secretary_name }}
        </template>
      </CFormCheckbox>
      <div
        v-if="(meeting.scrutineers ?? []).length > 0"
        class="flex flex-col gap-1"
      >
        <span class="label-sm">{{ t('meetings.session_roles.scrutineers') }}</span>
        <CFormCheckbox
          v-for="scrutineer in meeting.scrutineers ?? []"
          :key="scrutineer.name"
          v-model="signaturesForm.scrutineers_signed"
          :value="scrutineer.name"
          :label="scrutineer.name"
        />
      </div>
    </CForm>
    <template #footer>
      <CFormActions
        :status="signaturesFeedback.status.value"
        :message="signaturesFeedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          type="submit"
          form="meeting-signatures"
          :title="t('common.save')"
          :loading="savingSignatures"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
