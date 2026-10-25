<script setup lang="ts">
import { DEFAULT_DOCUMENT_VISIBLE_ROLES, ROLES } from '@/utils/constants'
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormDropzone from '@/components/molecules/CFormDropzone.vue'
import { documentsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { extractFileExtension } from '@/utils/files'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'uploaded': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const { formatFileSize } = useFormatters()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()

const uploading = ref(false)
const selectedFile = ref<File | null>(null)
const selectedRoles = ref<string[]>([...DEFAULT_DOCUMENT_VISIBLE_ROLES])

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  selectedFile.value = null
  feedback.reset()
  selectedRoles.value = [...DEFAULT_DOCUMENT_VISIBLE_ROLES]
})

function onDropzoneFiles(files: File[]) {
  selectedFile.value = files[0] ?? null
}

async function uploadFile() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!selectedFile.value) return
  uploading.value = true
  feedback.reset()

  try {
    const file = selectedFile.value
    const ext = extractFileExtension(file.name)

    const urlRes = await documentsApi.getUploadUrl(ext, file.type, file.size)
    const { url, key, bucket } = urlRes.data

    await documentsApi.uploadToS3(file, url)

    await documentsApi.confirmUpload({
      key,
      bucket,
      original_name: file.name,
      mime_type: file.type,
      size_bytes: file.size,
      visible_roles: selectedRoles.value,
    })

    feedback.setSuccess(t('common.saved'))
    emit('uploaded')
    scheduleClose(() => { emit('update:open', false) })
  } catch (e: unknown) {
    feedback.setError(toErrorMessage(e, t('documents.uploadFailed')))
  } finally {
    uploading.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('documents.upload')"
    :closable="!uploading"
    @update:open="$emit('update:open', $event)"
  >
    <div class="flex flex-col gap-4">
      <CFormDropzone
        :uploading="uploading"
        @files="onDropzoneFiles"
        @error="feedback.setError($event)"
      />
      <p
        v-if="selectedFile"
        class="text-on-surface font-medium"
      >
        {{ selectedFile.name }} ({{ formatFileSize(selectedFile.size) }})
      </p>

      <div class="flex flex-col gap-2">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('documents.visiblePar') }}</label>
        <div class="flex flex-wrap gap-3">
          <CFormCheckbox
            v-for="role in ROLES"
            :key="role"
            v-model="selectedRoles"
            :value="role"
            :label="t(`roles.${role}`)"
          />
        </div>
      </div>
    </div>

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
          :title="t('common.upload')"
          :loading="uploading"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          :disabled="!selectedFile || uploading"
          @click="uploadFile"
        />
      </CFormActions>
    </template>
  </CModal>
</template>
