<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { MeetingType } from '@/types/domain'
import { meetingsApi } from '@/api'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': [meetingId: string]
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)

function defaultForm() {
  return {
    meeting_type: 'annual' as MeetingType,
    meeting_date: '',
    location: '',
  }
}

const form = ref(defaultForm())
const agendaItems = ref<string[]>([''])

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = defaultForm()
  agendaItems.value = ['']
  feedback.reset()
})

function addAgendaItem() {
  agendaItems.value.push('')
}

function removeAgendaItem(index: number) {
  if (agendaItems.value.length <= 1) {
    agendaItems.value = ['']
    return
  }
  agendaItems.value.splice(index, 1)
}

async function handleSubmit() {
  if (feedback.status.value === 'success') {
    forceClose()
    return
  }

  if (!form.value.meeting_date) {
    feedback.setError(t('common.error'))
    return
  }

  saving.value = true
  feedback.reset()

  const normalizedAgenda = agendaItems.value
    .map(item => item.trim())
    .filter(Boolean)

  try {
    const response = await meetingsApi.create({
      meeting_type: form.value.meeting_type,
      meeting_date: new Date(form.value.meeting_date).toISOString(),
      location: form.value.location || undefined,
      agenda: normalizedAgenda.length ? normalizedAgenda.join('\n') : undefined,
    })

    feedback.setSuccess(t('common.saved'))
    scheduleClose(async () => {
      emit('update:open', false)
      emit('created', response.data.id)
    })
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
    :title="t('meetings.create')"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-create"
      @submit="handleSubmit"
    >
      <CFormSelect
        v-model="form.meeting_type"
        :label="t('meetings.infoFields.type')"
        :options="[
          { value: 'annual', label: t('meetings.types.annual') },
          { value: 'extraordinary', label: t('meetings.types.extraordinary') },
        ]"
      />
      <CFormInput
        v-model="form.meeting_date"
        :label="t('meetings.meetingDate')"
        type="datetime-local"
      />
      <CFormInput
        v-model="form.location"
        :label="t('meetings.location')"
      />
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium text-on-surface-variant">{{ t('meetings.agenda.title') }}</span>
        <div class="agenda-editor">
          <div
            v-for="(_, index) in agendaItems"
            :key="`agenda-${index}`"
            class="agenda-editor__row"
          >
            <CFormInput
              v-model="agendaItems[index]"
              :placeholder="`${t('meetings.agenda.title')} ${index + 1}`"
            />
            <CBtn
              v-if="agendaItems.length > 1"
              variant="icon"
              size="sm"
              :icon="CIconTrash"
              :title="t('common.delete')"
              :aria-label="t('common.delete')"
              @click="removeAgendaItem(index)"
            />
          </div>
          <CBtn
            variant="secondary"
            size="sm"
            :icon="CIconPlus"
            :title="t('common.add')"
            @click="addAgendaItem"
          />
        </div>
      </div>
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
          form="meeting-create"
          :title="t('common.save')"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>
.agenda-editor {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.agenda-editor__row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--space-2);
}
</style>
