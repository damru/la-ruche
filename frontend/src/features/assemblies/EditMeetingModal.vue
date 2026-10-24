<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconPlus from '@/components/atoms/CIconPlus.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import { meetingsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type {Meeting} from '@/types/domain'
import { DEFAULT_MEETING_TYPE, MEETING_TYPES } from '@/utils/constants'

const props = defineProps<{
  open: boolean
  meeting: Meeting
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'updated': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const form = ref(defaultForm())
const agendaItems = ref<string[]>([''])

function defaultForm() {
  return {
    meeting_type: DEFAULT_MEETING_TYPE,
    meeting_date: '',
    location: '',
  }
}

function toDateTimeLocal(value: string): string {
  const date = new Date(value)
  const offsetMs = date.getTimezoneOffset() * 60_000
  return new Date(date.getTime() - offsetMs).toISOString().slice(0, 16)
}

const meetingTypeOptions = computed(() =>
  MEETING_TYPES.map(v => ({ value: v, label: t(`meetings.types.${v}`) })),
)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  const agenda = (props.meeting.agenda ?? '')
    .split(/\r?\n/)
    .map(item => item.trim())
    .filter(Boolean)
  form.value = {
    meeting_type: props.meeting.meeting_type,
    meeting_date: toDateTimeLocal(props.meeting.meeting_date),
    location: props.meeting.location ?? '',
  }
  agendaItems.value = agenda.length ? agenda : ['']
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
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!form.value.meeting_date) return
  saving.value = true
  feedback.reset()
  const normalizedAgenda = agendaItems.value.map(item => item.trim()).filter(Boolean)
  try {
    await meetingsApi.update(props.meeting.id, {
      meeting_type: form.value.meeting_type,
      meeting_date: new Date(form.value.meeting_date).toISOString(),
      location: form.value.location || undefined,
      agenda: normalizedAgenda.length ? normalizedAgenda.join('\n') : undefined,
    })
    feedback.setSuccess(t('common.saved'))
    scheduleClose(() => {
      emit('update:open', false)
      emit('updated')
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
    :title="t('common.edit')"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-edit"
      @submit="handleSubmit"
    >
      <CFormSelect
        v-model="form.meeting_type"
        :label="t('common.type')"
        :options="meetingTypeOptions"
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
      <div class="flex flex-col gap-1 col-span-full">
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
          form="meeting-edit"
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
  align-items: center;
}
</style>
