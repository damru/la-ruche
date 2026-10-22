<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormDropzone from '@/components/molecules/CFormDropzone.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { meetingsApi, documentsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
  meetingId: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'uploaded': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const form = ref<{ file: File | null }>({ file: null })

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = { file: null }
  feedback.reset()
})

async function handleSubmit() {
  if (!form.value.file) return
  saving.value = true
  feedback.reset()
  try {
    const file = form.value.file
    const extension = file.name.includes('.') ? file.name.split('.').pop() ?? 'pdf' : 'pdf'
    const upload = await documentsApi.getUploadUrl(extension, file.type || 'application/pdf', file.size)
    await documentsApi.uploadToS3(file, upload.data.url, file.type || 'application/pdf')
    const confirmed = await documentsApi.confirmUpload({
      key: upload.data.key,
      bucket: upload.data.bucket,
      original_name: file.name,
      mime_type: file.type || 'application/pdf',
      size_bytes: file.size,
      visible_roles: ['manager', 'board_member', 'co_owner'],
    })
    await meetingsApi.attachMinutes(props.meetingId, confirmed.data.id)
    feedback.setSuccess(t('common.saved'))
    scheduleClose(() => {
      emit('update:open', false)
      emit('uploaded')
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
    :title="t('meetings.minutes.upload_signed')"
    size="md"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-minutes"
      @submit="handleSubmit"
    >
      <p class="body-sm">
        {{ t('meetings.minutes.modal_lead') }}
      </p>
      <CFormDropzone
        accept="application/pdf,.pdf,.md,.txt"
        :uploading="saving"
        @files="form.file = $event[0] ?? null"
        @error="feedback.setError($event)"
      />
      <p
        v-if="form.file"
        class="text-xs text-on-surface-variant"
      >
        {{ form.file.name }}
      </p>
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
          form="meeting-minutes"
          :title="t('meetings.minutes.upload_signed')"
          :loading="saving"
          :disabled="!form.file"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
