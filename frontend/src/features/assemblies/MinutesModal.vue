<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import { meetingsApi, documentsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import { DEFAULT_DOCUMENT_VISIBLE_ROLES } from '@/utils/constants'

const props = defineProps<{
  open: boolean
  meetingId: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
}>()

const { t } = useI18n()
const minutesFeedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const minutesForm = ref<{ file: File | null }>({ file: null })
const savingMinutes = ref(false)
const downloadingMinutesDraft = ref(false)

watch(() => props.open, (open) => {
  if (open) {
    minutesForm.value = { file: null }
    minutesFeedback.reset()
  } else {
    forceClose()
  }
})

async function downloadMinutesDraft() {
  downloadingMinutesDraft.value = true
  try {
    const response = await meetingsApi.getMinutesDraft(props.meetingId)
    const blob = new Blob([response.data.markdown], { type: 'text/markdown;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `pv-ag-${props.meetingId}.md`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
  } catch (err) {
    minutesFeedback.setError(toErrorMessage(err))
  } finally {
    downloadingMinutesDraft.value = false
  }
}

async function saveMinutes() {
  if (!minutesForm.value.file) return
  savingMinutes.value = true
  minutesFeedback.reset()
  try {
    const file = minutesForm.value.file
    const extension = file.name.includes('.') ? file.name.split('.').pop() ?? 'pdf' : 'pdf'
    const upload = await documentsApi.getUploadUrl(extension, file.type || 'application/pdf', file.size)
    await documentsApi.uploadToS3(file, upload.data.url, file.type || 'application/pdf')
    const confirmed = await documentsApi.confirmUpload({
      key: upload.data.key,
      bucket: upload.data.bucket,
      original_name: file.name,
      mime_type: file.type || 'application/pdf',
      size_bytes: file.size,
      visible_roles: [...DEFAULT_DOCUMENT_VISIBLE_ROLES],
    })
    await meetingsApi.attachMinutes(props.meetingId, confirmed.data.id)
    minutesFeedback.setSuccess(t('common.saved'))
    scheduleClose(async () => {
      emit('update:open', false)
      emit('saved')
    })
  } catch (err) {
    minutesFeedback.setError(toErrorMessage(err))
  } finally {
    savingMinutes.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('meetings.minutes.upload_signed')"
    size="md"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-minutes"
      @submit="saveMinutes"
    >
      <p class="body-sm">
        {{ t('meetings.minutes.modal_lead') }}
      </p>
      <CBtn
        variant="ghost"
        size="sm"
        type="button"
        :icon="CIconDownload"
        :title="t('meetings.minutes.download_draft')"
        :loading="downloadingMinutesDraft"
        @click.prevent="downloadMinutesDraft"
      />
      <CFormDropzone
        accept="application/pdf,.pdf,.md,.txt"
        :uploading="savingMinutes"
        @files="minutesForm.file = $event[0] ?? null"
        @error="minutesFeedback.setError($event)"
      />
      <p
        v-if="minutesForm.file"
        class="text-xs text-on-surface-variant"
      >
        {{ minutesForm.file.name }}
      </p>
    </CForm>
    <template #footer>
      <CFormActions
        :status="minutesFeedback.status.value"
        :message="minutesFeedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          type="submit"
          form="meeting-minutes"
          :title="t('meetings.minutes.upload_signed')"
          :loading="savingMinutes"
          :disabled="!minutesForm.file"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
