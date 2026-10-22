<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { documentsApi, reportsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'

const props = defineProps<{
  open: boolean
  unitOptions: { value: string; label: string }[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'created': []
}>()

const { t } = useI18n()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const uploadingPhotos = ref(false)

const form = ref({ title: '', description: '', unit_id: '' })
const selectedPhotos = ref<File[]>([])

watch(() => props.open, (open) => {
  if (open) {
    form.value = { title: '', description: '', unit_id: '' }
    selectedPhotos.value = []
    feedback.reset()
  } else {
    forceClose()
  }
})

function photoPreview(file: File): string {
  return URL.createObjectURL(file)
}

async function uploadPhoto(reportId: string, file: File): Promise<void> {
  const ext = file.name.split('.').pop() ?? 'jpg'
  const mimeType = file.type || 'image/jpeg'
  const urlRes = await documentsApi.getUploadUrl(ext, mimeType, file.size)
  const { url, key, bucket } = urlRes.data
  await documentsApi.uploadToS3(file, url, mimeType)
  const docRes = await documentsApi.confirmUpload({
    key,
    bucket,
    original_name: file.name,
    mime_type: mimeType,
    size_bytes: file.size,
    visible_roles: ['manager', 'board_member', 'co_owner'],
  })
  await reportsApi.attachDocument(reportId, docRes.data.id)
}

async function submit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!form.value.title.trim()) return
  saving.value = true
  feedback.reset()
  try {
    const reportRes = await reportsApi.create({
      title: form.value.title.trim(),
      description: form.value.description.trim() || undefined,
      unit_id: form.value.unit_id || undefined,
    })
    const reportId = reportRes.data.id
    if (selectedPhotos.value.length) {
      uploadingPhotos.value = true
      const errors: string[] = []
      for (const photo of selectedPhotos.value) {
        try {
          await uploadPhoto(reportId, photo)
        } catch {
          errors.push(photo.name)
        }
      }
      uploadingPhotos.value = false
      if (errors.length) {
        feedback.setError(t('reports.photoUploadError') + ': ' + errors.join(', '))
        setTimeout(() => { emit('created'); emit('update:open', false) }, 2000)
        return
      }
    }
    feedback.setSuccess(t('reports.createSuccess'))
    emit('created')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    saving.value = false
    uploadingPhotos.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('reports.create')"
    size="md"
    :closable="!saving && !uploadingPhotos"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="create-report"
      @submit="submit"
    >
      <CFormInput
        v-model="form.title"
        :label="t('common.title')"
        :placeholder="t('common.title')"
        required
      />
      <CFormTextarea
        v-model="form.description"
        :label="t('common.description')"
        :rows="3"
      />
      <CFormSelect
        v-model="form.unit_id"
        :label="t('claims.fields.unit')"
        :options="unitOptions"
      />

      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('reports.photos') }}</label>
        <CFileDropzone
          v-model:files="selectedPhotos"
          :hint="t('reports.photoHint')"
          accept="image/*"
        >
          <template #preview="{ files, remove }">
            <div class="photo-preview-strip">
              <div
                v-for="(photo, idx) in files"
                :key="idx"
                class="photo-preview-item"
              >
                <img
                  :src="photoPreview(photo)"
                  :alt="photo.name"
                  class="photo-thumb"
                />
                <button
                  type="button"
                  class="photo-remove"
                  :title="t('common.delete')"
                  @click="remove(idx)"
                >
                  ×
                </button>
              </div>
            </div>
          </template>
        </CFileDropzone>
        <span
          v-if="uploadingPhotos"
          class="body-xs"
          style="color:var(--color-on-surface-variant)"
        >{{ t('reports.photoUploading') }}</span>
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
          :loading="saving || uploadingPhotos"
          :closing="feedback.status.value === 'success'"
          type="submit"
          form="create-report"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>
.photo-preview-strip {
  display: flex; flex-wrap: wrap; gap: var(--space-2); margin-top: var(--space-2);
}
.photo-preview-item {
  position: relative; width: var(--space-16); height: var(--space-16); flex-shrink: 0;
}
.photo-thumb {
  width: var(--space-16); height: var(--space-16); object-fit: cover;
  border-radius: var(--radius-lg); border: 1px solid var(--color-outline-variant);
}
.photo-remove {
  position: absolute; top: -6px; right: -6px;
  background: var(--color-error); color: var(--color-on-error); border: none;
  border-radius: var(--radius-full); width: var(--space-5); height: var(--space-5);
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; font-size: 14px; line-height: 1;
  min-height: unset; min-width: unset;
}
</style>
