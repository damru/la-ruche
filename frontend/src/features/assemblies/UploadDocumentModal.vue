<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormDropzone from '@/components/molecules/CFormDropzone.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { meetingsApi, documentsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import { DEFAULT_DOCUMENT_VISIBLE_ROLES, MEETING_DOCUMENT_KIND_NOTICE, MEETING_UPLOADABLE_DOCUMENT_KINDS } from '@/utils/constants'

const props = defineProps<{
  open: boolean
  meetingId: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const form = ref(defaultForm())

type UploadableKind = (typeof MEETING_UPLOADABLE_DOCUMENT_KINDS)[number]

function defaultForm() {
  return {
    kind: MEETING_DOCUMENT_KIND_NOTICE as UploadableKind,
    file: null as File | null,
  }
}

const documentKindOptions = computed(() =>
  MEETING_UPLOADABLE_DOCUMENT_KINDS.map(k => ({ value: k, label: t(`meetings.documents.${k}`) })),
)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  form.value = defaultForm()
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
      visible_roles: [...DEFAULT_DOCUMENT_VISIBLE_ROLES],
    })
    await meetingsApi.attachDocument(props.meetingId, {
      document_id: confirmed.data.id,
      kind: form.value.kind,
    })
    feedback.setSuccess(t('common.saved'))
    scheduleClose(() => {
      emit('update:open', false)
      emit('created')
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
    :title="t('meetings.documents.uploadTitle')"
    @update:open="$emit('update:open', $event)"
  >
    <CForm
      id="meeting-document"
      @submit="handleSubmit"
    >
      <CFormSelect
        v-model="form.kind"
        :label="t('meetings.documents.uploadType')"
        :options="documentKindOptions"
      />
      <CFormDropzone
        accept="application/pdf,.pdf,.doc,.docx,.png,.jpg,.jpeg"
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
          form="meeting-document"
          :title="t('common.add')"
          :loading="saving"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
