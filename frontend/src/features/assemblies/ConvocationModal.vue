<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconCheck from '@/components/atoms/CIconCheck.svg'
import CIconX from '@/components/atoms/CIconX.svg'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { ConvocationChannel, ConvocationChecklist } from '@/types/domain'

const props = defineProps<{
  open: boolean
  meetingId: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'called': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const calling = ref(false)
const loadingChecklist = ref(false)
const checklist = ref<ConvocationChecklist | null>(null)
const form = ref<{ channel: ConvocationChannel; sent_at: string }>({
  channel: 'lrar',
  sent_at: '',
})

watch(() => props.open, async (open) => {
  if (!open) { forceClose(); return }
  form.value = { channel: 'lrar', sent_at: '' }
  feedback.reset()
  checklist.value = null
  loadingChecklist.value = true
  try {
    const res = await meetingsApi.getConvocationChecklist(props.meetingId)
    checklist.value = res.data
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    loadingChecklist.value = false
  }
})

async function handleSubmit() {
  if (!checklist.value?.ready_to_send) {
    feedback.setError(t('meetings.convocation.checklist_incomplete'))
    return
  }
  calling.value = true
  feedback.reset()
  try {
    await meetingsApi.call(props.meetingId, {
      channel: form.value.channel,
      sent_at: form.value.sent_at || undefined,
    })
    feedback.setSuccess(t('common.saved'))
    scheduleClose(() => {
      emit('update:open', false)
      emit('called')
    })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    calling.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('meetings.convocation.modal_title')"
    size="md"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-convocation"
      @submit="handleSubmit"
    >
      <p class="body-sm">
        {{ t('meetings.convocation.modal_lead') }}<sup><a
          href="https://www.legifrance.gouv.fr/loda/id/JORFTEXT000000508310"
          target="_blank"
          rel="noopener noreferrer"
          class="legal-ref"
        >art.&nbsp;7–13</a></sup>
      </p>
      <div
        v-if="loadingChecklist"
        class="text-xs text-on-surface-variant"
      >
        {{ t('common.loading') }}
      </div>
      <div
        v-else-if="checklist"
        class="flex flex-col gap-2"
      >
        <span class="label-sm">{{ t('meetings.convocation.checklist_title') }}</span>
        <ul class="flex flex-col gap-1">
          <li
            v-for="item in checklist.items"
            :key="item.key"
            class="flex items-center gap-2 text-sm"
          >
            <CIconCheck
              v-if="item.satisfied"
              class="text-success icon-sm"
            />
            <CIconX
              v-else
              :class="[item.severity === 'Blocking' ? 'text-error' : 'text-on-surface-variant', 'icon-sm']"
            />
            <span>{{ t('meetings.convocation.items.' + item.key) }}</span>
            <span
              v-if="!item.satisfied && item.severity === 'Advisory'"
              class="text-xs text-on-surface-variant"
            >
              ({{ t('meetings.convocation.advisory') }})
            </span>
          </li>
        </ul>
      </div>
      <CFormSelect
        v-model="form.channel"
        :label="t('meetings.convocation.channel_label')"
        :options="[
          { value: 'lrar', label: t('meetings.convocation.channels.lrar') },
          { value: 'hand_delivery', label: t('meetings.convocation.channels.hand_delivery') },
          { value: 'email', label: t('meetings.convocation.channels.email') },
        ]"
      />
      <CFormInput
        v-model="form.sent_at"
        type="datetime-local"
        :label="t('meetings.convocation.sent_at_label')"
        :placeholder="t('meetings.convocation.sent_at_placeholder')"
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
          form="meeting-convocation"
          :title="t('meetings.call')"
          :loading="calling"
          :disabled="!checklist?.ready_to_send"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
