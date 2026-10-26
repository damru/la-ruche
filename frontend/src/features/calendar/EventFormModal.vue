<script setup lang="ts">
// EventFormModal — create/edit manual calendar events (EP-014-S03, US-CAL-005).
// Reused for both create (event=null) and edit (event set) per SaveTaskModal pattern.

import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormRow from '@/components/molecules/CFormRow.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CFileDropzone from '@/components/molecules/CFileDropzone.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { calendarEventsApi, documentsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import { extractFileExtension } from '@/utils/files'
import type { CalendarEventCategory, ManualCalendarEvent } from '@/types/domain'
import { CALENDAR_EVENT_CATEGORIES } from '@/utils/constants'
import { DOCUMENT_ROLES } from '@/utils/roles'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  event: ManualCalendarEvent | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const selectedDocuments = ref<File[]>([])

function defaultForm() {
  return {
    title: '',
    description: '',
    event_date: new Date().toISOString().slice(0, 10),
    event_time: '',
    category: '' as '' | CalendarEventCategory,
  }
}
const form = ref(defaultForm())

const categoryOptions = computed(() => [
  { value: '', label: t('calendar.event.categoryNone') },
  ...CALENDAR_EVENT_CATEGORIES.map(c => ({
    value: c,
    label: t(`calendar.event.categories.${c}`),
  })),
])

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  feedback.reset()
  selectedDocuments.value = []
  if (props.event) {
    form.value = {
      title: props.event.title,
      description: props.event.description ?? '',
      event_date: props.event.event_date,
      event_time: props.event.event_time ? props.event.event_time.slice(0, 5) : '',
      category: props.event.category ?? '',
    }
  } else {
    form.value = defaultForm()
  }
})

async function attachSelectedDocuments(eventId: string) {
  const failed: string[] = []
  for (const file of selectedDocuments.value) {
    try {
      const extension = extractFileExtension(file.name)
      const mimeType = file.type || 'application/octet-stream'
      const uploadUrl = await documentsApi.getUploadUrl(extension, mimeType, file.size)
      await documentsApi.uploadToS3(file, uploadUrl.data.url, mimeType)
      const document = await documentsApi.confirmUpload({
        key: uploadUrl.data.key,
        bucket: uploadUrl.data.bucket,
        original_name: file.name,
        mime_type: mimeType,
        size_bytes: file.size,
        visible_roles: [...DOCUMENT_ROLES],
      })
      await calendarEventsApi.attachDocument(eventId, document.data.id)
    } catch {
      failed.push(file.name)
    }
  }
  return failed
}

async function handleSubmit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!form.value.title.trim() || !form.value.event_date) return
  saving.value = true
  feedback.reset()
  try {
    const payload = {
      title: form.value.title,
      description: form.value.description || undefined,
      event_date: form.value.event_date,
      event_time: form.value.event_time || undefined,
      category: form.value.category || undefined,
    }
    const eventId = props.event
      ? (await calendarEventsApi.update(props.event.id, payload)).data.id
      : (await calendarEventsApi.create(payload)).data.id

    if (selectedDocuments.value.length > 0) {
      const failed = await attachSelectedDocuments(eventId)
      if (failed.length > 0) {
        feedback.setError(`${t('calendar.event.documentUploadError')}: ${failed.join(', ')}`)
        emit('saved')
        return
      }
    }

    feedback.setSuccess(t('common.saved'))
    emit('saved')
    scheduleClose(() => { emit('update:open', false) })
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
    :title="event ? t('calendar.event.edit') : t('calendar.event.create')"
    size="md"
    :closable="!saving"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="calendar-event-form"
      @submit="handleSubmit"
    >
      <CFormInput
        v-model="form.title"
        :label="t('common.title')"
        required
      />
      <CFormTextarea
        v-model="form.description"
        :label="t('common.description')"
        :rows="2"
      />
      <CFormRow :cols="2">
        <CFormInput
          v-model="form.event_date"
          :label="t('common.date')"
          type="date"
          required
        />
        <CFormInput
          v-model="form.event_time"
          :label="t('calendar.event.fields.time')"
          type="time"
        />
      </CFormRow>
      <CFormSelect
        v-model="form.category"
        :label="t('calendar.event.fields.category')"
        :options="categoryOptions"
      />
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('common.attachments') }}</label>
        <CFileDropzone
          v-model:files="selectedDocuments"
          :hint="t('calendar.event.documentHint')"
          :remove-label="t('common.delete')"
        />
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
          @click="emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="saving"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          type="submit"
          form="calendar-event-form"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
