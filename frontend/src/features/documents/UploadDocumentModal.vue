<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CModal from '@/components/molecules/CModal.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CFormDropzone from '@/components/molecules/CFormDropzone.vue'
import { documentsApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import { useAutoClose } from '@/composables/useAutoClose'
import { extractFileExtension } from '@/utils/files'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'uploaded': []
}>()

const { t } = useI18n()
const { formatFileSize } = useFormatters()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()

const uploading = ref(false)
const uploadError = ref('')
const uploadSuccess = ref(false)
const selectedFile = ref<File | null>(null)
const selectedRoles = ref<string[]>(['manager', 'board_member', 'co_owner'])

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  selectedFile.value = null
  uploadError.value = ''
  uploadSuccess.value = false
  selectedRoles.value = ['manager', 'board_member', 'co_owner']
})

function onDropzoneFiles(files: File[]) {
  selectedFile.value = files[0] ?? null
}

async function uploadFile() {
  if (uploadSuccess.value) { forceClose(); return }
  if (!selectedFile.value) return
  uploading.value = true
  uploadError.value = ''

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

    uploadSuccess.value = true
    emit('uploaded')
    scheduleClose(() => { emit('update:open', false); uploadSuccess.value = false })
  } catch (e: unknown) {
    uploadError.value = toErrorMessage(e, t('documents.uploadFailed'))
  } finally {
    uploading.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('documents.upload')"
    @update:open="$emit('update:open', $event)"
  >
    <div class="upload-form">
      <CAlert
        v-if="uploadSuccess"
        variant="success"
        style="margin-bottom:12px"
      >
        {{ t('common.saved') }}
      </CAlert>
      <CAlert
        v-if="uploadError"
        variant="error"
        style="margin-bottom:12px"
      >
        {{ uploadError }}
      </CAlert>

      <CFormDropzone
        :uploading="uploading"
        @files="onDropzoneFiles"
        @error="uploadError = $event"
      />
      <p
        v-if="selectedFile"
        class="selected-file"
      >
        {{ selectedFile.name }} ({{ formatFileSize(selectedFile.size) }})
      </p>

      <div class="visibility-section">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('documents.visiblePar') }}</label>
        <div class="role-checkboxes">
          <CFormCheckbox
            v-for="role in ['manager', 'board_member', 'co_owner', 'tenant']"
            :key="role"
            v-model="selectedRoles"
            :value="role"
            :label="t(`roles.${role}`)"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <CBtn
        variant="ghost"
        :title="t('common.cancel')"
        @click="$emit('update:open', false)"
      />
      <CBtn
        variant="primary"
        :title="t('common.upload')"
        :loading="uploading"
        :closing="uploadSuccess"
        :disabled="!selectedFile || uploading"
        @click="uploadFile"
      />
    </template>
  </CModal>
</template>

<style scoped>
.upload-form { display: flex; flex-direction: column; gap: var(--space-4); }
.selected-file { color: var(--color-on-surface); font-weight: var(--fw-medium); }
.visibility-section { display: flex; flex-direction: column; gap: var(--space-2); }
.role-checkboxes { display: flex; flex-wrap: wrap; gap: var(--space-3); }
</style>
